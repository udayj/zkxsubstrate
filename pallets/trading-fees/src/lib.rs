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
		pallet_prelude::*,
	};
	use frame_system::pallet_prelude::*;
	use pallet_support::{
		traits::{AssetInterface, MarketInterface, TradingFeesInterface},
		types::{BaseFee, BaseFeeAggregate, FeeRates, OrderSide, Side},
	};
	use sp_arithmetic::{fixed_point::FixedI128, traits::Zero};

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
		Blake2_128Concat,
		u128, // collateral_id or market_id
		BaseFeeAggregate,
		ValueQuery,
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
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Base fees details updated
		BaseFeesUpdated { fee_tiers: u8 },
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
	}

	impl<T: Config> TradingFeesInterface for Pallet<T> {
		fn remove_base_fees_internal(id: u128) {
			// Delete all combinations of OrderSide and Side
			for side in &[Side::Buy, Side::Sell] {
				for order_side in &[OrderSide::Maker, OrderSide::Taker] {
					let max_fee_tier = MaxBaseFeeTier::<T>::get(id, order_side);
					for i in 1..max_fee_tier + 1 {
						BaseFeeTierMap::<T>::remove(id, (i, side, &order_side));
					}
					MaxBaseFeeTier::<T>::remove(id, order_side);
				}
			}
		}

		fn update_base_fees_internal(id: u128, fee_details: BaseFeeAggregate) -> DispatchResult {
			// Validate that the asset exists and it is a collateral
			if let Some(asset) = T::AssetPallet::get_asset(id) {
				ensure!(asset.is_collateral, Error::<T>::AssetNotCollateral);
			} else {
				// If it's not an asset, ensure that it's a valid market
				ensure!(T::MarketPallet::get_market(id).is_some(), Error::<T>::MarketNotFound);
			}

			Self::validate_fee_details(&fee_details);

			// Remove any fees if present
			BaseFeeMap::<T>::remove(id);

			// Add it to storage
			BaseFeeMap::<T>::set(id, fee_details);

			Ok(())
		}
	}

	// Pallet internal functions
	impl<T: Config> Pallet<T> {
		fn validate_fee_details(fee_details: &BaseFeeAggregate) -> DispatchResult {
			// Validate each variant of BaseFee
			Self::validate_base_fees(&fee_details.maker_buy);
			Self::validate_base_fees(&fee_details.maker_sell);
			Self::validate_base_fees(&fee_details.taker_buy);
			Self::validate_base_fees(&fee_details.taker_sell);

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
				// Adjust this comparison based on your actual fee structure requirements
				ensure!(current_fee.fee < prev_fee.fee, Error::<T>::InvalidFee);
			}

			Ok(())
		}
	}
}
