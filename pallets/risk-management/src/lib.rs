#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use core::option::Option;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use primitive_types::U256;
	use sp_arithmetic::traits::Zero;
	use sp_arithmetic::FixedI128;
	use zkx_support::traits::{
		MarketInterface, RiskManagementInterface, TradingAccountInterface, TradingInterface,
	};
	use zkx_support::types::{Direction, Order, OrderType, PositionDetailsForRiskManagement, Side};

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type TradingPallet: TradingInterface;
		type TradingAccountPallet: TradingAccountInterface;
		type MarketPallet: MarketInterface;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// position marked to be deleveraged
		PositionMarkedToBeDeleveraged,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Emitted when mark under collateralized position is called
		MarkUnderCollateralizedPositionCalled {
			account_id: U256,
			liq_result: bool,
			least_collateral_ratio_position: PositionDetailsForRiskManagement,
		},
	}

	// Pallet callable functions
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Function to mark under collateralized position
		#[pallet::weight(0)]
		pub fn mark_under_collateralized_position(
			origin: OriginFor<T>,
			account_id: U256,
			collateral_id: u128,
		) -> DispatchResult {
			// Make sure the caller is from a signed origin
			let _ = ensure_signed(origin)?;

			let liquidatable_position =
				T::TradingPallet::get_deleveragable_or_liquidatable_position(
					account_id,
					collateral_id,
				);

			ensure!(
				liquidatable_position.amount_to_be_sold == FixedI128::zero(),
				Error::<T>::PositionMarkedToBeDeleveraged
			);

			let (
				liq_result,
				_,
				_,
				_,
				_,
				least_collateral_ratio,
				least_collateral_ratio_position,
				least_collateral_ratio_position_asset_price,
			) = T::TradingAccountPallet::get_margin_info(
				account_id,
				collateral_id,
				FixedI128::zero(),
				FixedI128::zero(),
			);

			if least_collateral_ratio_position_asset_price == FixedI128::zero() {
				return Ok(());
			}

			if liq_result == true {
				// if margin ratio is <=0, we directly perform liquidation else we check for deleveraging
				if least_collateral_ratio > FixedI128::zero() {
					let amount_to_be_sold = Self::check_deleveraging(
						&least_collateral_ratio_position,
						least_collateral_ratio_position_asset_price,
					);
					T::TradingPallet::liquidate_position(
						account_id,
						collateral_id,
						&least_collateral_ratio_position,
						amount_to_be_sold,
					);
				} else {
					T::TradingPallet::liquidate_position(
						account_id,
						collateral_id,
						&least_collateral_ratio_position,
						FixedI128::zero(),
					);
				}
			}

			Self::deposit_event(Event::MarkUnderCollateralizedPositionCalled {
				account_id,
				liq_result,
				least_collateral_ratio_position,
			});

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn check_deleveraging(
			position: &PositionDetailsForRiskManagement,
			asset_price: FixedI128,
		) -> FixedI128 {
			let market = T::MarketPallet::get_market(position.market_id).unwrap();
			let req_margin = market.maintenance_margin_fraction;

			let margin_amount = position.margin_amount;
			let borrowed_amount = position.borrowed_amount;
			let position_size = position.size;

			let price_diff;
			if position.direction == Direction::Long {
				price_diff = asset_price - position.avg_execution_price;
			} else {
				price_diff = position.avg_execution_price - asset_price;
			}

			// Calculate amount to be sold for deleveraging
			let maintenance_requirement = req_margin * asset_price;
			let price_diff_maintenance = maintenance_requirement - price_diff;
			let amount_to_be_present = margin_amount / price_diff_maintenance;
			let amount_to_be_sold = position_size - amount_to_be_present;

			// Calculate the leverage after deleveraging
			let position_value = margin_amount + borrowed_amount;
			let amount_to_be_sold_value = amount_to_be_sold * position.avg_execution_price;
			let remaining_position_value = position_value - amount_to_be_sold_value;
			let leverage_after_deleveraging = remaining_position_value / margin_amount;

			if leverage_after_deleveraging <= 2.into() {
				FixedI128::zero()
			} else {
				amount_to_be_sold
			}
		}
	}

	impl<T: Config> RiskManagementInterface for Pallet<T> {
		fn check_for_risk(
			order: &Order,
			size: FixedI128,
			execution_price: FixedI128,
			oracle_price: FixedI128,
			margin_amount: FixedI128,
		) -> (FixedI128, bool) {
			// Fetch the maintanence margin requirement from Markets pallet
			let market = T::MarketPallet::get_market(order.market_id).unwrap();
			let req_margin = market.maintenance_margin_fraction;

			let leveraged_position_value = execution_price * size;
			let maintenance_requirement = req_margin * leveraged_position_value;

			let (liq_result, _, available_margin, _, _, _, _, _) =
				T::TradingAccountPallet::get_margin_info(
					order.account_id,
					market.asset_collateral,
					maintenance_requirement,
					margin_amount,
				);

			let mut is_error: bool = false;
			if liq_result == true {
				is_error = true;
			} else {
				if (order.direction == Direction::Short)
					&& (order.side == Side::Buy)
					&& (order.order_type == OrderType::Limit)
				{
					let price_diff = oracle_price - execution_price;
					let pnl = price_diff * size;

					// check whether user have enough balance to cover the immediate losses.
					is_error = if available_margin <= pnl { true } else { false };
				}
			}
			return (available_margin, is_error);
		}
	}
}
