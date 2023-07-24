#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use core::option::Option;
	use frame_support::inherent::Vec;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_arithmetic::{fixed_point::FixedI128, FixedPointNumber};
	use zkx_support::traits::{MarketInterface, TradingAccountInterface};
	use zkx_support::types::{Direction, Market, Order, Position, Side, TradingAccount};

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type MarketPallet: MarketInterface;
		type TradingAccountPallet: TradingAccountInterface;
	}

	#[pallet::storage]
	#[pallet::getter(fn positions)]
	pub(super) type PositionsMap<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		TradingAccount,
		Blake2_128Concat,
		[u64; 2],
		Position,
		ValueQuery,
	>;

	#[pallet::error]
	pub enum Error<T> {
		/// Invalid input for market
		MarketNotFound,
		/// Market not tradable
		MarketNotTradable,
		/// Quantity locked cannot be 0
		QuantityLockedError,
		/// Balance not enough to open the position
		InsufficientBalance,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Trade executed successfully
		TradeExecuted { id: u64 },
	}

	// Pallet callable functions
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// External function to be called for trade execution
		#[pallet::weight(0)]
		pub fn execute_trade(
			origin: OriginFor<T>,
			batch_id: u128,
			quantity_locked: FixedI128,
			market_id: u64,
			oracle_price: FixedI128,
			orders: Vec<Order>,
		) -> DispatchResult {
			// Make sure the caller is from a signed origin
			let sender = ensure_signed(origin)?;

			// Validate market
			let market = T::MarketPallet::get_market(market_id);
			ensure!(&market.is_some(), Error::<T>::MarketNotFound);
			let market = market.unwrap();
			ensure!(market.is_tradable == 1_u8, Error::<T>::MarketNotTradable);

			let collateral_id: u64 = market.asset_collateral;

			ensure!(
				quantity_locked != FixedI128::checked_from_integer(0).unwrap(),
				Error::<T>::QuantityLockedError
			);

			let mut margin_amount: FixedI128 = 0.into();
			let mut borrowed_amount: FixedI128 = 0.into();
			let mut avg_execution_price: FixedI128 = 0.into();

			for element in orders {
				if element.side == Side::Buy {
					(margin_amount, borrowed_amount, avg_execution_price) =
						Self::process_open_orders(
							element.clone(),
							FixedI128::checked_from_integer(10000).unwrap(),
							collateral_id,
						);
				} else {
					// to do
				}

				let position = Position {
					avg_execution_price,
					size: element.size,
					margin_amount,
					borrowed_amount,
					leverage: FixedI128::checked_from_integer(1).unwrap(),
				};
				let direction = if element.direction == Direction::Long { 1_u64 } else { 2_u64 };
				PositionsMap::<T>::set(element.user, [market_id, direction], position);
			}

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn process_open_orders(
			order: Order,
			execution_price: FixedI128,
			collateral_id: u64,
		) -> (FixedI128, FixedI128, FixedI128) {
			let mut margin_amount: FixedI128 = 0.into();
			let mut borrowed_amount: FixedI128 = 0.into();
			let mut average_execution_price: FixedI128 = execution_price;

			let order_value = order.size.mul(execution_price);
			margin_amount = order_value;

			let balance = T::TradingAccountPallet::get_balance(order.user.clone(), collateral_id);
			// Do error handling for balance check
			// ensure!(order_value <= balance, Error::<T>::InsufficientBalance);
			T::TradingAccountPallet::transfer_from(order.user, collateral_id, order_value);

			(margin_amount, borrowed_amount, average_execution_price)
		}
	}
}
