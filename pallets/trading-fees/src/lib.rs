#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use core::option::Option;
	use frame_support::{dispatch::Vec, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use pallet_support::{
		traits::{AssetInterface, TradingFeesInterface},
		types::{BaseFee, OrderSide, Side},
	};
	use primitive_types::U256;
	use sp_arithmetic::{fixed_point::FixedI128, traits::Zero};

	static DELETION_LIMIT: u32 = 100;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type AssetPallet: AssetInterface;
	}

	#[pallet::storage]
	#[pallet::getter(fn max_base_fee_tier)]
	pub(super) type MaxBaseFeeTier<T> = StorageValue<_, u8, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn base_fee_tier)]
	pub(super) type BaseFeeTierMap<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		u128, // collateral_id
		Blake2_128Concat,
		(u8, Side), // (tier, buy or sell)
		BaseFee,
		ValueQuery,
	>;

	#[pallet::error]
	pub enum Error<T> {
		/// Invalid tier
		InvalidTier,
		/// Invalid fee
		InvalidFee,
		/// Fee tiers length mismatch
		FeeTiersLengthMismatch,
		/// Invalid number of tokens
		InvalidVolume,
		/// There should be atleast one fee tier
		ZeroFeeTiers,
		/// Asset does not exist
		AssetNotFound,
		/// Asset is not a collateral
		AssetNotCollateral,
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
			collateral_id: u128,
			side: Side,
			fee_tiers: Vec<u8>,
			fee_details: Vec<BaseFee>,
		) -> DispatchResult {
			// Make sure the caller is from a signed origin
			let _ = ensure_signed(origin)?;

			// Validate that the asset exists and it is a collateral
			if let Some(asset) = T::AssetPallet::get_asset(collateral_id) {
				ensure!(asset.is_collateral, Error::<T>::AssetNotCollateral);
			} else {
				ensure!(false, Error::<T>::AssetNotFound);
			}

			// Clear all mappings
			let _ = MaxBaseFeeTier::<T>::kill();
			let _ = BaseFeeTierMap::<T>::clear_prefix(collateral_id, DELETION_LIMIT, None);

			ensure!(fee_tiers.len() == fee_details.len(), Error::<T>::FeeTiersLengthMismatch);
			ensure!(fee_tiers.len() >= 1, Error::<T>::ZeroFeeTiers);

			let update_base_fee_response =
				Self::update_base_fee(collateral_id, side, &fee_tiers, fee_details);
			match update_base_fee_response {
				Ok(()) => (),
				Err(e) => return Err(e),
			}

			// Emit event
			Self::deposit_event(
				Event::BaseFeesUpdated { fee_tiers: u8::try_from(fee_tiers.len()).unwrap() }
			);

			Ok(())
		}
	}

	impl<T: Config> TradingFeesInterface for Pallet<T> {
		fn get_fee_rate(
			collateral_id: u128,
			side: Side,
			order_side: OrderSide,
			volume: U256,
		) -> (FixedI128, u8) {
			// Get the max base fee tier
			let current_max_base_fee_tier = MaxBaseFeeTier::<T>::get();
			// Calculate base fee of the maker, taker and base fee tier
			let (base_fee_maker, base_fee_taker, base_fee_tier) =
				Self::find_user_base_fee(collateral_id, side, volume, current_max_base_fee_tier);

			// Get the fee according to the side
			if order_side == OrderSide::Maker {
				(base_fee_maker, base_fee_tier)
			} else {
				(base_fee_taker, base_fee_tier)
			}
		}
	}

	// Pallet internal functions
	impl<T: Config> Pallet<T> {
		fn find_user_base_fee(
			collateral_id: u128,
			side: Side,
			volume: U256,
			current_max_base_fee_tier: u8,
		) -> (FixedI128, FixedI128, u8) {
			let mut tier = current_max_base_fee_tier;
			let mut fee_details = BaseFeeTierMap::<T>::get(collateral_id, (tier, side));
			while tier >= 1 {
				fee_details = BaseFeeTierMap::<T>::get(collateral_id, (tier, side));
				if volume >= fee_details.volume {
					break
				}
				tier -= 1;
			}
			return (fee_details.maker_fee, fee_details.taker_fee, tier)
		}

		fn update_base_fee(
			collateral_id: u128,
			side: Side,
			fee_tiers: &Vec<u8>,
			fee_details: Vec<BaseFee>,
		) -> DispatchResult {
			let mut tier: u8;
			let mut fee_info: BaseFee;
			for pos in 0..fee_tiers.len() {
				tier = fee_tiers[pos];
				fee_info = fee_details[pos];
				ensure!(tier > 0_u8, Error::<T>::InvalidTier);
				ensure!(fee_info.volume >= U256::zero(), Error::<T>::InvalidVolume);
				ensure!(fee_info.maker_fee >= FixedI128::zero(), Error::<T>::InvalidFee);
				ensure!(fee_info.taker_fee >= FixedI128::zero(), Error::<T>::InvalidFee);

				// Get the max base fee tier
				let current_max_base_fee_tier = MaxBaseFeeTier::<T>::get();
				ensure!(tier <= current_max_base_fee_tier + 1_u8, Error::<T>::InvalidTier);

				// Verify whether the base fee of the tier being updated/added is correct
				// with respect to the lower tier, if lower tier exists
				let lower_tier_fee = BaseFeeTierMap::<T>::get(collateral_id, (tier - 1_u8, side));
				if tier - 1_u8 != 0 {
					ensure!(lower_tier_fee.volume < fee_info.volume, Error::<T>::InvalidVolume);
					ensure!(fee_info.maker_fee < lower_tier_fee.maker_fee, Error::<T>::InvalidFee);
					ensure!(fee_info.taker_fee < lower_tier_fee.taker_fee, Error::<T>::InvalidFee);
				} else {
					ensure!(lower_tier_fee.volume == U256::zero(), Error::<T>::InvalidVolume);
				}

				// Verify whether the base fee of the tier being updated/added is correct
				// with respect to the upper tier, if upper tier exists
				let upper_tier_fee = BaseFeeTierMap::<T>::get(collateral_id, (tier + 1_u8, side));
				if current_max_base_fee_tier > tier {
					ensure!(fee_info.volume < upper_tier_fee.volume, Error::<T>::InvalidVolume);
					ensure!(upper_tier_fee.maker_fee < fee_info.maker_fee, Error::<T>::InvalidFee);
					ensure!(upper_tier_fee.taker_fee < fee_info.taker_fee, Error::<T>::InvalidFee);
				} else {
					MaxBaseFeeTier::<T>::put(tier);
				}
				BaseFeeTierMap::<T>::insert(collateral_id, (tier, side), fee_info);
			}
			Ok(())
		}
	}
}
