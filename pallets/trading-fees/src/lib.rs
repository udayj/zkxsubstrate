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
		traits::TradingFeesInterface,
		types::{BaseFee, Discount, OrderSide, Side},
	};
	use primitive_types::U256;
	use sp_arithmetic::{fixed_point::FixedI128, traits::Zero};

	static DELETION_LIMIT: u32 = 100;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::storage]
	#[pallet::getter(fn max_base_fee_tier)]
	pub(super) type MaxBaseFeeTier<T> = StorageValue<_, u8, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn max_discount_tier)]
	pub(super) type MaxDiscountTier<T> = StorageValue<_, u8, ValueQuery>;

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

	#[pallet::storage]
	#[pallet::getter(fn discount_tier)]
	pub(super) type DiscountTierMap<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		u128, // collateral_id
		Blake2_128Concat,
		(u8, Side), // (tier, buy or sell)
		Discount,
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
		/// Invalid discount
		InvalidDiscount,
		/// Discount tiers length mismatch
		DiscountTiersLengthMismatch,
		/// Invalid number of tokens
		InvalidNumberOfTokens,
		/// There should be atleast one fee tier
		ZeroFeeTiers,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Base fees and discounts details updated
		BaseFeesAndDiscountsUpdated { fee_tiers: u8, discount_tiers: u8 },
	}

	// Pallet callable functions
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// External function for updating fee and discount details
		#[pallet::weight(0)]
		pub fn update_base_fees_and_discounts(
			origin: OriginFor<T>,
			collateral_id: u128,
			side: Side,
			fee_tiers: Vec<u8>,
			fee_details: Vec<BaseFee>,
			discount_tiers: Vec<u8>,
			discount_details: Vec<Discount>,
		) -> DispatchResult {
			// Make sure the caller is from a signed origin
			let _ = ensure_signed(origin)?;

			// Clear all mappings
			let _ = MaxBaseFeeTier::<T>::kill();
			let _ = MaxDiscountTier::<T>::kill();
			let _ = BaseFeeTierMap::<T>::clear_prefix(collateral_id, DELETION_LIMIT, None);
			let _ = DiscountTierMap::<T>::clear_prefix(collateral_id, DELETION_LIMIT, None);

			ensure!(fee_tiers.len() == fee_details.len(), Error::<T>::FeeTiersLengthMismatch);
			ensure!(fee_tiers.len() >= 1, Error::<T>::ZeroFeeTiers);

			let update_base_fee_response =
				Self::update_base_fee(collateral_id, side, &fee_tiers, fee_details);
			match update_base_fee_response {
				Ok(()) => (),
				Err(e) => return Err(e),
			}

			ensure!(
				discount_tiers.len() == discount_details.len(),
				Error::<T>::DiscountTiersLengthMismatch
			);
			let update_discount_response =
				Self::update_discount(collateral_id, side, &discount_tiers, discount_details);
			match update_discount_response {
				Ok(()) => (),
				Err(e) => return Err(e),
			}
			// Emit event
			Self::deposit_event(Event::BaseFeesAndDiscountsUpdated {
				fee_tiers: u8::try_from(fee_tiers.len()).unwrap(),
				discount_tiers: u8::try_from(discount_tiers.len()).unwrap(),
			});

			Ok(())
		}
	}

	impl<T: Config> TradingFeesInterface for Pallet<T> {
		fn get_fee_rate(
			collateral_id: u128,
			side: Side,
			order_side: OrderSide,
			volume: U256,
		) -> (FixedI128, u8, u8) {
			// Get the max base fee tier
			let current_max_base_fee_tier = MaxBaseFeeTier::<T>::get();
			// Calculate base fee of the maker, taker and base fee tier
			let (base_fee_maker, base_fee_taker, base_fee_tier) =
				Self::find_user_base_fee(collateral_id, side, volume, current_max_base_fee_tier);

			// Get the max discount tier
			let current_max_discount_tier = MaxDiscountTier::<T>::get();
			// Calculate the discount and discount tier
			let (discount, discount_tier) =
				Self::find_user_discount(collateral_id, side, volume, current_max_discount_tier);

			// Get the fee according to the side
			let base_fee;
			if order_side == OrderSide::Maker {
				base_fee = base_fee_maker;
			} else {
				base_fee = base_fee_taker;
			}

			// Calculate fee after the discount
			let one: FixedI128 = 1.into();
			let non_discount: FixedI128 = one - discount;
			let fee: FixedI128 = base_fee * non_discount;

			return (fee, base_fee_tier, discount_tier)
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

		fn find_user_discount(
			collateral_id: u128,
			side: Side,
			volume: U256,
			current_max_discount_tier: u8,
		) -> (FixedI128, u8) {
			let mut tier = current_max_discount_tier;
			let mut discount_details = DiscountTierMap::<T>::get(collateral_id, (tier, side));
			while tier >= 1 {
				discount_details = DiscountTierMap::<T>::get(collateral_id, (tier, side));
				if volume >= discount_details.volume {
					break
				}
				tier -= 1;
			}
			return (discount_details.discount, tier)
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
				ensure!(fee_info.volume >= U256::zero(), Error::<T>::InvalidNumberOfTokens);
				ensure!(fee_info.maker_fee >= FixedI128::zero(), Error::<T>::InvalidFee);
				ensure!(fee_info.taker_fee >= FixedI128::zero(), Error::<T>::InvalidFee);

				// Get the max base fee tier
				let current_max_base_fee_tier = MaxBaseFeeTier::<T>::get();
				ensure!(tier <= current_max_base_fee_tier + 1_u8, Error::<T>::InvalidTier);

				// Verify whether the base fee of the tier being updated/added is correct
				// with respect to the lower tier, if lower tier exists
				let lower_tier_fee = BaseFeeTierMap::<T>::get(collateral_id, (tier - 1_u8, side));
				if tier - 1_u8 != 0 {
					ensure!(
						lower_tier_fee.volume < fee_info.volume,
						Error::<T>::InvalidNumberOfTokens
					);
					ensure!(fee_info.maker_fee < lower_tier_fee.maker_fee, Error::<T>::InvalidFee);
					ensure!(fee_info.taker_fee < lower_tier_fee.taker_fee, Error::<T>::InvalidFee);
				} else {
					ensure!(
						lower_tier_fee.volume == U256::zero(),
						Error::<T>::InvalidNumberOfTokens
					);
				}

				// Verify whether the base fee of the tier being updated/added is correct
				// with respect to the upper tier, if upper tier exists
				let upper_tier_fee = BaseFeeTierMap::<T>::get(collateral_id, (tier + 1_u8, side));
				if current_max_base_fee_tier > tier {
					ensure!(
						fee_info.volume < upper_tier_fee.volume,
						Error::<T>::InvalidNumberOfTokens
					);
					ensure!(upper_tier_fee.maker_fee < fee_info.maker_fee, Error::<T>::InvalidFee);
					ensure!(upper_tier_fee.taker_fee < fee_info.taker_fee, Error::<T>::InvalidFee);
				} else {
					MaxBaseFeeTier::<T>::put(tier);
				}
				BaseFeeTierMap::<T>::insert(collateral_id, (tier, side), fee_info);
			}
			Ok(())
		}

		fn update_discount(
			collateral_id: u128,
			side: Side,
			discount_tiers: &Vec<u8>,
			discount_details: Vec<Discount>,
		) -> DispatchResult {
			let mut tier: u8;
			let mut discount_info: Discount;
			for pos in 0..discount_tiers.len() {
				tier = discount_tiers[pos];
				discount_info = discount_details[pos];
				ensure!(tier > 0_u8, Error::<T>::InvalidTier);
				ensure!(discount_info.volume >= U256::zero(), Error::<T>::InvalidNumberOfTokens);
				ensure!(discount_info.discount >= FixedI128::zero(), Error::<T>::InvalidDiscount);

				// Get the max base fee tier
				let current_max_discount_tier = MaxDiscountTier::<T>::get();
				ensure!(tier <= current_max_discount_tier + 1_u8, Error::<T>::InvalidTier);

				// Verify whether the discount of the tier being updated/added is correct
				// with respect to the lower tier, if lower tier exists
				let lower_tier_discount =
					DiscountTierMap::<T>::get(collateral_id, (tier - 1_u8, side));
				if tier - 1_u8 != 0 {
					ensure!(
						lower_tier_discount.volume < discount_info.volume,
						Error::<T>::InvalidNumberOfTokens
					);
					ensure!(
						lower_tier_discount.discount < discount_info.discount,
						Error::<T>::InvalidDiscount
					);
				} else {
					ensure!(
						lower_tier_discount.volume == U256::zero(),
						Error::<T>::InvalidNumberOfTokens
					);
				}

				// Verify whether the discount of the tier being updated/added is correct
				// with respect to the upper tier, if upper tier exists
				let upper_tier_discount =
					DiscountTierMap::<T>::get(collateral_id, (tier + 1_u8, side));
				if current_max_discount_tier > tier {
					ensure!(
						discount_info.volume < upper_tier_discount.volume,
						Error::<T>::InvalidNumberOfTokens
					);
					ensure!(
						discount_info.discount < upper_tier_discount.discount,
						Error::<T>::InvalidDiscount
					);
				} else {
					MaxDiscountTier::<T>::put(tier);
				}
				DiscountTierMap::<T>::insert(collateral_id, (tier, side), discount_info);
			}
			Ok(())
		}
	}
}
