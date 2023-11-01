#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use core::option::Option;
	use frame_support::{
		dispatch::Vec,
		pallet_prelude::{ValueQuery, *},
	};
	use frame_system::pallet_prelude::*;
	use primitive_types::U256;
	use sp_arithmetic::{fixed_point::FixedI128, traits::Zero, FixedPointNumber};
	use zkx_support::{
		ecdsa_verify,
		helpers::sig_u256_to_sig_felt,
		traits::{
			AssetInterface, FieldElementExt, FixedI128Ext, Hashable, MarketInterface,
			PricesInterface, RiskManagementInterface, TradingAccountInterface,
			TradingFeesInterface, TradingInterface, U256Ext,
		},
		types::{
			AccountInfo, BalanceChangeReason, DeleveragablePosition, Direction, ForceClosureFlag,
			FundModifyType, MarginInfo, Market, Order, OrderSide, OrderType, Position,
			PositionDetailsForRiskManagement, PositionExtended, Side, TimeInForce,
		},
		Signature,
	};
	static LEVERAGE_ONE: FixedI128 = FixedI128::from_inner(1000000000000000000);

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type AssetPallet: AssetInterface;
		type MarketPallet: MarketInterface;
		type TradingAccountPallet: TradingAccountInterface;
		type TradingFeesPallet: TradingFeesInterface;
		type PricesPallet: PricesInterface;
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
	#[pallet::getter(fn deleveragable_position)]
	// Here, k1 - account_id,  k2 -  collateral_id, v -  DeleveragablePosition
	pub(super) type DeleveragableMap<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		U256,
		Blake2_128Concat,
		u128,
		DeleveragablePosition,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn force_closure_flag)]
	// Here, k1 - account_id,  k2 -  collateral_id, v -  force closure flag enum
	pub(super) type ForceClosureFlagMap<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		U256,
		Blake2_128Concat,
		u128,
		ForceClosureFlag,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn order_hash)]
	// k1 - order id, v - order hash
	pub(super) type OrderHashMap<T: Config> = StorageMap<_, Twox64Concat, u128, U256, ValueQuery>;

	#[pallet::error]
	pub enum Error<T> {
		/// Balance not enough to open the position
		TradeBatchError501,
		/// Invalid value for leverage (less than min or greater than currently allowed leverage)
		TradeBatchError502,
		/// Market matched and order market are different
		TradeBatchError504,
		/// Order size less than min quantity
		TradeBatchError505,
		/// Price is not within slippage limit
		TradeBatchError506,
		/// Execution price is not valid wrt limit price for long sell or short buy
		TradeBatchError507,
		/// Execution price is not valid wrt limit price for long buy or short sell
		TradeBatchError508,
		/// Invalid market
		TradeBatchError509,
		/// User's account is not registered
		TradeBatchError510,
		/// Taker side or direction is invalid wrt to makers
		TradeBatchError511,
		/// Maker side or direction does not match with other makers
		TradeBatchError512,
		/// Invalid oracle price,
		TradeBatchError513,
		/// Taker order is post only
		TradeBatchError515,
		/// FoK Orders should be filled completely
		TradeBatchError516,
		/// Maker order can only be limit order
		TradeBatchError518,
		/// Slippage must be between 0 and 15
		TradeBatchError521,
		/// Invalid quantity locked
		TradeBatchError522,
		/// Maker order skipped since quantity_executed = quantity_locked for the batch
		TradeBatchError523,
		/// Order is trying to close an empty position
		TradeBatchError524,
		/// Batch id already used
		TradeBatchError525,
		/// Position marked to be deleveraged, but liquidation order passed
		TradeBatchError526,
		/// Position marked to be liquidated, but deleveraging order passed
		TradeBatchError527,
		/// Invalid liquidation or deleveraging market
		TradeBatchError528,
		/// Invalid liquidation or deleveraging market direction
		TradeBatchError529,
		/// Position cannot be opened becuase of passive risk management
		TradeBatchError531,
		/// Not enough margin to cover losses - short limit sell or long limit sell
		TradeBatchError532,
		/// Order is fully executed
		TradeBatchError533,
		/// Invalid order hash - order could not be hashed into a Field Element
		TradeBatchError534,
		/// Invalid Signature Field Elements - sig_r and/or sig_s could not be converted into a
		/// Signature
		TradeBatchError535,
		/// ECDSA Signature could not be verified
		TradeBatchError536,
		/// Public Key not found for account id
		TradeBatchError537,
		/// Invalid public key - publickey u256 could not be converted to Field Element
		TradeBatchError538,
		/// When force closure flag is Liquidate or Deleverage, order type can only be Forced
		TradeBatchError539,
		/// If taker is forced, force closure flag must be present
		TradeBatchError540,
		/// Order hash mismatch for a particular order id
		TradeBatchError541,
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
			is_maker: bool,
		},
		/// Insurance fund updation event
		InsuranceFundChange {
			collateral_id: u128,
			amount: FixedI128,
			modify_type: FundModifyType,
			block_number: BlockNumberFor<T>,
		},
		/// Force closure flag updation event
		ForceClosureFlagsChanged { account_id: U256, collateral_id: u128, force_closure_flag: u8 },
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
			ensure_signed(origin)?;

			ensure!(!BatchStatusMap::<T>::contains_key(batch_id), Error::<T>::TradeBatchError525);

			// Validate market
			let market = T::MarketPallet::get_market(market_id);
			ensure!(market.is_some(), Error::<T>::TradeBatchError509);
			let market = market.unwrap();
			ensure!(market.is_tradable == true, Error::<T>::TradeBatchError509);

			let tick_precision = market.tick_precision;

			let collateral_asset = T::AssetPallet::get_asset(market.asset_collateral).unwrap();
			let collateral_token_decimal = collateral_asset.decimals;

			// validates oracle_price
			ensure!(oracle_price > FixedI128::zero(), Error::<T>::TradeBatchError513);

			//Update market price
			let market_price = T::PricesPallet::get_market_price(market_id);
			if market_price == FixedI128::zero() {
				T::PricesPallet::update_market_price(market_id, oracle_price);
			}

			let collateral_id: u128 = market.asset_collateral;
			let initial_taker_locked_quantity: FixedI128;

			ensure!(
				quantity_locked > FixedI128::checked_from_integer(0).unwrap(),
				Error::<T>::TradeBatchError522
			);

			// Calculate quantity that can be executed for the taker, before starting with the maker
			// orders
			let taker_order = &orders[orders.len() - 1];
			let initial_taker_locked_response = Self::calculate_initial_taker_locked_size(
				taker_order,
				quantity_locked,
				market_id,
				collateral_id,
			);
			match initial_taker_locked_response {
				Ok(quantity) => initial_taker_locked_quantity = quantity,
				Err(e) => return Err(e.into()),
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
				let mut margin_amount: FixedI128;
				let mut borrowed_amount: FixedI128;
				let mut avg_execution_price: FixedI128;
				let execution_price: FixedI128;
				let quantity_to_execute: FixedI128;
				let mut margin_lock_amount: FixedI128;
				let new_position_size: FixedI128;
				let mut new_leverage: FixedI128;
				let new_margin_locked: FixedI128;
				let mut new_portion_executed: FixedI128;
				let realized_pnl: FixedI128;
				let new_realized_pnl: FixedI128;
				let opening_fee: FixedI128;
				let order_side: OrderSide;

				let validation_response =
					Self::perform_validations(element, oracle_price, &market, collateral_id);
				match validation_response {
					Ok(()) => (),
					Err(e) => {
						// if maker order, emit event and process next order
						if element.order_id != orders[orders.len() - 1].order_id {
							Self::deposit_event(Event::OrderError {
								order_id: element.order_id,
								error_code: Self::get_error_code(e),
							});
							continue
						} else {
							// if taker order, revert with error
							return Err(e.into())
						}
					},
				}

				let order_portion_executed = PortionExecutedMap::<T>::get(element.order_id);
				let position_details =
					PositionsMap::<T>::get(&element.account_id, (market_id, element.direction));
				let current_margin_locked =
					T::TradingAccountPallet::get_locked_margin(element.account_id, collateral_id);

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
							continue
						},
					}
					// Calculate quantity left to be executed
					let quantity_remaining = initial_taker_locked_quantity - quantity_executed;

					// Calculate quantity that needs to be executed for the current maker
					let maker_quantity_to_execute_response = Self::calculate_quantity_to_execute(
						order_portion_executed,
						market_id,
						collateral_id,
						&position_details,
						element,
						quantity_remaining,
					);
					match maker_quantity_to_execute_response {
						Ok(quantity) => quantity_to_execute = quantity,
						Err(e) => {
							Self::deposit_event(Event::OrderError {
								order_id: element.order_id,
								error_code: Self::get_error_code(e),
							});
							continue
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
						Err(e) => return Err(e.into()),
					}

					// Taker quantity to be executed will be sum of maker quantities executed
					quantity_to_execute = quantity_executed;
					if quantity_to_execute == FixedI128::zero() {
						Self::deposit_event(Event::TradeExecutionFailed { batch_id });
						return Ok(())
					}

					// Handle FoK order
					if element.time_in_force == TimeInForce::FOK {
						if quantity_to_execute != element.size {
							return Err((Error::<T>::TradeBatchError516).into())
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
							Err(e) => return Err(e.into()),
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
							Err(e) => return Err(e.into()),
						}
					}

					order_side = OrderSide::Taker;

					taker_execution_price = execution_price;
					taker_quantity = quantity_to_execute;
				}

				new_portion_executed = order_portion_executed + quantity_to_execute;

				let is_final: bool;
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
							_balance,
							margin_lock,
							trading_fee,
						)) => {
							margin_amount = margin;
							borrowed_amount = borrowed;
							avg_execution_price = average_execution;
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
								continue
							} else {
								// if taker order, revert with error code
								return Err(e.into())
							}
						},
					}

					margin_amount =
						margin_amount.round_to_precision(collateral_token_decimal.into());
					borrowed_amount =
						borrowed_amount.round_to_precision(collateral_token_decimal.into());
					margin_lock_amount =
						margin_lock_amount.round_to_precision(collateral_token_decimal.into());
					avg_execution_price =
						avg_execution_price.round_to_precision(tick_precision.into());
					new_position_size = quantity_to_execute + position_details.size;
					new_leverage = (margin_amount + borrowed_amount) / margin_amount;
					new_leverage = new_leverage.round_to_precision(2);
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
					PositionsMap::<T>::set(
						&element.account_id,
						(market_id, element.direction),
						updated_position,
					);

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
						order_side,
						execution_price,
						market_id,
						collateral_id,
					);
					match response {
						Ok((
							margin,
							borrowed,
							average_execution,
							_balance,
							margin_lock,
							current_pnl,
						)) => {
							margin_amount = margin;
							borrowed_amount = borrowed;
							avg_execution_price = average_execution;
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
								continue
							} else {
								// if taker order, revert with error code
								return Err(e.into())
							}
						},
					}

					margin_amount =
						margin_amount.round_to_precision(collateral_token_decimal.into());
					borrowed_amount =
						borrowed_amount.round_to_precision(collateral_token_decimal.into());
					margin_lock_amount =
						margin_lock_amount.round_to_precision(collateral_token_decimal.into());
					avg_execution_price =
						avg_execution_price.round_to_precision(tick_precision.into());
					new_position_size = position_details.size - quantity_to_execute;

					let force_closure_flag =
						ForceClosureFlagMap::<T>::get(element.account_id, collateral_id);
					// Deleveraging case, update deleveragable position and force closure flag
					// accordingly
					if force_closure_flag.is_some() &&
						force_closure_flag.unwrap() == ForceClosureFlag::Deleverage
					{
						let deleveragable_position =
							DeleveragableMap::<T>::get(element.account_id, collateral_id);
						let new_deleverage_position_size =
							deleveragable_position.amount_to_be_sold - quantity_to_execute;

						if new_deleverage_position_size == FixedI128::zero() {
							DeleveragableMap::<T>::remove(element.account_id, collateral_id);

							// Remove the liquidation flag and check for deferred deposits
							Self::reset_force_closure_flags(element.account_id, collateral_id)?;
						} else {
							let new_deleverage_position = DeleveragablePosition {
								market_id: deleveragable_position.market_id,
								direction: deleveragable_position.direction,
								amount_to_be_sold: new_deleverage_position_size,
							};
							DeleveragableMap::<T>::insert(
								element.account_id,
								collateral_id,
								new_deleverage_position,
							);
						}

						let total_value = margin_amount + borrowed_amount;
						new_leverage = total_value / margin_amount;
						new_leverage = new_leverage.round_to_precision(2);
						new_margin_locked = current_margin_locked;
					} else {
						// Normal and liquidation case
						new_leverage = position_details.leverage;
						new_margin_locked = current_margin_locked - margin_lock_amount;
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
								&markets,
							);

							// If force closure flag is liquidation and if all positions are closed,
							// it means that liquidation is complete
							if force_closure_flag.is_some() &&
								force_closure_flag.unwrap() == ForceClosureFlag::Liquidate &&
								markets.is_empty()
							{
								// Remove the liquidation flag and check for deferred deposits
								Self::reset_force_closure_flags(element.account_id, collateral_id)?;
							}
						}
						PositionsMap::<T>::remove(
							element.account_id,
							(market_id, element.direction),
						);
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
						PositionsMap::<T>::set(
							&element.account_id,
							(market_id, element.direction),
							updated_position,
						);
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

				// Update locked margin and portion executed
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
					is_maker: element.order_id != orders[orders.len() - 1].order_id,
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
		fn reset_force_closure_flags(account_id: U256, collateral_id: u128) -> DispatchResult {
			// Reset the flag
			ForceClosureFlagMap::<T>::remove(account_id, collateral_id);

			// Add deferred deposits if any
			T::TradingAccountPallet::add_deferred_balance(account_id, collateral_id)?;

			Ok(())
		}

		fn calculate_initial_taker_locked_size(
			order: &Order,
			quantity_locked: FixedI128,
			market_id: u128,
			collateral_id: u128,
		) -> Result<FixedI128, Error<T>> {
			let order_portion_executed = PortionExecutedMap::<T>::get(order.order_id);

			let position_details =
				PositionsMap::<T>::get(&order.account_id, (market_id, order.direction));

			// This call is necessary if taker is Forced order, so that force closure flag can be
			// set and also if deleveraging, amount to be sold can be calculated,
			// which is required to calculate quantity to execute
			if order.order_type == OrderType::Forced {
				T::RiskManagementPallet::check_for_force_closure(
					order.account_id,
					collateral_id,
					market_id,
					order.direction,
				);

				let force_closure_flag =
					ForceClosureFlagMap::<T>::get(order.account_id, collateral_id);
				ensure!(force_closure_flag.is_some(), Error::<T>::TradeBatchError540);
			}

			let quantity_response = Self::calculate_quantity_to_execute(
				order_portion_executed,
				market_id,
				collateral_id,
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
			market_id: u128,
			collateral_id: u128,
			position_details: &Position,
			order: &Order,
			quantity_remaining: FixedI128,
		) -> Result<FixedI128, Error<T>> {
			let executable_quantity = order.size - portion_executed;
			ensure!(executable_quantity > FixedI128::zero(), Error::<T>::TradeBatchError533);

			let mut quantity_to_execute = FixedI128::min(executable_quantity, quantity_remaining);
			ensure!(quantity_to_execute > FixedI128::zero(), Error::<T>::TradeBatchError523);

			if order.side == Side::Buy {
				Ok(quantity_to_execute)
			} else {
				if order.order_type == OrderType::Forced {
					let force_closure_flag =
						ForceClosureFlagMap::<T>::get(order.account_id, collateral_id);

					match force_closure_flag.unwrap() {
						ForceClosureFlag::Deleverage => {
							let deleveragable_position =
								DeleveragableMap::<T>::get(&order.account_id, collateral_id);
							ensure!(
								deleveragable_position.market_id == market_id,
								Error::<T>::TradeBatchError528
							);
							ensure!(
								deleveragable_position.direction == order.direction,
								Error::<T>::TradeBatchError529
							);
							quantity_to_execute = FixedI128::min(
								quantity_to_execute,
								deleveragable_position.amount_to_be_sold,
							);
						},
						_ => {
							quantity_to_execute =
								FixedI128::min(quantity_to_execute, position_details.size);
						},
					}
				} else {
					quantity_to_execute =
						FixedI128::min(quantity_to_execute, position_details.size);
				}
				ensure!(quantity_to_execute > FixedI128::zero(), Error::<T>::TradeBatchError524);
				Ok(quantity_to_execute)
			}
		}

		fn perform_validations(
			order: &Order,
			_oracle_price: FixedI128,
			market: &Market,
			collateral_id: u128,
		) -> Result<(), Error<T>> {
			// Validate that the user is registered
			let is_registered = T::TradingAccountPallet::is_registered_user(order.account_id);
			ensure!(is_registered, Error::<T>::TradeBatchError510);

			// Validate that if force closure flag is either deleverage or liquidate
			// order type can only be 'Forced'
			if order.order_type != OrderType::Forced {
				let force_closure_flag =
					ForceClosureFlagMap::<T>::get(order.account_id, collateral_id);
				ensure!(force_closure_flag.is_none(), Error::<T>::TradeBatchError539);
			}

			// Validate that size of order is >= min quantity for market
			ensure!(order.size >= market.minimum_order_size, Error::<T>::TradeBatchError505);

			// Validate that market matched and market in order are same
			ensure!(market.id == order.market_id, Error::<T>::TradeBatchError504);

			// Validate leverage value
			ensure!(
				order.leverage >= LEVERAGE_ONE &&
					order.leverage <= market.currently_allowed_leverage,
				Error::<T>::TradeBatchError502
			);

			// Signature validation
			let sig_felt = sig_u256_to_sig_felt(&order.sig_r, &order.sig_s);

			// Sig_r and/or Sig_s could not be converted to FieldElement
			ensure!(sig_felt.is_ok(), Error::<T>::TradeBatchError535);

			let (sig_r_felt, sig_s_felt) = sig_felt.unwrap();
			let sig = Signature { r: sig_r_felt, s: sig_s_felt };

			let order_hash = order.hash(&order.hash_type);

			// Order could not be hashed
			ensure!(order_hash.is_ok(), Error::<T>::TradeBatchError534);

			let public_key = T::TradingAccountPallet::get_public_key(&order.account_id);

			// Public key not found for this account_id
			ensure!(public_key.is_some(), Error::<T>::TradeBatchError537);

			let public_key_felt = public_key.unwrap().try_to_felt();

			// Public Key U256 could not be converted to FieldElement
			ensure!(public_key_felt.is_ok(), Error::<T>::TradeBatchError538);

			let order_hash = order_hash.unwrap();

			let verification = ecdsa_verify(&public_key_felt.unwrap(), &order_hash, &sig);

			// Signature verification returned error or false
			ensure!(verification.is_ok() && verification.unwrap(), Error::<T>::TradeBatchError536);

			let order_hash_u256 = order_hash.to_u256();
			// Check for order hash collision
			let is_success = Self::order_hash_check(order.order_id, order_hash_u256);
			ensure!(is_success, Error::<T>::TradeBatchError541);

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
				(current_direction == maker1_direction && current_side == maker1_side) ||
					(current_direction == opposite_direction && current_side == opposite_side),
				Error::<T>::TradeBatchError512
			);

			ensure!(order_type == OrderType::Limit, Error::<T>::TradeBatchError518);

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
				(current_direction == maker1_direction && current_side == opposite_side) ||
					(current_direction == opposite_direction && current_side == maker1_side),
				Error::<T>::TradeBatchError511
			);

			// Taker order cannot be post only order
			ensure!(post_only == false, Error::<T>::TradeBatchError515);

			Ok(())
		}

		fn validate_limit_price(
			price: FixedI128,
			execution_price: FixedI128,
			direction: Direction,
			side: Side,
		) -> Result<(), Error<T>> {
			if (direction == Direction::Long && side == Side::Buy) ||
				(direction == Direction::Short && side == Side::Sell)
			{
				ensure!(execution_price <= price, Error::<T>::TradeBatchError508);
			} else {
				ensure!(price <= execution_price, Error::<T>::TradeBatchError507);
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
			ensure!(
				slippage > FixedI128::zero() &&
					slippage <= FixedI128::from_inner(150000000000000000),
				Error::<T>::TradeBatchError521
			);
			let threshold = slippage * oracle_price;

			if (direction == Direction::Long && side == Side::Buy) ||
				(direction == Direction::Short && side == Side::Sell)
			{
				ensure!(
					execution_price <= (oracle_price + threshold),
					Error::<T>::TradeBatchError506
				);
			} else {
				ensure!(
					(oracle_price - threshold) <= execution_price,
					Error::<T>::TradeBatchError506
				);
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
			let _block_number = <frame_system::Pallet<T>>::block_number();

			let position_details =
				PositionsMap::<T>::get(&order.account_id, (market_id, order.direction));

			// Calculate average execution price
			if position_details.size == FixedI128::zero() {
				average_execution_price = execution_price;
			} else {
				let cumulative_order_value = (position_details.size *
					position_details.avg_execution_price) +
					(order_size * execution_price);
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

			ensure!(is_liquidation == false, Error::<T>::TradeBatchError531);

			let (fee_rate, _, _) =
				T::TradingFeesPallet::get_fee_rate(Side::Buy, order_side, U256::zero());
			let fee = fee_rate * leveraged_order_value;
			let trading_fee = FixedI128::from_inner(0) - fee;

			ensure!(fee <= available_margin, Error::<T>::TradeBatchError501);
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
			order_side: OrderSide,
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
			let mut pnl = order_size * price_diff;
			let margin_plus_pnl = margin_amount_to_reduce + pnl;
			let borrowed_amount: FixedI128;
			let margin_amount: FixedI128;

			let force_closure_flag = ForceClosureFlagMap::<T>::get(order.account_id, collateral_id);
			if force_closure_flag.is_some() &&
				force_closure_flag.unwrap() == ForceClosureFlag::Deleverage
			{
				// In deleveraging, we only reduce borrowed field
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
						ensure!(false, Error::<T>::TradeBatchError532);
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
				if order.order_type != OrderType::Forced {
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
					let force_closure_flag =
						ForceClosureFlagMap::<T>::get(order.account_id, collateral_id);
					match force_closure_flag.unwrap() {
						// Liquidation case when user is not underwater
						ForceClosureFlag::Liquidate => {
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
										amount: margin_amount_to_reduce,
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
						},
						ForceClosureFlag::Deleverage => {
							pnl = FixedI128::zero();
						},
					}
				}
			}

			let (fee_rate, _, _) =
				T::TradingFeesPallet::get_fee_rate(Side::Sell, order_side, U256::zero());
			let fee = fee_rate * leveraged_order_value;

			// Deduct fee while closing a position
			T::TradingAccountPallet::transfer_from(
				order.account_id,
				collateral_id,
				fee,
				BalanceChangeReason::Fee,
			);

			Ok((
				margin_amount,
				borrowed_amount,
				position_details.avg_execution_price,
				unused_balance,
				margin_amount_to_reduce,
				pnl,
			))
		}

		fn get_maintenance_requirement(
			market_id: u128,
			position: &Position,
		) -> (FixedI128, FixedI128) {
			let market = T::MarketPallet::get_market(market_id).unwrap();
			let required_margin = market.maintenance_margin_fraction;

			// Calculate required margin
			let position_value = position.size * position.avg_execution_price;
			let maintenance_requirement = position_value * required_margin;

			let market_price = T::PricesPallet::get_market_price(market_id);

			(maintenance_requirement, market_price)
		}

		fn get_error_code(error: Error<T>) -> u16 {
			match error {
				Error::<T>::TradeBatchError501 => 501,
				Error::<T>::TradeBatchError502 => 502,
				Error::<T>::TradeBatchError504 => 504,
				Error::<T>::TradeBatchError505 => 505,
				Error::<T>::TradeBatchError506 => 506,
				Error::<T>::TradeBatchError507 => 507,
				Error::<T>::TradeBatchError508 => 508,
				Error::<T>::TradeBatchError510 => 510,
				Error::<T>::TradeBatchError511 => 511,
				Error::<T>::TradeBatchError512 => 512,
				Error::<T>::TradeBatchError515 => 515,
				Error::<T>::TradeBatchError518 => 518,
				Error::<T>::TradeBatchError523 => 523,
				Error::<T>::TradeBatchError524 => 524,
				Error::<T>::TradeBatchError526 => 526,
				Error::<T>::TradeBatchError527 => 527,
				Error::<T>::TradeBatchError528 => 528,
				Error::<T>::TradeBatchError529 => 529,
				Error::<T>::TradeBatchError531 => 531,
				Error::<T>::TradeBatchError532 => 532,
				Error::<T>::TradeBatchError533 => 533,
				Error::<T>::TradeBatchError534 => 534,
				Error::<T>::TradeBatchError535 => 535,
				Error::<T>::TradeBatchError536 => 536,
				Error::<T>::TradeBatchError537 => 537,
				Error::<T>::TradeBatchError538 => 538,
				Error::<T>::TradeBatchError539 => 539,
				Error::<T>::TradeBatchError541 => 541,
				Error::<T>::TradeBatchError540 => 540,
				_ => 500,
			}
		}

		fn order_hash_check(order_id: u128, order_hash: U256) -> bool {
			// Get the hash of the order associated with the order_id
			let existing_hash = OrderHashMap::<T>::get(order_id);
			// If the hash isn't stored in the contract yet
			if existing_hash == U256::zero() {
				OrderHashMap::<T>::insert(order_id, order_hash);
				true
			} else {
				if existing_hash == order_hash {
					true
				} else {
					false
				}
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

		fn set_flags_for_force_orders(
			account_id: U256,
			collateral_id: u128,
			force_closure_flag: ForceClosureFlag,
			deleveragable_position: DeleveragablePosition,
		) {
			// Liquidation
			if force_closure_flag == ForceClosureFlag::Liquidate {
				ForceClosureFlagMap::<T>::insert(
					account_id,
					collateral_id,
					ForceClosureFlag::Liquidate,
				);
				DeleveragableMap::<T>::remove(account_id, collateral_id);
			} else {
				// Deleveraging
				ForceClosureFlagMap::<T>::insert(
					account_id,
					collateral_id,
					ForceClosureFlag::Deleverage,
				);
				DeleveragableMap::<T>::insert(account_id, collateral_id, deleveragable_position);
			}

			// Emit event
			Self::deposit_event(Event::ForceClosureFlagsChanged {
				account_id,
				collateral_id,
				force_closure_flag: force_closure_flag.into(),
			});
		}

		fn get_deleveragable_position(
			account_id: U256,
			collateral_id: u128,
		) -> DeleveragablePosition {
			DeleveragableMap::<T>::get(account_id, collateral_id)
		}

		fn get_positions(account_id: U256, collateral_id: u128) -> Vec<PositionExtended> {
			let markets = CollateralToMarketMap::<T>::get(account_id, collateral_id);
			let mut pos_vec = Vec::<PositionExtended>::new();
			for element in markets {
				let long_pos: Position =
					PositionsMap::<T>::get(account_id, (element, Direction::Long));
				let short_pos: Position =
					PositionsMap::<T>::get(account_id, (element, Direction::Short));

				if long_pos.size != FixedI128::zero() {
					let (maintenance_requirement, market_price) =
						Self::get_maintenance_requirement(element, &long_pos);
					let position_extended =
						PositionExtended::new(long_pos, maintenance_requirement, market_price);
					pos_vec.push(position_extended);
				}
				if short_pos.size != FixedI128::zero() {
					let (maintenance_requirement, market_price) =
						Self::get_maintenance_requirement(element, &short_pos);
					let position_extended =
						PositionExtended::new(short_pos, maintenance_requirement, market_price);
					pos_vec.push(position_extended);
				}
			}
			pos_vec
		}

		fn get_account_margin_info(account_id: U256, collateral_id: u128) -> MarginInfo {
			let (
				is_liquidation,
				total_margin,
				available_margin,
				unrealized_pnl_sum,
				maintenance_margin_requirement,
				least_collateral_ratio,
				least_collateral_ratio_position,
				least_collateral_ratio_position_asset_price,
			) = T::TradingAccountPallet::get_margin_info(
				account_id,
				collateral_id,
				FixedI128::zero(),
				FixedI128::zero(),
			);

			MarginInfo {
				is_liquidation,
				total_margin,
				available_margin,
				unrealized_pnl_sum,
				maintenance_margin_requirement,
				least_collateral_ratio,
				least_collateral_ratio_position,
				least_collateral_ratio_position_asset_price,
			}
		}

		fn get_account_info(account_id: U256, collateral_id: u128) -> AccountInfo {
			let (_, total_margin, available_margin, _, _, _, _, _) =
				T::TradingAccountPallet::get_margin_info(
					account_id,
					collateral_id,
					FixedI128::zero(),
					FixedI128::zero(),
				);

			let markets = CollateralToMarketMap::<T>::get(account_id, collateral_id);
			let mut positions = Vec::<PositionExtended>::new();
			for element in markets {
				let long_pos: Position =
					PositionsMap::<T>::get(account_id, (element, Direction::Long));
				let short_pos: Position =
					PositionsMap::<T>::get(account_id, (element, Direction::Short));
				if long_pos.size != FixedI128::zero() {
					let (maintenance_requirement, market_price) =
						Self::get_maintenance_requirement(element, &long_pos);
					let position_extended =
						PositionExtended::new(long_pos, maintenance_requirement, market_price);
					positions.push(position_extended);
				}
				if short_pos.size != FixedI128::zero() {
					let (maintenance_requirement, market_price) =
						Self::get_maintenance_requirement(element, &short_pos);
					let position_extended =
						PositionExtended::new(short_pos, maintenance_requirement, market_price);
					positions.push(position_extended);
				}
			}

			let collateral_balance =
				T::TradingAccountPallet::get_balance(account_id, collateral_id);

			let force_closure_flag = ForceClosureFlagMap::<T>::get(account_id, collateral_id);
			let deleveragable_position = DeleveragableMap::<T>::get(account_id, collateral_id);

			AccountInfo {
				positions,
				available_margin,
				total_margin,
				collateral_balance,
				force_closure_flag,
				deleveragable_position,
			}
		}

		fn get_account_list(start_index: u128, end_index: u128) -> Vec<U256> {
			T::TradingAccountPallet::get_account_list(start_index, end_index)
		}

		fn get_force_closure_flags(
			account_id: U256,
			collateral_id: u128,
		) -> Option<ForceClosureFlag> {
			ForceClosureFlagMap::<T>::get(account_id, collateral_id)
		}
	}
}
