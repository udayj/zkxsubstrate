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
	use frame_support::pallet_prelude::{ValueQuery, *};
	use frame_system::pallet_prelude::*;
	use primitive_types::U256;
	use sp_arithmetic::{fixed_point::FixedI128, FixedPointNumber};
	use zkx_support::traits::{MarketInterface, TradingAccountInterface};
	use zkx_support::types::{
		Direction, ErrorEventList, Market, Order, OrderEventList, OrderType, Position, Side,
		TimeInForce,
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
	pub(super) type BatchStatusMap<T: Config> = StorageMap<_, Twox64Concat, U256, bool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn portion_executed)]
	pub(super) type PortionExecutedMap<T: Config> =
		StorageMap<_, Twox64Concat, u128, FixedI128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn positions)]
	pub(super) type PositionsMap<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		U256, // account_id
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
		U256, // account_id
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
		U256, // account_id
		Blake2_128Concat,
		u64,  // index
		U256, // market id
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn open_interest)]
	pub(super) type OpenInterestMap<T: Config> =
		StorageMap<_, Twox64Concat, U256, FixedI128, ValueQuery>;

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
		/// Maker order skipped since quantity_executed = quantity_locked for the batch
		MakerOrderSkipped,
		/// Order is fully executed
		OrderFullyExecuted,
		/// Order is trying to close an empty position
		ClosingEmptyPosition,
		/// Maker side or direction does not match with other makers
		InvalidMakerDirectionSide,
		/// Maker order can only be limit order
		InvalidMakerOrderType,
		/// Taker side or direction is invalid wrt to makers
		InvalidTakerDirectionSide,
		/// Taker order is post only
		InvalidTakerPostOnly,
		/// Execution price is not valid wrt limit price for long sell or short buy
		LimitPriceErrorLongSell,
		/// Execution price is not valid wrt limit price for long buy or short sell
		LimitPriceErrorLongBuy,
		/// Price is not within slippage limit
		SlippageError,
		/// FOK orders should be filled completely
		FOKError,
		/// Not enough margin to cover losses - short limit sell or long limit sell
		NotEnoughMargin,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Trade executed successfully
		TradeExecuted { batch_id: u64 },
		/// Order error
		OrderError { order_id: u128, error_code: u16 },
		/// Order of a user executed successfully
		OrderExecuted {
			user: U256,
			order_id: u128,
			market_id: U256,
			size: FixedI128,
			direction: Direction,
			side: Side,
			order_type: OrderType,
			execution_price: FixedI128,
		},
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

			let taker_order = &orders[orders.len() - 1];
			let initial_taker_locked_response = Self::calculate_initial_taker_locked_size(
				taker_order,
				quantity_locked,
				market_id,
				collateral_id,
			);
			match initial_taker_locked_response {
				Ok(quantity) => initial_taker_locked_quantity = quantity,
				Err(e) => match e {
					_ => return Err(DispatchError::Other("ExecutableQuantityZero")), // Double check this
				},
			}

			let mut quantity_executed: FixedI128 = 0.into();
			let mut total_order_volume: FixedI128 = 0.into();
			let mut updated_position: Position;
			let mut open_interest: FixedI128 = 0.into();

			let mut error_events: Vec<ErrorEventList> = Vec::new();
			let mut order_events: Vec<OrderEventList> = Vec::new();

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

				let validation_response = Self::perform_validations(element, oracle_price, &market);
				match validation_response {
					Ok(()) => (),
					Err(e) => {
						error_events.push(ErrorEventList {
							order_id: element.order_id,
							error_code: Self::get_error_code(e),
						});
						continue;
					},
				}

				let order_portion_executed = PortionExecutedMap::<T>::get(element.order_id);
				let direction = if element.direction == Direction::Long { LONG } else { SHORT };
				let position_details =
					PositionsMap::<T>::get(&element.user, [market_id, direction]);
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
						Err(e) => {
							error_events.push(ErrorEventList {
								order_id: element.order_id,
								error_code: Self::get_error_code(e),
							});
							continue;
						},
					}
					// Calculate quantity left to be executed
					let quantity_remaining = initial_taker_locked_quantity - quantity_executed;
					// Calculate quantity that needs to be executed for the current maker
					let maker_quantity_to_execute_response = Self::calculate_quantity_to_execute(
						order_portion_executed,
						market_id,
						&position_details,
						element,
						quantity_remaining,
					);
					match maker_quantity_to_execute_response {
						Ok(quantity) => quantity_to_execute = quantity,
						Err(e) => {
							error_events.push(ErrorEventList {
								order_id: element.order_id,
								error_code: Self::get_error_code(e),
							});
							continue;
						},
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
						Err(e) => {
							error_events.push(ErrorEventList {
								order_id: element.order_id,
								error_code: Self::get_error_code(e),
							});
							continue;
						},
					}

					// Taker quantity to be executed will be sum of maker quantities executed
					quantity_to_execute = quantity_executed;
					ensure!(quantity_to_execute > 0.into(), Error::<T>::MakerOrderSkipped);

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
							Err(e) => {
								error_events.push(ErrorEventList {
									order_id: element.order_id,
									error_code: Self::get_error_code(e),
								});
								continue;
							},
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
							Err(e) => {
								error_events.push(ErrorEventList {
									order_id: element.order_id,
									error_code: Self::get_error_code(e),
								});
								continue;
							},
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
						Err(e) => {
							error_events.push(ErrorEventList {
								order_id: element.order_id,
								error_code: Self::get_error_code(e),
							});
							continue;
						},
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
							PositionsMap::<T>::get(&element.user, [market_id, opposite_direction]);
						if opposite_position.size == 0.into() {
							let length =
								CollateralToMarketLengthMap::<T>::get(&element.user, collateral_id);
							CollateralToMarketMap::<T>::insert(&element.user, length, market_id);
							CollateralToMarketLengthMap::<T>::insert(
								&element.user,
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
					let response = Self::process_close_orders(
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
						Err(e) => {
							error_events.push(ErrorEventList {
								order_id: element.order_id,
								error_code: Self::get_error_code(e),
							});
							continue;
						},
					}

					new_position_size = position_details.size - quantity_to_execute;

					// To do - handle liquidation/deleveraging order

					new_leverage = position_details.leverage;
					new_margin_locked = current_margin_locked - margin_lock_amount;

					// To do - handle the case when liquidatable position is present
					// if amount to be sold is 0, do nothing
					// else check whether current market and direction is liquidatable position and update

					// If the user does not have any position in this market
					// hen remove the market from CollateralToMarketMap
					if new_position_size == 0.into() {
						let opposite_direction =
							if element.direction == Direction::Long { SHORT } else { LONG };
						let opposite_position =
							PositionsMap::<T>::get(&element.user, [market_id, opposite_direction]);
						if opposite_position.size == 0.into() {
							let length =
								CollateralToMarketLengthMap::<T>::get(&element.user, collateral_id);
							CollateralToMarketMap::<T>::remove(&element.user, length);
							CollateralToMarketLengthMap::<T>::insert(
								&element.user,
								collateral_id,
								length - 1_u64,
							);
						}
						updated_position = Position {
							avg_execution_price: 0.into(),
							size: 0.into(),
							margin_amount: 0.into(),
							borrowed_amount: 0.into(),
							leverage: 0.into(),
						};
					} else {
						// To do - Calculate pnl

						updated_position = Position {
							avg_execution_price,
							size: new_position_size,
							margin_amount,
							borrowed_amount,
							leverage: new_leverage,
						};
					}

					let is_final: bool;
					if element.time_in_force == TimeInForce::IOC {
						new_portion_executed = element.size;
						is_final = true;
					} else {
						if new_portion_executed == element.size {
							is_final = true;
						} else {
							if new_position_size == 0.into() {
								is_final = true;
							} else {
								is_final = false;
							}
						}
					}

					open_interest = open_interest - quantity_to_execute;
				}

				// Update position, locked margin and portion executed
				let direction = if element.direction == Direction::Long { LONG } else { SHORT };
				PositionsMap::<T>::set(&element.user, [market_id, direction], updated_position);
				T::TradingAccountPallet::set_locked_margin(
					element.user,
					collateral_id,
					new_margin_locked,
				);
				PortionExecutedMap::<T>::insert(element.order_id, new_portion_executed);

				order_events.push(OrderEventList {
					user: element.user,
					order_id: element.order_id,
					market_id: element.market_id,
					size: quantity_to_execute,
					direction: element.direction,
					side: element.side,
					order_type: element.order_type,
					execution_price,
				})
			}

			// Update open interest
			let actual_open_interest = open_interest / 2.into();
			let current_open_interest = OpenInterestMap::<T>::get(market_id);
			OpenInterestMap::<T>::insert(market_id, current_open_interest + actual_open_interest);

			BatchStatusMap::<T>::insert(batch_id, true);

			for element in &error_events {
				Self::deposit_event(Event::OrderError {
					order_id: element.order_id,
					error_code: element.error_code,
				});
			}

			for element in &order_events {
				Self::deposit_event(Event::OrderExecuted {
					user: element.user,
					order_id: element.order_id,
					market_id: element.market_id,
					size: element.size,
					direction: element.direction,
					side: element.side,
					order_type: element.order_type,
					execution_price: element.execution_price,
				});
			}

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn calculate_initial_taker_locked_size(
			order: &Order,
			quantity_locked: FixedI128,
			market_id: U256,
			collateral_id: U256,
		) -> Result<FixedI128, Error<T>> {
			let LONG: U256 = U256::from(1_u8);
			let SHORT: U256 = U256::from(2_u8);

			let order_portion_executed = PortionExecutedMap::<T>::get(order.order_id);

			let direction = if order.direction == Direction::Long { LONG } else { SHORT };
			let position_details = PositionsMap::<T>::get(&order.user, [market_id, direction]);

			let quantity_response = Self::calculate_quantity_to_execute(
				order_portion_executed,
				market_id,
				&position_details,
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
			position_details: &Position,
			order: &Order,
			quantity_remaining: FixedI128,
		) -> Result<FixedI128, Error<T>> {
			let executable_quantity = order.size - portion_executed;
			ensure!(executable_quantity > 0.into(), Error::<T>::MakerOrderSkipped); // Modify code with tick/step size

			let quantity_to_execute = FixedI128::min(executable_quantity, quantity_remaining);
			ensure!(quantity_to_execute > 0.into(), Error::<T>::OrderFullyExecuted);

			if order.side == Side::Buy {
				Ok(quantity_to_execute)
			} else {
				// To Do - handle Liquidation/Deleveraging scenario

				let quantity_to_execute = quantity_to_execute - position_details.size;
				ensure!(quantity_to_execute > 0.into(), Error::<T>::ClosingEmptyPosition);

				Ok(quantity_to_execute)
			}
		}

		fn perform_validations(
			order: &Order,
			oracle_price: FixedI128,
			market: &Market,
		) -> Result<(), Error<T>> {
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
		) -> Result<(), Error<T>> {
			let opposite_direction = if maker1_direction == Direction::Long {
				Direction::Short
			} else {
				Direction::Long
			};
			let opposite_side = if maker1_side == Side::Buy { Side::Sell } else { Side::Buy };

			ensure!(
				(current_direction == maker1_direction && current_side == maker1_side)
					|| (current_direction == opposite_direction && current_side == opposite_side),
				Error::<T>::InvalidMakerDirectionSide
			);

			ensure!(order_type == OrderType::Limit, Error::<T>::InvalidMakerOrderType);

			Ok(())
		}

		fn validate_taker(
			maker1_direction: Direction,
			maker1_side: Side,
			current_direction: Direction,
			current_side: Side,
			post_only: bool,
		) -> Result<(), Error<T>> {
			let opposite_direction = if maker1_direction == Direction::Long {
				Direction::Short
			} else {
				Direction::Long
			};
			let opposite_side = if maker1_side == Side::Buy { Side::Sell } else { Side::Buy };

			ensure!(
				(current_direction == maker1_direction && current_side == opposite_side)
					|| (current_direction == opposite_direction && current_side == maker1_side),
				Error::<T>::InvalidTakerDirectionSide
			);

			// Taker order cannot be post only order
			ensure!(post_only == false, Error::<T>::InvalidTakerPostOnly);

			Ok(())
		}

		fn validate_limit_price(
			price: FixedI128,
			execution_price: FixedI128,
			direction: Direction,
			side: Side,
		) -> Result<(), Error<T>> {
			if (direction == Direction::Long && side == Side::Buy)
				|| (direction == Direction::Short && side == Side::Sell)
			{
				ensure!(execution_price <= price, Error::<T>::LimitPriceErrorLongBuy);
			} else {
				ensure!(price <= execution_price, Error::<T>::LimitPriceErrorLongSell);
			}

			Ok(())
		}

		fn validate_within_slippage(
			slippage: FixedI128,
			oracle_price: FixedI128,
			execution_price: FixedI128,
			direction: Direction,
			side: Side,
		) -> Result<(), Error<T>> {
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
		) -> Result<(FixedI128, FixedI128, FixedI128, FixedI128, FixedI128), Error<T>> {
			let LONG: U256 = U256::from(1_u8);
			let SHORT: U256 = U256::from(2_u8);
			let mut margin_amount: FixedI128 = 0.into();
			let mut borrowed_amount: FixedI128 = 0.into();
			let mut average_execution_price: FixedI128 = execution_price;

			// To do - get fee rate and calculate fee

			let direction = if order.direction == Direction::Long { LONG } else { SHORT };
			let position_details = PositionsMap::<T>::get(&order.user, [market_id, direction]);

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

		fn process_close_orders(
			order: &Order,
			order_size: FixedI128,
			execution_price: FixedI128,
			market_id: U256,
			collateral_id: U256,
		) -> Result<(FixedI128, FixedI128, FixedI128, FixedI128, FixedI128), Error<T>> {
			let LONG: U256 = U256::from(1_u8);
			let SHORT: U256 = U256::from(2_u8);
			let actual_execution_price: FixedI128;
			let price_diff: FixedI128;

			let direction = if order.direction == Direction::Long { LONG } else { SHORT };
			let position_details = PositionsMap::<T>::get(&order.user, [market_id, direction]);

			if order.direction == Direction::Long {
				actual_execution_price = execution_price;
				price_diff = execution_price - position_details.avg_execution_price;
			} else {
				price_diff = position_details.avg_execution_price - execution_price;
				actual_execution_price = position_details.avg_execution_price + price_diff;
			}

			// Total value of asset at current price
			let leveraged_order_value = order_size * actual_execution_price;

			// Calculate amount that needs to be returned to liquidity fund
			let ratio_of_position = order_size / position_details.size;
			let borrowed_amount_to_return = position_details.borrowed_amount * ratio_of_position;
			let margin_amount_to_reduce = position_details.margin_amount * ratio_of_position;

			// Calculate pnl
			let pnl = order_size * price_diff;
			let margin_plus_pnl = margin_amount_to_reduce + pnl;

			// To do - handle deleveraging order

			let borrowed_amount = position_details.borrowed_amount - borrowed_amount_to_return;
			let margin_amount = position_details.margin_amount - margin_amount_to_reduce;

			// To do - deduct fund from holding contract
			// To do - deposit fund to liquidity fund if position is leveraged

			let balance = T::TradingAccountPallet::get_balance(order.user, collateral_id);

			// Check if user is under water, ie,
			// user has lost some borrowed funds
			if margin_plus_pnl.is_negative() {
				let amount_to_transfer_from = margin_plus_pnl.saturating_abs();

				// Check if user's balance can cover the deficit
				if amount_to_transfer_from > balance {
					if order.order_type == OrderType::Limit {
						ensure!(false, Error::<T>::NotEnoughMargin);
					}

					if balance.is_negative() {
						// To do - withdraw amount_to_transfer_from from insurance fund
					} else {
						// To do - withdraw (amount_to_transfer_from - balance) from insurance fund
					}
				}

				// If user's position value has become negative
				// it's a deficit for holding contract
				if leveraged_order_value.is_negative() {
					// To do - deposit abs(leveraged_order_value) to holding
				}

				// Deduct under water amount (if any) + margin amt to reduce from user
				T::TradingAccountPallet::transfer_from(
					order.user,
					collateral_id,
					amount_to_transfer_from + margin_amount_to_reduce,
				);
			// To do - calculate realized pnl
			} else {
				// User is not under water
				// User is in loss
				if pnl.is_negative() {
					// Loss cannot be covered by the user
					if pnl.saturating_abs() > balance {
						// If balance is negative, deduct whole loss from insurance fund
						if balance.is_negative() {
							// To do - deduct abs(pnl) from insurance fund
						} else {
							// To do - deduct (abs(pnl) - balance) from insurance fund
						}
					}

					// Deduct required funds from user
					T::TradingAccountPallet::transfer_from(
						order.user,
						collateral_id,
						pnl.saturating_abs(),
					);
				} else {
					// User is in profit
					// Transfer the profit to user
					T::TradingAccountPallet::transfer(order.user, collateral_id, pnl);
				}

				// To do - Handle liquidation and deleveraging orders

				// Deduct  proportionate margin amount from user
				T::TradingAccountPallet::transfer_from(
					order.user,
					collateral_id,
					margin_amount_to_reduce,
				);
			}

			Ok((
				margin_amount,
				borrowed_amount,
				position_details.avg_execution_price,
				balance,
				margin_amount_to_reduce,
			))
		}

		fn get_error_code(error: Error<T>) -> u16 {
			match error {
				Error::<T>::InsufficientBalance => 501,
				Error::<T>::InvalidLeverage => 502,
				Error::<T>::MarketMismatch => 504,
				Error::<T>::SizeTooSmall => 505,
				Error::<T>::SlippageError => 506,
				Error::<T>::LimitPriceErrorLongSell => 507,
				Error::<T>::LimitPriceErrorLongBuy => 508,
				Error::<T>::UserNotRegistered => 510,
				Error::<T>::InvalidTakerDirectionSide => 511,
				Error::<T>::InvalidMakerDirectionSide => 512,
				Error::<T>::InvalidTakerPostOnly => 515,
				Error::<T>::InvalidMakerOrderType => 518,
				Error::<T>::OrderFullyExecuted => 523,
				Error::<T>::ClosingEmptyPosition => 524,
				Error::<T>::NotEnoughMargin => 532,
				Error::<T>::MakerOrderSkipped => 533,
				_ => 500,
			}
		}
	}
}
//
