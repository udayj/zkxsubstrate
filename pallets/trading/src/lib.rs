#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use core::option::Option;
	use frame_support::inherent::Vec;
	use frame_support::pallet_prelude::{ValueQuery, *};
	use frame_system::pallet_prelude::*;
	use primitive_types::U256;
	use sp_arithmetic::traits::Zero;
	use sp_arithmetic::{fixed_point::FixedI128, FixedPointNumber};
	use zkx_support::helpers::sig_u256_to_sig_felt;
	use zkx_support::traits::{
		Hashable, MarketInterface, MarketPricesInterface, RiskManagementInterface,
		TradingAccountInterface, TradingFeesInterface, TradingInterface, U256Ext,
	};
	use zkx_support::types::{
		BalanceChangeReason, Direction, FundModifyType, LiquidatablePosition, Market, Order,
		OrderSide, OrderType, Position, PositionDetailsForRiskManagement, Side, TimeInForce,
		TradingAccountMinimal,
	};
	use zkx_support::{ecdsa_verify, Signature};
	static LEVERAGE_ONE: FixedI128 = FixedI128::from_inner(1000000000000000000);

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type MarketPallet: MarketInterface;
		type TradingAccountPallet: TradingAccountInterface;
		type TradingFeesPallet: TradingFeesInterface;
		type MarketPricesPallet: MarketPricesInterface;
		type RiskManagementPallet: RiskManagementInterface;
	}

	#[pallet::storage]
	#[pallet::getter(fn batch_status)]
	// k1 - batch id, v - true/false
	pub(super) type BatchStatusMap<T: Config> = StorageMap<_, Twox64Concat, U256, bool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn portion_executed)]
	// k1 - order id, v - portion executed
	pub(super) type PortionExecutedMap<T: Config> =
		StorageMap<_, Twox64Concat, u128, FixedI128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn positions)]
	// k1 - account id, k2 - (market_id, direction), v - position object
	pub(super) type PositionsMap<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		U256, // account_id
		Blake2_128Concat,
		(u128, Direction), // market_id and direction
		Position,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn collateral_to_market)]
	// k1 - account_id, k2 - collateral_id, v - vector of market ids
	pub(super) type CollateralToMarketMap<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, U256, Blake2_128Concat, u128, Vec<u128>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn open_interest)]
	// k1 - market id, v - open interest
	pub(super) type OpenInterestMap<T: Config> =
		StorageMap<_, Twox64Concat, u128, FixedI128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn initial_margin)]
	// k1 - (market_id, direction), v - initial margin locked
	pub(super) type InitialMarginMap<T: Config> =
		StorageMap<_, Twox64Concat, (u128, Direction), FixedI128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn deleveragable_or_liquidatable_position)]
	// Here, k1 - account_id,  k2 -  collateral_id, v -  LiquidatablePosition
	pub(super) type DeleveragableOrLiquidatableMap<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		U256,
		Blake2_128Concat,
		u128,
		LiquidatablePosition,
		ValueQuery,
	>;

	#[pallet::error]
	pub enum Error<T> {
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
		FOKError { error_code: u16 },
		/// Not enough margin to cover losses - short limit sell or long limit sell
		NotEnoughMargin,
		/// Order error with error code
		OrderError { error_code: u16 },
		/// Invalid oracle price
		InvalidOraclePrice { error_code: u16 },
		/// Invalid order hash - order could not be hashed into a Field Element
		InvalidOrderHash,
		/// Invalid Signature Field Elements - sig_r and/or sig_s could not be converted into a Signature
		InvalidSignatureFelt,
		/// ECDSA Signature could not be verified
		InvalidSignature,
		/// Public Key not found for account id
		NoPublicKeyFound,
		/// Invalid public key - publickey u256 could not be converted to Field Element
		InvalidPublicKey,
		/// Invalid liquidation or deleveraging market
		InvalidLiqOrDelMarket,
		/// Invalid liquidation or deleveraging market direction
		InvalidLiqOrDelDirection,
		/// Position marked to be deleveraged, but liquidation order passed
		InvalidDeleveragingOrder,
		/// Position marked to be liquidated, but deleveraging order passed
		InvalidLiquidationOrder,
		/// Revert error related to trade batch
		TradeBatchError { error_code: u16 },
		/// Position cannot be opened becuase of passive risk management
		PassiveRiskError,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Trade batch executed successfully
		TradeExecuted {
			batch_id: U256,
			market_id: u128,
			size: FixedI128,
			execution_price: FixedI128,
			direction: u8,
			side: u8,
		},
		/// Trade batch failed since no makers got executed
		TradeExecutionFailed { batch_id: U256 },
		/// Order error
		OrderError { order_id: u128, error_code: u16 },
		/// Order of a user executed successfully
		OrderExecuted {
			account_id: U256,
			order_id: u128,
			market_id: u128,
			size: FixedI128,
			direction: u8,
			side: u8,
			order_type: u8,
			execution_price: FixedI128,
			pnl: FixedI128,
			opening_fee: FixedI128,
			is_final: bool,
		},
		/// Insurance fund updation event
		InsuranceFundChange {
			collateral_id: u128,
			amount: FixedI128,
			modify_type: FundModifyType,
			block_number: T::BlockNumber,
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
			market_id: u128,
			oracle_price: FixedI128,
			orders: Vec<Order>,
		) -> DispatchResult {
			// Make sure the caller is from a signed origin
			let sender = ensure_signed(origin)?;

			ensure!(
				!BatchStatusMap::<T>::contains_key(batch_id),
				Error::<T>::TradeBatchError { error_code: 525 }
			);

			// Validate market
			let market = T::MarketPallet::get_market(market_id);
			ensure!(market.is_some(), Error::<T>::TradeBatchError { error_code: 509 });
			let market = market.unwrap();
			ensure!(market.is_tradable == true, Error::<T>::TradeBatchError { error_code: 509 });

			// validates oracle_price
			ensure!(
				oracle_price > FixedI128::zero(),
				Error::<T>::TradeBatchError { error_code: 513 }
			);

			//Update market price
			let market_price = T::MarketPricesPallet::get_market_price(market_id);
			if market_price == FixedI128::zero() {
				T::MarketPricesPallet::update_market_price(market_id, oracle_price);
			}

			let collateral_id: u128 = market.asset_collateral;
			let initial_taker_locked_quantity: FixedI128;

			ensure!(
				quantity_locked != FixedI128::checked_from_integer(0).unwrap(),
				Error::<T>::TradeBatchError { error_code: 522 }
			);

			// Calculate quantity that can be executed for the taker, before starting with the maker orders
			let taker_order = &orders[orders.len() - 1];
			let initial_taker_locked_response = Self::calculate_initial_taker_locked_size(
				taker_order,
				quantity_locked,
				market_id,
				collateral_id,
			);
			match initial_taker_locked_response {
				Ok(quantity) => initial_taker_locked_quantity = quantity,
				Err(e) => {
					let error_code = Self::get_error_code(e);
					return Err((Error::<T>::TradeBatchError { error_code }).into());
				},
			}

			let mut quantity_executed: FixedI128 = FixedI128::zero();
			let mut total_order_volume: FixedI128 = FixedI128::zero();
			let mut updated_position: Position;
			let mut open_interest: FixedI128 = FixedI128::zero();
			let mut taker_quantity: FixedI128 = FixedI128::zero();
			let mut taker_execution_price: FixedI128 = FixedI128::zero();
			let mut initial_margin_locked_long: FixedI128 =
				InitialMarginMap::<T>::get((market_id, Direction::Long));
			let mut initial_margin_locked_short: FixedI128 =
				InitialMarginMap::<T>::get((market_id, Direction::Short));

			for element in &orders {
				let margin_amount: FixedI128;
				let borrowed_amount: FixedI128;
				let avg_execution_price: FixedI128;
				let execution_price: FixedI128;
				let quantity_to_execute: FixedI128;
				let user_available_balance: FixedI128;
				let margin_lock_amount: FixedI128;
				let new_position_size: FixedI128;
				let new_leverage: FixedI128;
				let new_margin_locked: FixedI128;
				let mut new_portion_executed: FixedI128;
				let new_liquidatable_position: LiquidatablePosition;
				let realized_pnl: FixedI128;
				let new_realized_pnl: FixedI128;
				let opening_fee: FixedI128;
				let order_side: OrderSide;

				let validation_response = Self::perform_validations(element, oracle_price, &market);
				match validation_response {
					Ok(()) => (),
					Err(e) => {
						// if maker order, emit event and process next order
						if element.order_id != orders[orders.len() - 1].order_id {
							Self::deposit_event(Event::OrderError {
								order_id: element.order_id,
								error_code: Self::get_error_code(e),
							});
							continue;
						} else {
							// if taker order, revert with error
							return Err((Error::<T>::TradeBatchError {
								error_code: Self::get_error_code(e),
							})
							.into());
						}
					},
				}

				let order_portion_executed = PortionExecutedMap::<T>::get(element.order_id);
				let position_details =
					PositionsMap::<T>::get(&element.account_id, (market_id, element.direction));
				let current_margin_locked =
					T::TradingAccountPallet::get_locked_margin(element.account_id, collateral_id);

				// Get liquidatable position details
				let liq_position: LiquidatablePosition =
					DeleveragableOrLiquidatableMap::<T>::get(&element.account_id, collateral_id);

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
							Self::deposit_event(Event::OrderError {
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
						liq_position,
						quantity_remaining,
					);
					match maker_quantity_to_execute_response {
						Ok(quantity) => quantity_to_execute = quantity,
						Err(e) => {
							Self::deposit_event(Event::OrderError {
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
					order_side = OrderSide::Maker;
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
							return Err((Error::<T>::TradeBatchError {
								error_code: Self::get_error_code(e),
							})
							.into());
						},
					}

					// Taker quantity to be executed will be sum of maker quantities executed
					quantity_to_execute = quantity_executed;
					if quantity_to_execute == FixedI128::zero() {
						Self::deposit_event(Event::TradeExecutionFailed { batch_id });
						return Ok(());
					}

					// Handle FoK order
					if element.time_in_force == TimeInForce::FOK {
						if quantity_to_execute != element.size {
							return Err((Error::<T>::TradeBatchError { error_code: 516 }).into());
						}
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
								return Err((Error::<T>::TradeBatchError {
									error_code: Self::get_error_code(e),
								})
								.into());
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
								return Err((Error::<T>::TradeBatchError {
									error_code: Self::get_error_code(e),
								})
								.into());
							},
						}
					}

					order_side = OrderSide::Taker;

					taker_execution_price = execution_price;
					taker_quantity = quantity_to_execute;
				}

				new_portion_executed = order_portion_executed + quantity_to_execute;

				let mut is_final: bool = false;
				// BUY order
				if element.side == Side::Buy {
					let response = Self::process_open_orders(
						element,
						quantity_to_execute,
						order_side,
						execution_price,
						oracle_price,
						market_id,
						collateral_id,
					);
					match response {
						Ok((
							margin,
							borrowed,
							average_execution,
							balance,
							margin_lock,
							trading_fee,
						)) => {
							margin_amount = margin;
							borrowed_amount = borrowed;
							avg_execution_price = average_execution;
							user_available_balance = balance;
							margin_lock_amount = margin_lock;
							realized_pnl = trading_fee;
						},
						Err(e) => {
							// if maker order, emit event and process next order
							if element.order_id != orders[orders.len() - 1].order_id {
								Self::deposit_event(Event::OrderError {
									order_id: element.order_id,
									error_code: Self::get_error_code(e),
								});
								continue;
							} else {
								// if taker order, revert with error code
								return Err((Error::<T>::TradeBatchError {
									error_code: Self::get_error_code(e),
								})
								.into());
							}
						},
					}

					new_position_size = quantity_to_execute + position_details.size;
					new_leverage = (margin_amount + borrowed_amount) / margin_amount;
					new_margin_locked = current_margin_locked + margin_lock_amount;
					new_realized_pnl = position_details.realized_pnl + realized_pnl;
					opening_fee = realized_pnl;

					// If the user previously does not have any position in this market
					// then add the market to CollateralToMarketMap
					if position_details.size == FixedI128::zero() {
						let opposite_direction = if element.direction == Direction::Long {
							Direction::Short
						} else {
							Direction::Long
						};
						let opposite_position = PositionsMap::<T>::get(
							&element.account_id,
							(market_id, opposite_direction),
						);
						if opposite_position.size == FixedI128::zero() {
							let mut markets =
								CollateralToMarketMap::<T>::get(&element.account_id, collateral_id);
							markets.push(market_id);
							CollateralToMarketMap::<T>::insert(
								&element.account_id,
								collateral_id,
								markets,
							);
						}
					}

					updated_position = Position {
						market_id,
						direction: element.direction,
						side: element.side,
						avg_execution_price,
						size: new_position_size,
						margin_amount,
						borrowed_amount,
						leverage: new_leverage,
						realized_pnl: new_realized_pnl,
					};

					open_interest = open_interest + quantity_to_execute;

					// Update initial margin locked amount map
					if element.direction == Direction::Long {
						initial_margin_locked_long = initial_margin_locked_long + margin_lock_amount
					} else {
						initial_margin_locked_short =
							initial_margin_locked_short + margin_lock_amount
					}

					if element.time_in_force == TimeInForce::IOC {
						is_final = true;
					} else {
						if new_portion_executed == element.size {
							is_final = true;
						} else {
							is_final = false;
						}
					}
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
						Ok((
							margin,
							borrowed,
							average_execution,
							balance,
							margin_lock,
							current_pnl,
						)) => {
							margin_amount = margin;
							borrowed_amount = borrowed;
							avg_execution_price = average_execution;
							user_available_balance = balance;
							margin_lock_amount = margin_lock;
							realized_pnl = current_pnl;
						},
						Err(e) => {
							// if maker order, emit event and process next order
							if element.order_id != orders[orders.len() - 1].order_id {
								Self::deposit_event(Event::OrderError {
									order_id: element.order_id,
									error_code: Self::get_error_code(e),
								});
								continue;
							} else {
								// if taker order, revert with error code
								return Err((Error::<T>::TradeBatchError {
									error_code: Self::get_error_code(e),
								})
								.into());
							}
						},
					}

					new_position_size = position_details.size - quantity_to_execute;

					// To do - handle liquidation/deleveraging order
					if (element.order_type == OrderType::Liquidation)
						|| (element.order_type == OrderType::Deleveraging)
					{
						let new_liq_position_size =
							liq_position.amount_to_be_sold - quantity_to_execute;

						if new_liq_position_size == FixedI128::zero() {
							new_liquidatable_position = LiquidatablePosition {
								market_id: 0,
								direction: Direction::Long,
								amount_to_be_sold: FixedI128::zero(),
								liquidatable: false,
							};
						} else {
							new_liquidatable_position = LiquidatablePosition {
								market_id: liq_position.market_id,
								direction: liq_position.direction,
								amount_to_be_sold: new_liq_position_size,
								liquidatable: liq_position.liquidatable,
							};
						}
						DeleveragableOrLiquidatableMap::<T>::insert(
							element.account_id,
							collateral_id,
							new_liquidatable_position,
						);
						if element.order_type == OrderType::Deleveraging {
							// Position not marked as 'deleveragable'
							if liq_position.liquidatable != false {
								return Err(
									(Error::<T>::TradeBatchError { error_code: 526 }).into()
								);
							}
							let total_value = margin_amount + borrowed_amount;
							new_leverage = total_value / margin_amount;
							new_margin_locked = current_margin_locked;
						} else {
							// Position not marked as 'liquidatable'
							if liq_position.liquidatable != true {
								return Err(
									(Error::<T>::TradeBatchError { error_code: 527 }).into()
								);
							}
							new_leverage = position_details.leverage;
							new_margin_locked = current_margin_locked - margin_lock_amount;
						}
					} else {
						new_leverage = position_details.leverage;
						new_margin_locked = current_margin_locked - margin_lock_amount;

						if (liq_position.market_id == market_id)
							&& (liq_position.direction == element.direction)
						{
							let new_liq_position_size =
								liq_position.amount_to_be_sold - quantity_to_execute;

							if new_liq_position_size == FixedI128::zero() {
								new_liquidatable_position = LiquidatablePosition {
									market_id: 0,
									direction: Direction::Long,
									amount_to_be_sold: FixedI128::zero(),
									liquidatable: false,
								};
							} else {
								new_liquidatable_position = LiquidatablePosition {
									market_id: liq_position.market_id,
									direction: liq_position.direction,
									amount_to_be_sold: new_liq_position_size,
									liquidatable: liq_position.liquidatable,
								};
							}
						} else {
							new_liquidatable_position = liq_position;
						}
						DeleveragableOrLiquidatableMap::<T>::insert(
							element.account_id,
							collateral_id,
							new_liquidatable_position,
						);
					}
					new_realized_pnl = position_details.realized_pnl + realized_pnl;
					opening_fee = FixedI128::zero();

					// If the user does not have any position in this market
					// then remove the market from CollateralToMarketMap
					if new_position_size == FixedI128::zero() {
						let opposite_direction = if element.direction == Direction::Long {
							Direction::Short
						} else {
							Direction::Long
						};
						let opposite_position = PositionsMap::<T>::get(
							&element.account_id,
							(market_id, opposite_direction),
						);
						if opposite_position.size == FixedI128::zero() {
							let mut markets =
								CollateralToMarketMap::<T>::get(&element.account_id, collateral_id);
							for index in 0..markets.len() {
								if markets[index] == market_id {
									markets.remove(index);
								}
							}
							CollateralToMarketMap::<T>::insert(
								&element.account_id,
								collateral_id,
								markets,
							);
						}
						updated_position = Position {
							market_id,
							direction: element.direction,
							side: element.side,
							avg_execution_price: FixedI128::zero(),
							size: FixedI128::zero(),
							margin_amount: FixedI128::zero(),
							borrowed_amount: FixedI128::zero(),
							leverage: FixedI128::zero(),
							realized_pnl: FixedI128::zero(),
						};
					} else {
						// To do - Calculate pnl

						updated_position = Position {
							market_id,
							direction: element.direction,
							side: element.side,
							avg_execution_price,
							size: new_position_size,
							margin_amount,
							borrowed_amount,
							leverage: new_leverage,
							realized_pnl: new_realized_pnl,
						};
					}

					if element.time_in_force == TimeInForce::IOC {
						new_portion_executed = element.size;
						is_final = true;
					} else {
						if new_portion_executed == element.size {
							is_final = true;
						} else {
							if new_position_size == FixedI128::zero() {
								is_final = true;
							} else {
								is_final = false;
							}
						}
					}

					open_interest = open_interest - quantity_to_execute;

					// Update initial margin locked amount map
					if element.direction == Direction::Long {
						initial_margin_locked_long = initial_margin_locked_long - margin_lock_amount
					} else {
						initial_margin_locked_short =
							initial_margin_locked_short - margin_lock_amount
					}
				}

				// Update position, locked margin and portion executed
				PositionsMap::<T>::set(
					&element.account_id,
					(market_id, element.direction),
					updated_position,
				);
				T::TradingAccountPallet::set_locked_margin(
					element.account_id,
					collateral_id,
					new_margin_locked,
				);
				PortionExecutedMap::<T>::insert(element.order_id, new_portion_executed);

				Self::deposit_event(Event::OrderExecuted {
					account_id: element.account_id,
					order_id: element.order_id,
					market_id: element.market_id,
					size: quantity_to_execute,
					direction: element.direction.into(),
					side: element.side.into(),
					order_type: element.order_type.into(),
					execution_price,
					pnl: realized_pnl,
					opening_fee,
					is_final,
				});
			}

			// Update open interest
			let actual_open_interest = open_interest / 2.into();
			let current_open_interest = OpenInterestMap::<T>::get(market_id);
			OpenInterestMap::<T>::insert(market_id, current_open_interest + actual_open_interest);

			// Update initial margin locked
			InitialMarginMap::<T>::insert((market_id, Direction::Long), initial_margin_locked_long);
			InitialMarginMap::<T>::insert(
				(market_id, Direction::Short),
				initial_margin_locked_short,
			);

			BatchStatusMap::<T>::insert(batch_id, true);

			// Emit trade executed event
			Self::deposit_event(Event::TradeExecuted {
				batch_id,
				market_id,
				size: taker_quantity,
				execution_price: taker_execution_price,
				direction: orders[orders.len() - 1].direction.into(),
				side: orders[orders.len() - 1].side.into(),
			});

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn calculate_initial_taker_locked_size(
			order: &Order,
			quantity_locked: FixedI128,
			market_id: u128,
			collateral_id: u128,
		) -> Result<FixedI128, Error<T>> {
			let order_portion_executed = PortionExecutedMap::<T>::get(order.order_id);

			let position_details =
				PositionsMap::<T>::get(&order.account_id, (market_id, order.direction));

			// Get liquidatable position details
			let liq_position: LiquidatablePosition =
				DeleveragableOrLiquidatableMap::<T>::get(&order.account_id, collateral_id);

			let quantity_response = Self::calculate_quantity_to_execute(
				order_portion_executed,
				market_id,
				&position_details,
				&order,
				liq_position,
				quantity_locked,
			);
			match quantity_response {
				Ok(quantity) => Ok(quantity),
				Err(e) => Err(e),
			}
		}

		fn calculate_quantity_to_execute(
			portion_executed: FixedI128,
			market_id: u128,
			position_details: &Position,
			order: &Order,
			liq_position: LiquidatablePosition,
			quantity_remaining: FixedI128,
		) -> Result<FixedI128, Error<T>> {
			let executable_quantity = order.size - portion_executed;
			ensure!(executable_quantity > FixedI128::zero(), Error::<T>::OrderFullyExecuted); // Modify code with tick/step size

			let mut quantity_to_execute = FixedI128::min(executable_quantity, quantity_remaining);
			ensure!(quantity_to_execute > FixedI128::zero(), Error::<T>::MakerOrderSkipped);

			if order.side == Side::Buy {
				Ok(quantity_to_execute)
			} else {
				if order.order_type == OrderType::Liquidation {
					ensure!(liq_position.market_id == market_id, Error::<T>::InvalidLiqOrDelMarket);
					ensure!(
						liq_position.direction == order.direction,
						Error::<T>::InvalidLiqOrDelDirection
					);
					quantity_to_execute =
						FixedI128::min(quantity_to_execute, liq_position.amount_to_be_sold);
				} else {
					quantity_to_execute =
						FixedI128::min(quantity_to_execute, position_details.size);
				}
				ensure!(quantity_to_execute > FixedI128::zero(), Error::<T>::ClosingEmptyPosition);
				Ok(quantity_to_execute)
			}
		}

		fn perform_validations(
			order: &Order,
			oracle_price: FixedI128,
			market: &Market,
		) -> Result<(), Error<T>> {
			// Validate that the user is registered
			let is_registered = T::TradingAccountPallet::is_registered_user(order.account_id);
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

			// Signature validation
			let sig_felt = sig_u256_to_sig_felt(&order.sig_r, &order.sig_s);

			// Sig_r and/or Sig_s could not be converted to FieldElement
			ensure!(sig_felt.is_ok(), Error::<T>::InvalidSignatureFelt);

			let (sig_r_felt, sig_s_felt) = sig_felt.unwrap();
			let sig = Signature { r: sig_r_felt, s: sig_s_felt };

			let order_hash = order.hash(&order.hash_type);

			// Order could not be hashed
			ensure!(order_hash.is_ok(), Error::<T>::InvalidOrderHash);

			let public_key = T::TradingAccountPallet::get_public_key(&order.account_id);

			// Public key not found for this account_id
			ensure!(public_key.is_some(), Error::<T>::NoPublicKeyFound);

			let public_key_felt = public_key.unwrap().try_to_felt();

			// Public Key U256 could not be converted to FieldElement
			ensure!(public_key_felt.is_ok(), Error::<T>::InvalidPublicKey);

			let verification = ecdsa_verify(&public_key_felt.unwrap(), &order_hash.unwrap(), &sig);

			// Signature verification returned error or false
			ensure!(verification.is_ok() && verification.unwrap(), Error::<T>::InvalidSignature);

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
			order_side: OrderSide,
			execution_price: FixedI128,
			oracle_price: FixedI128,
			market_id: u128,
			collateral_id: u128,
		) -> Result<(FixedI128, FixedI128, FixedI128, FixedI128, FixedI128, FixedI128), Error<T>> {
			let margin_amount: FixedI128;
			let borrowed_amount: FixedI128;
			let average_execution_price: FixedI128;
			let block_number = <frame_system::Pallet<T>>::block_number();

			let position_details =
				PositionsMap::<T>::get(&order.account_id, (market_id, order.direction));

			// Calculate average execution price
			if position_details.size == FixedI128::zero() {
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

			// Check if the position can be opened
			let (available_margin, is_liquidation) = T::RiskManagementPallet::check_for_risk(
				order,
				order_size,
				execution_price,
				oracle_price,
				margin_order_value,
			);

			ensure!(is_liquidation == false, Error::<T>::PassiveRiskError);

			let (fee_rate, _, _) =
				T::TradingFeesPallet::get_fee_rate(Side::Buy, order_side, U256::from(0));
			let fee = fee_rate * leveraged_order_value;
			let trading_fee = FixedI128::from_inner(0) - fee;

			ensure!(fee <= available_margin, Error::<T>::InsufficientBalance);
			T::TradingAccountPallet::transfer_from(
				order.account_id,
				collateral_id,
				fee,
				BalanceChangeReason::Fee,
			);

			Ok((
				margin_amount,
				borrowed_amount,
				average_execution_price,
				available_margin,
				margin_order_value,
				trading_fee,
			))
		}

		fn process_close_orders(
			order: &Order,
			order_size: FixedI128,
			execution_price: FixedI128,
			market_id: u128,
			collateral_id: u128,
		) -> Result<(FixedI128, FixedI128, FixedI128, FixedI128, FixedI128, FixedI128), Error<T>> {
			let actual_execution_price: FixedI128;
			let price_diff: FixedI128;
			let block_number = <frame_system::Pallet<T>>::block_number();

			let position_details =
				PositionsMap::<T>::get(&order.account_id, (market_id, order.direction));

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
			let borrowed_amount: FixedI128;
			let margin_amount: FixedI128;

			if order.order_type == OrderType::Deleveraging {
				// In delevereaging, we only reduce borrowed field
				borrowed_amount = position_details.borrowed_amount - leveraged_order_value;
				margin_amount = position_details.margin_amount;
			} else {
				borrowed_amount = position_details.borrowed_amount - borrowed_amount_to_return;
				margin_amount = position_details.margin_amount - margin_amount_to_reduce;
			}

			let unused_balance =
				T::TradingAccountPallet::get_unused_balance(order.account_id, collateral_id);

			// Check if user is under water, ie,
			// user has lost some borrowed funds
			if margin_plus_pnl.is_negative() {
				let amount_to_transfer_from = margin_plus_pnl.saturating_abs();

				// Check if user's balance can cover the deficit
				if amount_to_transfer_from > unused_balance {
					if order.order_type == OrderType::Limit {
						ensure!(false, Error::<T>::NotEnoughMargin);
					}

					if unused_balance.is_negative() {
						// Complete funds lost by user should be taken from insurance fund
						Self::deposit_event(Event::InsuranceFundChange {
							collateral_id,
							amount: amount_to_transfer_from,
							modify_type: FundModifyType::Decrease,
							block_number,
						});
					} else {
						// Some amount of lost funds can be taken from user available balance
						// Rest of the funds should be taken from insurance fund
						Self::deposit_event(Event::InsuranceFundChange {
							collateral_id,
							amount: amount_to_transfer_from - unused_balance,
							modify_type: FundModifyType::Decrease,
							block_number,
						});
					}
				}

				// Deduct under water amount (if any) + margin amt to reduce from user
				T::TradingAccountPallet::transfer_from(
					order.account_id,
					collateral_id,
					amount_to_transfer_from + margin_amount_to_reduce,
					BalanceChangeReason::PnlRealization,
				);
			// To do - calculate PnL
			} else {
				let balance = T::TradingAccountPallet::get_balance(order.account_id, collateral_id);
				if (order.order_type == OrderType::Market) || (order.order_type == OrderType::Limit)
				{
					// User is not under water
					// User is in loss
					if pnl.is_negative() {
						// Loss cannot be covered by the user
						if pnl.saturating_abs() > balance {
							// If balance is negative, deduct whole loss from insurance fund
							if balance.is_negative() {
								// User balance is negative, so deduct funds
								// from insurance fund
								Self::deposit_event(Event::InsuranceFundChange {
									collateral_id,
									amount: pnl.saturating_abs(),
									modify_type: FundModifyType::Decrease,
									block_number,
								});
							} else {
								// User has some balance to cover losses, remaining
								// should be taken from insurance fund
								Self::deposit_event(Event::InsuranceFundChange {
									collateral_id,
									amount: pnl.saturating_abs() - balance,
									modify_type: FundModifyType::Decrease,
									block_number,
								});
							}
						}

						// Deduct required funds from user
						T::TradingAccountPallet::transfer_from(
							order.account_id,
							collateral_id,
							pnl.saturating_abs(),
							BalanceChangeReason::PnlRealization,
						);
					} else {
						// User is in profit
						// Transfer the profit to user
						T::TradingAccountPallet::transfer(
							order.account_id,
							collateral_id,
							pnl,
							BalanceChangeReason::PnlRealization,
						);
					}
				} else {
					if order.order_type == OrderType::Liquidation {
						// if balance >= margin amount, deposit remaining margin in insurance
						if margin_amount_to_reduce <= balance {
							// Deposit margin_plus_pnl to insurance fund
							Self::deposit_event(Event::InsuranceFundChange {
								collateral_id,
								amount: margin_plus_pnl,
								modify_type: FundModifyType::Increase,
								block_number,
							});
						} else {
							if balance.is_negative() {
								// Deduct margin_amount_to_reduce from insurance fund
								Self::deposit_event(Event::InsuranceFundChange {
									collateral_id,
									amount: margin_plus_pnl,
									modify_type: FundModifyType::Decrease,
									block_number,
								});
							} else {
								// if user has some balance
								let pnl_abs = pnl.saturating_abs();
								if balance <= pnl_abs {
									// Deduct (pnl_abs -  balance) from insurance fund
									Self::deposit_event(Event::InsuranceFundChange {
										collateral_id,
										amount: pnl_abs - balance,
										modify_type: FundModifyType::Decrease,
										block_number,
									});
								} else {
									// Deposit (balance - pnl_abs) to insurance fund
									Self::deposit_event(Event::InsuranceFundChange {
										collateral_id,
										amount: balance - pnl_abs,
										modify_type: FundModifyType::Increase,
										block_number,
									});
								}
							}
						}
						// Deduct proportionate margin amount from user
						T::TradingAccountPallet::transfer_from(
							order.account_id,
							collateral_id,
							margin_amount_to_reduce,
							BalanceChangeReason::Liquidation,
						);
					// To do - calculate PnL
					} else {
						// To do - calculate PnL
					}
				}
			}

			Ok((
				margin_amount,
				borrowed_amount,
				position_details.avg_execution_price,
				unused_balance,
				margin_amount_to_reduce,
				pnl,
			))
		}

		fn get_trading_account(account_id: &U256) -> TradingAccountMinimal {
			let trading_account = T::TradingAccountPallet::get_account(account_id).unwrap();
			TradingAccountMinimal::new(trading_account.account_address, trading_account.index)
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
				Error::<T>::MakerOrderSkipped => 523,
				Error::<T>::ClosingEmptyPosition => 524,
				Error::<T>::InvalidDeleveragingOrder => 526,
				Error::<T>::InvalidLiquidationOrder => 527,
				Error::<T>::InvalidLiqOrDelMarket => 528,
				Error::<T>::InvalidLiqOrDelDirection => 529,
				Error::<T>::PassiveRiskError => 531,
				Error::<T>::NotEnoughMargin => 532,
				Error::<T>::OrderFullyExecuted => 533,
				Error::<T>::InvalidOrderHash => 534,
				Error::<T>::InvalidSignatureFelt => 535,
				Error::<T>::InvalidSignature => 536,
				Error::<T>::NoPublicKeyFound => 537,
				Error::<T>::InvalidPublicKey => 538,
				_ => 500,
			}
		}
	}

	impl<T: Config> TradingInterface for Pallet<T> {
		fn get_markets_of_collateral(account_id: U256, collateral_id: u128) -> Vec<u128> {
			let markets = CollateralToMarketMap::<T>::get(account_id, collateral_id);
			markets
		}

		fn get_position(account_id: U256, market_id: u128, direction: Direction) -> Position {
			let position_details = PositionsMap::<T>::get(account_id, (market_id, direction));
			position_details
		}

		fn liquidate_position(
			account_id: U256,
			collateral_id: u128,
			position: &PositionDetailsForRiskManagement,
			amount_to_be_sold: FixedI128,
		) {
			let amount;
			let liquidatable;
			if amount_to_be_sold == FixedI128::zero() {
				amount = position.size;
				liquidatable = true;
			} else {
				amount = amount_to_be_sold;
				liquidatable = false;
			}

			let liquidatable_position: LiquidatablePosition = LiquidatablePosition {
				market_id: position.market_id,
				direction: position.direction,
				amount_to_be_sold: amount,
				liquidatable,
			};

			DeleveragableOrLiquidatableMap::<T>::insert(
				account_id,
				collateral_id,
				liquidatable_position,
			);
		}

		fn get_deleveragable_or_liquidatable_position(
			account_id: U256,
			collateral_id: u128,
		) -> LiquidatablePosition {
			DeleveragableOrLiquidatableMap::<T>::get(account_id, collateral_id)
		}

		fn get_positions(account_id: U256, collateral_id: u128) -> Vec<Position> {
			let markets = CollateralToMarketMap::<T>::get(account_id, collateral_id);
			let mut pos_vec = Vec::<Position>::new();
			for element in markets {
				let long_pos: Position =
					PositionsMap::<T>::get(account_id, (element, Direction::Long));
				let short_pos: Position =
					PositionsMap::<T>::get(account_id, (element, Direction::Short));
				if long_pos.size != 0.into() {
					pos_vec.push(long_pos);
				}
				if short_pos.size != 0.into() {
					pos_vec.push(short_pos);
				}
			}
			return pos_vec;
		}
	}
}
