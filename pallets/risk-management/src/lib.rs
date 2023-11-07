#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use core::option::Option;
	use primitive_types::U256;
	use sp_arithmetic::{traits::Zero, FixedI128};
	use zkx_support::{
		traits::{
			FixedI128Ext, MarketInterface, PricesInterface, RiskManagementInterface,
			TradingAccountInterface, TradingInterface,
		},
		types::{
			DeleveragablePosition, Direction, ForceClosureFlag, Order, OrderType, Position, Side,
		},
	};

	static TWO_FI128: FixedI128 = FixedI128::from_inner(2000000000000000000);
	static TWO_POINT_FIVE_FI128: FixedI128 = FixedI128::from_inner(2500000000000000000);

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type TradingPallet: TradingInterface;
		type TradingAccountPallet: TradingAccountInterface;
		type MarketPallet: MarketInterface;
		type PricesPallet: PricesInterface;
	}

	impl<T: Config> Pallet<T> {
		fn is_account_deleveragable(
			account_id: U256,
			collateral_id: u128,
			market_id: u128,
			direction: Direction,
		) -> (bool, FixedI128) {
			let markets = T::TradingPallet::get_markets_of_collateral(account_id, collateral_id);
			let mut total_account_value = FixedI128::zero();
			let mut total_maintenance_margin = FixedI128::zero();

			for market in markets {
				let mark_price = T::PricesPallet::get_mark_price(market);
				// If price is 0, we abort risk management
				if mark_price == FixedI128::zero() {
					return (true, FixedI128::zero())
				}
				let long_position =
					T::TradingPallet::get_position(account_id, market, Direction::Long);
				let short_position =
					T::TradingPallet::get_position(account_id, market, Direction::Short);

				if long_position.size != FixedI128::zero() {
					Self::calculate_tav_and_tmr(
						&mut total_account_value,
						&mut total_maintenance_margin,
						long_position,
						mark_price,
					)
				}
				if short_position.size != FixedI128::zero() {
					Self::calculate_tav_and_tmr(
						&mut total_account_value,
						&mut total_maintenance_margin,
						short_position,
						mark_price,
					)
				}
			}

			let unused_balance =
				T::TradingAccountPallet::get_unused_balance(account_id, collateral_id);
			total_account_value = total_account_value + unused_balance;

			// IF TAV > TMR after selling some part of positions,
			// it means that deleveraging is possible
			if total_account_value > total_maintenance_margin {
				// Calculate amount to be sold
				let market = T::MarketPallet::get_market(market_id).unwrap();
				let req_margin_fraction = market.maintenance_margin_fraction;

				let position = T::TradingPallet::get_position(account_id, market_id, direction);
				let mark_price = T::PricesPallet::get_mark_price(market_id);

				let price_diff = if position.direction == Direction::Long {
					mark_price - position.avg_execution_price
				} else {
					position.avg_execution_price - mark_price
				};

				if (price_diff >= FixedI128::zero()) || position.leverage <= TWO_FI128 {
					return (true, FixedI128::zero())
				}

				// amount to sell = initial_size - (margin amount/(margin ratio * price - Difference
				// in prices))
				let amount_to_be_present =
					position.margin_amount / ((req_margin_fraction * mark_price) - price_diff);
				let amount_to_be_sold = position.size - amount_to_be_present;
				let amount_to_be_sold =
					amount_to_be_sold.round_to_precision(market.step_precision.into());

				// Calculate the leverage after deleveraging
				let position_value = position.margin_amount + position.borrowed_amount;
				let amount_to_be_sold_value = amount_to_be_sold * position.avg_execution_price;
				let remaining_position_value = position_value - amount_to_be_sold_value;
				let leverage_after_deleveraging = remaining_position_value / position.margin_amount;

				if leverage_after_deleveraging <= 2.into() {
					let new_size = (TWO_FI128 * position.margin_amount) / mark_price;
					let amount_to_be_sold = position.size - new_size;
					let amount_to_be_sold =
						amount_to_be_sold.round_to_precision(market.step_precision.into());
					(true, amount_to_be_sold)
				} else {
					(true, amount_to_be_sold)
				}
			} else {
				(false, FixedI128::zero())
			}
		}

		fn calculate_tav_and_tmr(
			total_account_value: &mut FixedI128,
			total_maintenance_margin: &mut FixedI128,
			position: Position,
			price: FixedI128,
		) {
			let (position_value, maintenance_margin) =
				Self::is_position_deleveragable(position, price);
			*total_account_value = *total_account_value + position_value;
			*total_maintenance_margin = *total_maintenance_margin + maintenance_margin;
		}

		fn is_position_deleveragable(
			position: Position,
			price: FixedI128,
		) -> (FixedI128, FixedI128) {
			let price_diff;
			if position.direction == Direction::Long {
				price_diff = price - position.avg_execution_price;
			} else {
				price_diff = position.avg_execution_price - price;
			}
			let pnl = position.size * price_diff;

			let position_value;
			let maintenance_requirement;

			let market = T::MarketPallet::get_market(position.market_id).unwrap();
			let req_margin_fraction = market.maintenance_margin_fraction;

			// If pnl is negative, it means that position can be deleveraged
			// Sell the position such that resulting leverage is 2.5
			// amount_to_sell = initial_size - ((2.5 * margin_amount)/current_asset_price)
			if pnl < FixedI128::zero() {
				let new_size = (TWO_POINT_FIVE_FI128 * position.margin_amount) / price;
				let new_size = new_size.round_to_precision(market.step_precision.into());
				let value_to_sell = (position.size - new_size) * price;
				// Calculate the new borrowed amount if the total position size is
				// above calculated new_size
				let new_borrowed_amount = position.borrowed_amount - value_to_sell;
				position_value = (new_size * price) - new_borrowed_amount;
				maintenance_requirement =
					position.avg_execution_price * new_size * req_margin_fraction;
			} else {
				position_value = (position.size * price) - position.borrowed_amount;
				maintenance_requirement =
					position.avg_execution_price * position.size * req_margin_fraction;
			}

			(position_value, maintenance_requirement)
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
			let req_margin_fraction = market.maintenance_margin_fraction;

			let leveraged_position_value = execution_price * size;
			let maintenance_requirement = req_margin_fraction * leveraged_position_value;

			let (liq_result, _, available_margin, _, _) = T::TradingAccountPallet::get_margin_info(
				order.account_id,
				market.asset_collateral,
				maintenance_requirement,
				margin_amount,
			);

			let mut is_error: bool = false;
			if liq_result == true {
				is_error = true;
			} else {
				if (order.direction == Direction::Short) &&
					(order.side == Side::Buy) &&
					(order.order_type == OrderType::Limit)
				{
					let price_diff = oracle_price - execution_price;
					let pnl = price_diff * size;

					// check whether user have enough balance to cover the immediate losses.
					is_error = if available_margin <= pnl { true } else { false };
				}
			}
			return (available_margin, is_error)
		}

		fn check_for_force_closure(
			account_id: U256,
			collateral_id: u128,
			market_id: u128,
			direction: Direction,
		) {
			let (liq_result, _, _, _, _) = T::TradingAccountPallet::get_margin_info(
				account_id,
				collateral_id,
				FixedI128::zero(),
				FixedI128::zero(),
			);

			if liq_result == true {
				let (is_deleveragable, amount_to_be_sold) =
					Self::is_account_deleveragable(account_id, collateral_id, market_id, direction);
				if is_deleveragable {
					if amount_to_be_sold == FixedI128::zero() {
						return
					}
					T::TradingPallet::set_flags_for_force_orders(
						account_id,
						collateral_id,
						ForceClosureFlag::Deleverage,
						DeleveragablePosition { market_id, direction, amount_to_be_sold },
					);
				} else {
					T::TradingPallet::set_flags_for_force_orders(
						account_id,
						collateral_id,
						ForceClosureFlag::Liquidate,
						DeleveragablePosition {
							market_id,
							direction,
							amount_to_be_sold: FixedI128::zero(),
						},
					);
				}
			}
		}
	}
}
