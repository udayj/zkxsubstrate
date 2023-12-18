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
		types::{BaseFee, FeeRates, OrderSide, Side},
	};
	use sp_arithmetic::{fixed_point::FixedI128, traits::Zero};

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type AssetPallet: AssetInterface;
	}

	#[pallet::storage]
	#[pallet::getter(fn max_base_fee_tier)]
	pub(super) type MaxBaseFeeTier<T> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		u128, // collateral_id
		Blake2_128Concat,
		Side, // buy or sell
		u8,
		ValueQuery,
	>;

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

			// Delete the fee details corresponding to the current side
			MaxBaseFeeTier::<T>::remove(collateral_id, side);
			let max_fee_tier = MaxBaseFeeTier::<T>::get(collateral_id, side);
			for i in 1..max_fee_tier + 1 {
				BaseFeeTierMap::<T>::remove(collateral_id, (i, side));
			}

			let fee_details_length = fee_details.len();
			ensure!(fee_details_length >= 1, Error::<T>::ZeroFeeTiers);

			let update_base_fee_response = Self::update_base_fee(collateral_id, side, fee_details);
			match update_base_fee_response {
				Ok(()) => (),
				Err(e) => return Err(e),
			}

			// Emit event
			Self::deposit_event(
				Event::BaseFeesUpdated { fee_tiers: u8::try_from(fee_details_length).unwrap() }
			);

			Ok(())
		}
	}

	impl<T: Config> TradingFeesInterface for Pallet<T> {
		fn get_fee_rate(
			collateral_id: u128,
			side: Side,
			order_side: OrderSide,
			volume: FixedI128,
		) -> (FixedI128, u8) {
			// Get the max base fee tier
			let current_max_base_fee_tier = MaxBaseFeeTier::<T>::get(collateral_id, side);
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

		fn get_all_fee_rates(collateral_id: u128, volume: FixedI128) -> FeeRates {
			// Get the max base fee tier
			let current_max_base_fee_tier_buy = MaxBaseFeeTier::<T>::get(collateral_id, Side::Buy);
			// Calculate base fee of the maker, taker and base fee tier
			let (maker_buy, taker_buy, _) = Self::find_user_base_fee(
				collateral_id,
				Side::Buy,
				volume,
				current_max_base_fee_tier_buy,
			);
			let current_max_base_fee_tier_sell =
				MaxBaseFeeTier::<T>::get(collateral_id, Side::Sell);
			let (maker_sell, taker_sell, _) = Self::find_user_base_fee(
				collateral_id,
				Side::Sell,
				volume,
				current_max_base_fee_tier_sell,
			);

			FeeRates { maker_buy, maker_sell, taker_buy, taker_sell }
		}
	}

	// Pallet internal functions
	impl<T: Config> Pallet<T> {
		fn find_user_base_fee(
			collateral_id: u128,
			side: Side,
			volume: FixedI128,
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
			fee_details: Vec<BaseFee>,
		) -> DispatchResult {
			let mut fee_info: BaseFee;

			for index in 0..fee_details.len() {
				fee_info = fee_details[index];
				ensure!(fee_info.volume >= FixedI128::zero(), Error::<T>::InvalidVolume);
				ensure!(fee_info.maker_fee >= FixedI128::zero(), Error::<T>::InvalidFee);
				ensure!(fee_info.taker_fee >= FixedI128::zero(), Error::<T>::InvalidFee);

				// Verify whether the base fee of the tier being updated/added is correct
				// with respect to the lower tier, if lower tier exists
				let lower_tier_fee = BaseFeeTierMap::<T>::get(collateral_id, (index as u8, side));
				if index != 0 {
					ensure!(lower_tier_fee.volume < fee_info.volume, Error::<T>::InvalidVolume);
					ensure!(fee_info.maker_fee < lower_tier_fee.maker_fee, Error::<T>::InvalidFee);
					ensure!(fee_info.taker_fee < lower_tier_fee.taker_fee, Error::<T>::InvalidFee);
				} else {
					ensure!(lower_tier_fee.volume == FixedI128::zero(), Error::<T>::InvalidVolume);
				}
				BaseFeeTierMap::<T>::insert(collateral_id, ((index + 1) as u8, side), fee_info);
			}
			let max_tier = fee_details.len() as u8;
			MaxBaseFeeTier::<T>::insert(collateral_id, side, max_tier);

			Ok(())
		}
	}
}
