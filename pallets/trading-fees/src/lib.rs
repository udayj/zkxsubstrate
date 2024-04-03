#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use core::option::Option;
	use frame_support::{
		dispatch::{DispatchResult, Vec},
		pallet_prelude::{OptionQuery, ValueQuery, *},
	};
	use frame_system::pallet_prelude::*;
	use pallet_support::{
		traits::{AssetInterface, MarketInterface, TradingFeesInterface},
		types::{BaseFee, BaseFeeAggregate, FeeShareDetails, OrderSide, Side},
	};
	use sp_arithmetic::{
		fixed_point::FixedI128,
		traits::{One, Zero},
	};

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type AssetPallet: AssetInterface;
		type MarketPallet: MarketInterface;
	}

	#[pallet::storage]
	#[pallet::getter(fn max_base_fee_tier)]
	pub(super) type MaxBaseFeeTier<T> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		u128, // collateral_id or market_id
		Blake2_128Concat,
		OrderSide, // maker or taker
		u8,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn base_fee_tier)]
	pub(super) type BaseFeeTierMap<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		u128, // collateral_id or market_id
		Blake2_128Concat,
		(u8, Side, OrderSide), // (tier, buy or sell, maker or taker)
		BaseFee,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn base_fees_all)]
	pub(super) type BaseFeeMap<T: Config> = StorageMap<
		_,
		Twox64Concat,
		u128, // collateral_id or market_id
		BaseFeeAggregate,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn fee_share)]
	pub(super) type FeeShare<T: Config> = StorageMap<
		_,
		Twox64Concat,
		u128, // collateral_id
		Vec<Vec<FeeShareDetails>>,
		OptionQuery,
	>;

	#[pallet::error]
	pub enum Error<T> {
		/// Invalid fee
		InvalidFee,
		/// Invalid number of tokens
		InvalidVolume,
		/// There should be atleast one fee tier
		ZeroFeeTiers,
		/// Asset does not exist
		AssetNotFound,
		/// Asset is not a collateral
		AssetNotCollateral,
		/// Market does not exist
		MarketNotFound,
		/// Empty array passed for
		EmptyFeeShares,
		/// Invalid value for fee share
		InvalidFeeShare,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Base fees details updated
		/// No longer used
		BaseFeesUpdated {
			fee_tiers: u8,
		},
		BaseFeeAggregateSet {
			id: u128,
			base_fee_aggregate: BaseFeeAggregate,
		},
		FeeShareSet {
			fee_share: Vec<Vec<FeeShareDetails>>,
		},
	}

	// Pallet callable functions
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// External function for updating fee details
		#[pallet::weight(0)]
		pub fn update_base_fees(
			origin: OriginFor<T>,
			id: u128,
			fee_details: BaseFeeAggregate,
		) -> DispatchResult {
			// Make sure the caller is root
			ensure_root(origin)?;

			Self::update_base_fees_internal(id, fee_details)?;
			Ok(())
		}

		/// External function for updating fee share details
		#[pallet::weight(0)]
		pub fn update_fee_share(
			origin: OriginFor<T>,
			id: u128,
			fee_share_details: Vec<Vec<FeeShareDetails>>,
		) -> DispatchResult {
			// Make sure the caller is root
			ensure_root(origin)?;

			Self::update_fee_shares_internal(id, fee_share_details)?;
			Ok(())
		}
	}

	impl<T: Config> TradingFeesInterface for Pallet<T> {
		fn update_base_fees_internal(id: u128, fee_details: BaseFeeAggregate) -> DispatchResult {
			// Validate that the asset exists and it is a collateral
			if let Some(asset) = T::AssetPallet::get_asset(id) {
				ensure!(asset.is_collateral, Error::<T>::AssetNotCollateral);
			} else {
				// If it's not an asset, ensure that it's a valid market
				ensure!(T::MarketPallet::get_market(id).is_some(), Error::<T>::MarketNotFound);
			}

			// Validate the fee details
			Self::validate_fee_details(&fee_details)?;

			// Remove any fees if present
			BaseFeeMap::<T>::remove(id);

			// Add it to storage
			BaseFeeMap::<T>::set(id, Some(fee_details.clone()));

			Self::deposit_event(Event::BaseFeeAggregateSet { id, base_fee_aggregate: fee_details });

			Ok(())
		}

		fn get_all_fees(market_id: u128, collateral_id: u128) -> BaseFeeAggregate {
			// First try to fetch market fees
			// If it doesn't exist, fetch asset fees
			// NOTE: Asset fees can be 0
			BaseFeeMap::<T>::get(market_id)
				.or_else(|| BaseFeeMap::<T>::get(collateral_id))
				.unwrap_or_else(BaseFeeAggregate::default)
		}

		fn update_fee_shares_internal(
			id: u128,
			fee_share_details: Vec<Vec<FeeShareDetails>>,
		) -> DispatchResult {
			for level in 0..fee_share_details.len() {
				// Validate the fee share details
				Self::validate_fee_shares(&fee_share_details[level])?;
			}

			// Remove any fee share details if present
			FeeShare::<T>::remove(id);

			// Add it to storage
			FeeShare::<T>::insert(id, &fee_share_details);

			Self::deposit_event(Event::FeeShareSet { fee_share: fee_share_details });

			Ok(())
		}

		fn get_all_fee_shares(id: u128) -> Vec<Vec<FeeShareDetails>> {
			FeeShare::<T>::get(id).unwrap_or_default()
		}

		fn get_fee_share(account_level: u8, id: u128, volume: FixedI128) -> FixedI128 {
			let fee_share_details = FeeShare::<T>::get(id);

			// If no fee share tiers are set, return 0 as fee share
			if fee_share_details.is_none() {
				return FixedI128::zero();
			}

			let fee_share_details = fee_share_details.unwrap();

			// If account level is invalid, return 0 as fee share
			if account_level as usize > &fee_share_details.len() - 1 {
				return FixedI128::zero();
			}

			// Find the appropriate fee share tier for the user
			let fee_share_details = &fee_share_details[account_level as usize];
			for (_, tier) in fee_share_details.iter().enumerate().rev() {
				if volume >= tier.volume {
					return tier.fee_share;
				}
			}

			// If volume is not greater than any tier's volume, it falls into the lowest tier
			fee_share_details[0].fee_share
		}
	}

	// Pallet internal functions
	impl<T: Config> Pallet<T> {
		fn validate_fee_details(fee_details: &BaseFeeAggregate) -> DispatchResult {
			// Validate each variant of BaseFee
			Self::validate_base_fees(&fee_details.maker_buy)?;
			Self::validate_base_fees(&fee_details.maker_sell)?;
			Self::validate_base_fees(&fee_details.taker_buy)?;
			Self::validate_base_fees(&fee_details.taker_sell)?;

			Ok(())
		}

		fn validate_base_fees(base_fees: &Vec<BaseFee>) -> DispatchResult {
			// The base_fees array cannot be empty
			ensure!(!base_fees.is_empty(), Error::<T>::ZeroFeeTiers);

			// Validate the first fee tier
			let first_fee = &base_fees[0];
			ensure!(first_fee.fee >= FixedI128::zero(), Error::<T>::InvalidFee);
			// The volume of first tier must be 0
			ensure!(first_fee.volume == FixedI128::zero(), Error::<T>::InvalidVolume);

			// Validate the base fees in pairs;
			// Each base_fee's fee < previous base_fee's fee
			// Each base_fee's volume > previous base_fee's volume
			for window in base_fees.windows(2) {
				let (prev_fee, current_fee) = (&window[0], &window[1]);

				// Ensure volume increases with each tier
				ensure!(current_fee.volume > prev_fee.volume, Error::<T>::InvalidVolume);
				// Fee in each tier must be less than or equal to the previous one
				ensure!(current_fee.fee <= prev_fee.fee, Error::<T>::InvalidFee);
			}

			Ok(())
		}

		fn validate_fee_shares(fee_shares: &Vec<FeeShareDetails>) -> DispatchResult {
			// The fee_shares array cannot be empty
			ensure!(!fee_shares.is_empty(), Error::<T>::EmptyFeeShares);

			// Validate the first fee tier
			let first_fee_share = &fee_shares[0];
			// The volume of first tier must be 0
			ensure!(first_fee_share.volume == FixedI128::zero(), Error::<T>::InvalidVolume);
			// Validate fee share is between 0 and 1
			ensure!(
				first_fee_share.fee_share >= FixedI128::zero() &&
					first_fee_share.fee_share <= FixedI128::one(),
				Error::<T>::InvalidFeeShare
			);

			// Validate the fee shares in pairs;
			// Each tier's fee share >= previous tier's fee share
			// Each tier's volume > previous tier's volume
			for window in fee_shares.windows(2) {
				let (prev_fee_share, current_fee_share) = (&window[0], &window[1]);

				// Ensure volume increases with each tier
				ensure!(
					current_fee_share.volume > prev_fee_share.volume,
					Error::<T>::InvalidVolume
				);
				// Fee share in each tier must be more than equal to the previous one
				ensure!(
					current_fee_share.fee_share >= prev_fee_share.fee_share,
					Error::<T>::InvalidFee
				);
				// Validate fee share is between 0 and 1
				ensure!(
					current_fee_share.fee_share >= FixedI128::zero() &&
						current_fee_share.fee_share <= FixedI128::one(),
					Error::<T>::InvalidFeeShare
				);
			}

			Ok(())
		}
	}
}
