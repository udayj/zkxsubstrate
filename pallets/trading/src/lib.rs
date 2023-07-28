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
	use primitive_types::U256;
	use sp_arithmetic::{fixed_point::FixedI128, FixedPointNumber};
	use zkx_support::traits::{MarketInterface, TradingAccountInterface};
	use zkx_support::types::{
		Direction, Market, Order, OrderType, Position, Side, TimeInForce, TradingAccount,
	};

	static LEVERAGE_ONE: FixedI128 = FixedI128::from_inner(1000000000000000000);

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
	#[pallet::getter(fn batch_status)]
	pub(super) type BatchStatusMap<T: Config> = StorageMap<_, Twox64Concat, U256, bool>;

	#[pallet::storage]
	#[pallet::getter(fn portion_executed)]
	pub(super) type PortionExecutedMap<T: Config> = StorageMap<_, Twox64Concat, u128, FixedI128>;

	#[pallet::storage]
	#[pallet::getter(fn positions)]
	pub(super) type PositionsMap<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		TradingAccount,
		Blake2_128Concat,
		[U256; 2],
		Position,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn collateral_to_market_length)]
	pub(super) type CollateralToMarketLengthMap<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		TradingAccount,
		Blake2_128Concat,
		U256, // collateral id
		u64,  // number of markets
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn collateral_to_market)]
	pub(super) type CollateralToMarketMap<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		TradingAccount,
		Blake2_128Concat,
		u64,  // index
		U256, // market id
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn open_interest)]
	pub(super) type OpenInterestMap<T: Config> = StorageMap<_, Twox64Concat, U256, FixedI128>;

	#[pallet::error]
	pub enum Error<T> {
		/// Batch with same ID already execute
		BatchAlreadyExecuted,
		/// Invalid input for market
		MarketNotFound,
		/// Market not tradable
		MarketNotTradable,
		/// Quantity locked cannot be 0
		QuantityLockedError,
		/// Balance not enough to open the position
		InsufficientBalance,
		/// User's account is not registered
		UserNotRegistered,
		/// Order size less than min quantity
		SizeTooSmall,
		/// Market matched and order market are different
		MarketMismatch,
		/// Invalid value for leverage (less than min or greater than currently allowed leverage)
		InvalidLeverage,
		/// Quantity to execute is 0 since order is completely executed
		ExecutableQuantityZero,
		/// Maker side or direction does not match with other makers
		InvalidMaker,
		/// Taker side or direction is invalid wrt to makers, or taker order is post only
		InvalidTaker,
		/// Execution price is not valid wrt limit price
		LimitPriceError,
		/// Price is not within slippage limit
		SlippageError,
		/// FOK orders should be filled completely
		FOKError,
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
			batch_id: U256,
			quantity_locked: FixedI128,
			market_id: U256,
			oracle_price: FixedI128,
			orders: Vec<Order>,
		) -> DispatchResult {
			// Make sure the caller is from a signed origin
			let sender = ensure_signed(origin)?;

			let LONG: U256 = U256::from(1_u8);
			let SHORT: U256 = U256::from(2_u8);

			ensure!(!BatchStatusMap::<T>::contains_key(batch_id), Error::<T>::BatchAlreadyExecuted);

			// Validate market
			let market = T::MarketPallet::get_market(market_id);
			ensure!(market.is_some(), Error::<T>::MarketNotFound);
			let market = market.unwrap();
			ensure!(market.is_tradable == 1_u8, Error::<T>::MarketNotTradable);

			let collateral_id: U256 = market.asset_collateral;
			let initial_taker_locked_quantity: FixedI128;

			ensure!(
				quantity_locked != FixedI128::checked_from_integer(0).unwrap(),
				Error::<T>::QuantityLockedError
			);

			let taker_order = orders[orders.len() - 1];
			let initial_taker_locked_response = Self::calculate_initial_taker_locked_size(
				taker_order,
				quantity_locked,
				market_id,
				collateral_id,
			);
			match initial_taker_locked_response {
				Ok(quantity) => initial_taker_locked_quantity = quantity,
				Err(e) => return Err(e),
			}

			let mut quantity_executed: FixedI128 = 0.into();
			let mut total_order_volume: FixedI128 = 0.into();
			let mut updated_position: Position;
			let mut open_interest: FixedI128 = 0.into();

			for element in &orders {
				let mut margin_amount: FixedI128 = 0.into(); // To do - don't assign value
				let mut borrowed_amount: FixedI128 = 0.into(); // To do - don't assign value
				let mut avg_execution_price: FixedI128 = 0.into(); // To do - don't assign value
				let mut execution_price: FixedI128 = 0.into(); // To do - don't assign value
				let mut quantity_to_execute: FixedI128 = 0.into(); // To do - don't assign value
				let mut user_available_balance: FixedI128 = 0.into(); // To do - don't assign value
				let mut margin_lock_amount: FixedI128 = 0.into(); // To do - don't assign value
				let mut new_position_size: FixedI128 = 0.into(); // To do - don't assign value
				let mut new_leverage: FixedI128 = 0.into(); // To do - don't assign value
				let mut new_margin_locked: FixedI128 = 0.into(); // To do - don't assign value
				let mut new_portion_executed: FixedI128 = 0.into(); // To do - don't assign value

				let validation_response = Self::perform_validations(element, oracle_price, market);
				match validation_response {
					Ok(()) => (),
					Err(e) => return Err(e),
				}

				let order_portion_executed =
					PortionExecutedMap::<T>::get(element.order_id).unwrap();
				let direction = if element.direction == Direction::Long { LONG } else { SHORT };
				let position_details = PositionsMap::<T>::get(element.user, [market_id, direction]);
				let current_margin_locked =
					T::TradingAccountPallet::get_locked_margin(element.user, collateral_id);

				// Maker Order
				if element.order_id != orders[orders.len() - 1].order_id {
					let validation_response = Self::validate_maker(
						orders[0].direction,
						orders[0].side,
						element.direction,
						element.side,
						element.order_type,
					);
					match validation_response {
						Ok(()) => (),
						Err(e) => return Err(e),
					}
					// Calculate quantity left to be executed
					let quantity_remaining = initial_taker_locked_quantity - quantity_executed;
					// Calculate quantity that needs to be executed for the current maker
					let maker_quantity_to_execute_response = Self::calculate_quantity_to_execute(
						order_portion_executed,
						market_id,
						position_details,
						element,
						quantity_remaining,
					);
					match maker_quantity_to_execute_response {
						Ok(quantity) => quantity_to_execute = quantity,
						Err(e) => return Err(e),
					}

					// For a maker execution price will always be the price in its order object
					execution_price = element.price;

					quantity_executed = quantity_executed + quantity_to_execute;
					total_order_volume = total_order_volume + (element.price * quantity_to_execute);
				} else {
					// Taker Order
					let validation_response = Self::validate_taker(
						orders[0].direction,
						orders[0].side,
						element.direction,
						element.side,
						element.post_only,
					);
					match validation_response {
						Ok(()) => (),
						Err(e) => return Err(e),
					}

					// Taker quantity to be executed will be sum of maker quantities executed
					quantity_to_execute = quantity_executed;
					ensure!(quantity_to_execute > 0.into(), Error::<T>::ExecutableQuantityZero);

					// Handle FoK order
					if element.time_in_force == TimeInForce::FOK {
						ensure!(quantity_to_execute == element.size, Error::<T>::FOKError);
					}

					// Calculate execution price for taker
					execution_price = total_order_volume / quantity_to_execute;

					// Validate execution price of taker
					if element.order_type == OrderType::Limit {
						let limit_validation = Self::validate_limit_price(
							element.price,
							execution_price,
							element.direction,
							element.side,
						);
						match limit_validation {
							Ok(()) => (),
							Err(e) => return Err(e),
						}
					} else {
						let slippage_validation = Self::validate_within_slippage(
							element.slippage,
							oracle_price,
							execution_price,
							element.direction,
							element.side,
						);
						match slippage_validation {
							Ok(()) => (),
							Err(e) => return Err(e),
						}
					}
				}

				new_portion_executed = order_portion_executed + quantity_to_execute;

				// BUY order
				if element.side == Side::Buy {
					let response = Self::process_open_orders(
						element,
						quantity_to_execute,
						execution_price,
						market_id,
						collateral_id,
					);
					match response {
						Ok((margin, borrowed, average_execution, balance, margin_lock)) => {
							margin_amount = margin;
							borrowed_amount = borrowed;
							avg_execution_price = average_execution;
							user_available_balance = balance;
							margin_lock_amount = margin_lock;
						},
						Err(e) => return Err(e),
					}

					new_position_size = quantity_to_execute + position_details.size;
					new_leverage = (margin_amount + borrowed_amount) / margin_amount;
					new_margin_locked = current_margin_locked + margin_lock_amount;

					// If the user previously does not have any position in this market
					// then add the market to CollateralToMarketMap
					if position_details.size == 0.into() {
						let opposite_direction =
							if element.direction == Direction::Long { SHORT } else { LONG };
						let opposite_position =
							PositionsMap::<T>::get(element.user, [market_id, opposite_direction]);
						if opposite_position.size == 0.into() {
							let length =
								CollateralToMarketLengthMap::<T>::get(element.user, collateral_id);
							CollateralToMarketMap::<T>::insert(element.user, length, market_id);
							CollateralToMarketLengthMap::<T>::insert(
								element.user,
								collateral_id,
								length + 1_u64,
							);
						}
					}

					updated_position = Position {
						avg_execution_price,
						size: new_position_size,
						margin_amount,
						borrowed_amount,
						leverage: new_leverage,
					};

					open_interest = open_interest + quantity_to_execute;
				} else {
					// SELL order
					updated_position = Position {
						avg_execution_price: 0.into(),
						size: 0.into(),
						margin_amount: 0.into(),
						borrowed_amount: 0.into(),
						leverage: 0.into(),
					};
				}

				// Update position, locked margin and portion executed
				let direction = if element.direction == Direction::Long { LONG } else { SHORT };
				PositionsMap::<T>::set(element.user, [market_id, direction], updated_position);
				T::TradingAccountPallet::set_locked_margin(
					element.user,
					collateral_id,
					new_margin_locked,
				);
				PortionExecutedMap::<T>::insert(element.order_id, new_portion_executed);
			}

			// Update open interest
			let actual_open_interest = open_interest / 2.into();
			let current_open_interest = OpenInterestMap::<T>::get(market_id).unwrap();
			OpenInterestMap::<T>::insert(market_id, current_open_interest + actual_open_interest);

			BatchStatusMap::<T>::insert(batch_id, true);

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn calculate_initial_taker_locked_size(
			order: Order,
			quantity_locked: FixedI128,
			market_id: U256,
			collateral_id: U256,
		) -> Result<FixedI128, DispatchError> {
			let LONG: U256 = U256::from(1_u8);
			let SHORT: U256 = U256::from(2_u8);

			let order_portion_executed = PortionExecutedMap::<T>::get(order.order_id).unwrap();

			let direction = if order.direction == Direction::Long { LONG } else { SHORT };
			let position_details = PositionsMap::<T>::get(order.user, [market_id, direction]);

			let quantity_response = Self::calculate_quantity_to_execute(
				order_portion_executed,
				market_id,
				position_details,
				&order,
				quantity_locked,
			);
			match quantity_response {
				Ok(quantity) => Ok(quantity),
				Err(e) => Err(e),
			}
		}

		fn calculate_quantity_to_execute(
			portion_executed: FixedI128,
			market_id: U256,
			position_details: Position,
			order: &Order,
			quantity_remaining: FixedI128,
		) -> Result<FixedI128, DispatchError> {
			let executable_quantity = order.size - portion_executed;
			ensure!(executable_quantity > 0.into(), Error::<T>::ExecutableQuantityZero); // Modify code with tick/step size

			let quantity_to_execute = FixedI128::min(executable_quantity, quantity_remaining);
			ensure!(quantity_to_execute > 0.into(), Error::<T>::ExecutableQuantityZero);

			if order.side == Side::Buy {
				Ok(quantity_to_execute)
			} else {
				// To Do - handle SELL case
				Ok(quantity_to_execute) // This is just a placeholder
			}
		}

		fn perform_validations(
			order: &Order,
			oracle_price: FixedI128,
			market: Market,
		) -> Result<(), DispatchError> {
			// Validate that the user is registered
			let is_registered = T::TradingAccountPallet::is_registered_user(order.user);
			ensure!(is_registered, Error::<T>::UserNotRegistered);

			// Validate that size of order is >= min quantity for market
			ensure!(order.size >= market.minimum_order_size, Error::<T>::SizeTooSmall);

			// Validate that market matched and market in order are same
			ensure!(market.id == order.market_id, Error::<T>::MarketMismatch);

			// Validate leverage value
			ensure!(
				order.leverage >= LEVERAGE_ONE
					&& order.leverage <= market.currently_allowed_leverage,
				Error::<T>::InvalidLeverage
			);

			Ok(())
		}

		fn validate_maker(
			maker1_direction: Direction,
			maker1_side: Side,
			current_direction: Direction,
			current_side: Side,
			order_type: OrderType,
		) -> Result<(), DispatchError> {
			let opposite_direction = if maker1_direction == Direction::Long {
				Direction::Short
			} else {
				Direction::Long
			};
			let opposite_side = if maker1_side == Side::Buy { Side::Sell } else { Side::Buy };

			ensure!(
				!(current_direction == maker1_direction && current_side == maker1_side),
				Error::<T>::InvalidMaker
			);
			ensure!(
				!(current_direction == opposite_direction && current_side == opposite_side),
				Error::<T>::InvalidMaker
			);
			ensure!(order_type == OrderType::Limit, Error::<T>::InvalidMaker);

			Ok(())
		}

		fn validate_taker(
			maker1_direction: Direction,
			maker1_side: Side,
			current_direction: Direction,
			current_side: Side,
			post_only: bool,
		) -> Result<(), DispatchError> {
			let opposite_direction = if maker1_direction == Direction::Long {
				Direction::Short
			} else {
				Direction::Long
			};
			let opposite_side = if maker1_side == Side::Buy { Side::Sell } else { Side::Buy };

			ensure!(
				!(current_direction == maker1_direction && current_side == opposite_side),
				Error::<T>::InvalidTaker
			);
			ensure!(
				!(current_direction == opposite_direction && current_side == maker1_side),
				Error::<T>::InvalidTaker
			);

			// Taker order cannot be post only order
			ensure!(post_only == false, Error::<T>::InvalidTaker);

			Ok(())
		}

		fn validate_limit_price(
			price: FixedI128,
			execution_price: FixedI128,
			direction: Direction,
			side: Side,
		) -> Result<(), DispatchError> {
			if (direction == Direction::Long && side == Side::Buy)
				|| (direction == Direction::Short && side == Side::Sell)
			{
				ensure!(execution_price <= price, Error::<T>::LimitPriceError);
			} else {
				ensure!(price <= execution_price, Error::<T>::LimitPriceError);
			}

			Ok(())
		}

		fn validate_within_slippage(
			slippage: FixedI128,
			oracle_price: FixedI128,
			execution_price: FixedI128,
			direction: Direction,
			side: Side,
		) -> Result<(), DispatchError> {
			let threshold = slippage * oracle_price;

			if (direction == Direction::Long && side == Side::Buy)
				|| (direction == Direction::Short && side == Side::Sell)
			{
				ensure!(execution_price <= (oracle_price + threshold), Error::<T>::SlippageError);
			} else {
				ensure!((oracle_price - threshold) <= execution_price, Error::<T>::SlippageError);
			}

			Ok(())
		}

		fn process_open_orders(
			order: &Order,
			order_size: FixedI128,
			execution_price: FixedI128,
			market_id: U256,
			collateral_id: U256,
		) -> Result<(FixedI128, FixedI128, FixedI128, FixedI128, FixedI128), DispatchError> {
			let LONG: U256 = U256::from(1_u8);
			let SHORT: U256 = U256::from(2_u8);
			let mut margin_amount: FixedI128 = 0.into();
			let mut borrowed_amount: FixedI128 = 0.into();
			let mut average_execution_price: FixedI128 = execution_price;

			// To do - get fee rate and calculate fee

			let direction = if order.direction == Direction::Long { LONG } else { SHORT };
			let position_details = PositionsMap::<T>::get(order.user, [market_id, direction]);

			// Calculate average execution price
			if position_details.size == 0.into() {
				average_execution_price = execution_price;
			} else {
				let cumulative_order_value = (position_details.size
					* position_details.avg_execution_price)
					+ (order_size * execution_price);
				let cumulative_order_size = position_details.size + order_size;
				average_execution_price = cumulative_order_value / cumulative_order_size;
			}

			let leveraged_order_value = order_size * execution_price;
			let margin_order_value = leveraged_order_value / order.leverage;
			let amount_to_be_borrowed = leveraged_order_value - margin_order_value;
			margin_amount = position_details.margin_amount + margin_order_value;
			borrowed_amount = position_details.borrowed_amount + amount_to_be_borrowed;

			// To do - calculate fee

			// To do - If leveraged order, deduct from liquidity fund
			// To do - deposit to holding fund

			let balance = T::TradingAccountPallet::get_balance(order.user, collateral_id);
			ensure!(margin_order_value <= balance, Error::<T>::InsufficientBalance);
			T::TradingAccountPallet::transfer_from(order.user, collateral_id, margin_order_value);

			Ok((
				margin_amount,
				borrowed_amount,
				average_execution_price,
				balance,
				margin_order_value,
			))
		}
	}
}
//
