#![cfg_attr(not(feature = "std"), no_std)]

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use core::option::Option;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use primitive_types::U256;
	use sp_arithmetic::fixed_point::FixedI128;
	use zkx_support::traits::TradingFeesInterface;
	use zkx_support::types::{BaseFee, Discount};

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::storage]
	#[pallet::getter(fn max_base_tier)]
	pub(super) type MaxBaseFeeTier<T> = StorageValue<_, u8, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn max_discount_tier)]
	pub(super) type MaxDiscountTier<T> = StorageValue<_, u8, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn base_fee_tier)]
	pub(super) type BaseFeeTierMap<T: Config> =
		StorageMap<_, Blake2_128Concat, u8, BaseFee, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn discount_tier)]
	pub(super) type DiscountTierMap<T: Config> =
		StorageMap<_, Blake2_128Concat, u8, Discount, ValueQuery>;

	#[pallet::error]
	pub enum Error<T> {
		/// Invalid tier
		InvalidTier,
		/// Invalid fee
		InvalidFee,
		/// Invalid discount
		InvalidDiscount,
		/// Invalid number of tokens
		InvalidNumberOfTokens,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Base fee and discount details updated
		BaseFeeAndDiscountUpdated { tier: u8, fee_details: BaseFee, discount_details: Discount },
	}

	// Pallet callable functions
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// External function for updating fee and discount details
		#[pallet::weight(0)]
		pub fn update_base_fees_and_discount(
			origin: OriginFor<T>,
			tier: u8,
			fee_details: BaseFee,
			discount_details: Discount,
		) -> DispatchResult {
			// Make sure the caller is from a signed origin
			let sender = ensure_signed(origin)?;

			ensure!(tier > 0_u8, Error::<T>::InvalidTier);
			ensure!(
				fee_details.number_of_tokens >= U256::from(0),
				Error::<T>::InvalidNumberOfTokens
			);
			ensure!(fee_details.maker_fee >= 0.into(), Error::<T>::InvalidFee);
			ensure!(fee_details.taker_fee >= 0.into(), Error::<T>::InvalidFee);
			ensure!(
				discount_details.number_of_tokens >= U256::from(0),
				Error::<T>::InvalidNumberOfTokens
			);
			ensure!(discount_details.discount >= 0.into(), Error::<T>::InvalidDiscount);

			// Get the max base fee tier
			let current_max_base_fee_tier = MaxBaseFeeTier::<T>::get();
			ensure!(tier <= current_max_base_fee_tier + 1_u8, Error::<T>::InvalidTier);

			// Verify whether the base fee of the tier being updated/added is correct
			// with respect to the lower tier, if lower tier exists
			let lower_tier_fee = BaseFeeTierMap::<T>::get(tier - 1_u8);
			let lower_tier_discount = DiscountTierMap::<T>::get(tier - 1_u8);
			if tier - 1_u8 != 0 {
				ensure!(
					lower_tier_fee.number_of_tokens < fee_details.number_of_tokens,
					Error::<T>::InvalidNumberOfTokens
				);
				ensure!(fee_details.maker_fee < lower_tier_fee.maker_fee, Error::<T>::InvalidFee);
				ensure!(fee_details.taker_fee < lower_tier_fee.taker_fee, Error::<T>::InvalidFee);
				ensure!(
					lower_tier_discount.number_of_tokens < discount_details.number_of_tokens,
					Error::<T>::InvalidNumberOfTokens
				);
				ensure!(
					lower_tier_discount.discount < discount_details.discount,
					Error::<T>::InvalidDiscount
				);
			} else {
				ensure!(
					lower_tier_fee.number_of_tokens == U256::from(0),
					Error::<T>::InvalidNumberOfTokens
				);
				ensure!(
					lower_tier_discount.number_of_tokens == U256::from(0),
					Error::<T>::InvalidNumberOfTokens
				);
			}

			// Verify whether the base fee of the tier being updated/added is correct
			// with respect to the upper tier, if upper tier exists
			let upper_tier_fee = BaseFeeTierMap::<T>::get(tier + 1_u8);
			let upper_tier_discount = DiscountTierMap::<T>::get(tier + 1_u8);
			if current_max_base_fee_tier > tier {
				ensure!(
					fee_details.number_of_tokens < upper_tier_fee.number_of_tokens,
					Error::<T>::InvalidNumberOfTokens
				);
				ensure!(upper_tier_fee.maker_fee < fee_details.maker_fee, Error::<T>::InvalidFee);
				ensure!(upper_tier_fee.taker_fee < fee_details.taker_fee, Error::<T>::InvalidFee);
				ensure!(
					discount_details.number_of_tokens < upper_tier_discount.number_of_tokens,
					Error::<T>::InvalidNumberOfTokens
				);
				ensure!(
					discount_details.discount < upper_tier_discount.discount,
					Error::<T>::InvalidDiscount
				);
			} else {
				MaxBaseFeeTier::<T>::put(tier);
				BaseFeeTierMap::<T>::insert(tier, fee_details.clone());
				MaxDiscountTier::<T>::put(tier);
				DiscountTierMap::<T>::insert(tier, discount_details.clone());
			}

			// Emit event
			Self::deposit_event(Event::BaseFeeAndDiscountUpdated {
				tier,
				fee_details,
				discount_details,
			});

			Ok(())
		}
	}

	impl<T: Config> TradingFeesInterface for Pallet<T> {
		fn get_fee_rate(user: U256, side: bool, number_of_tokens: U256) -> (FixedI128, u8, u8) {
			// Get the max base fee tier
			let current_max_base_fee_tier = MaxBaseFeeTier::<T>::get();
			// Calculate base fee of the maker, taker and base fee tier
			let (base_fee_maker, base_fee_taker, base_fee_tier) =
				Self::find_user_base_fee(number_of_tokens, current_max_base_fee_tier);

			// Get the max discount tier
			let current_max_discount_tier = MaxDiscountTier::<T>::get();
			// Calculate the discount and discount tier
			let (discount, discount_tier) =
				Self::find_user_discount(number_of_tokens, current_max_discount_tier);

			// Get the fee according to the side
			let base_fee;
			if side == true {
				base_fee = base_fee_maker;
			} else {
				base_fee = base_fee_taker;
			}

			// Calculate fee after the discount
			let one: FixedI128 = 1.into();
			let non_discount: FixedI128 = one - discount;
			let fee: FixedI128 = base_fee * non_discount;

			return (fee, base_fee_tier, discount_tier);
		}
	}

	// Pallet internal functions
	impl<T: Config> Pallet<T> {
		fn find_user_base_fee(
			number_of_tokens: U256,
			current_max_base_fee_tier: u8,
		) -> (FixedI128, FixedI128, u8) {
			let mut tier = current_max_base_fee_tier;
			let mut fee_details = BaseFeeTierMap::<T>::get(tier);
			let mut result;
			while tier >= 1 {
				fee_details = BaseFeeTierMap::<T>::get(tier);
				result = number_of_tokens - fee_details.number_of_tokens;
				if result >= U256::from(0) {
					break;
				}
				tier -= 1;
			}
			return (fee_details.maker_fee, fee_details.taker_fee, tier);
		}

		fn find_user_discount(
			number_of_tokens: U256,
			current_max_discount_tier: u8,
		) -> (FixedI128, u8) {
			let mut tier = current_max_discount_tier;
			let mut discount_details = DiscountTierMap::<T>::get(tier);
			let mut result;
			while tier >= 1 {
				discount_details = DiscountTierMap::<T>::get(tier);
				result = number_of_tokens - discount_details.number_of_tokens;
				if result >= U256::from(0) {
					break;
				}
				tier -= 1;
			}
			return (discount_details.discount, tier);
		}
	}
}
