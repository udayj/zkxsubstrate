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
		ensure,
		pallet_prelude::{OptionQuery, ValueQuery, *},
		traits::UnixTime,
	};
	use frame_system::{
		offchain::{AppCrypto, CreateSignedTransaction, SendSignedTransaction, Signer},
		pallet_prelude::*,
	};
	use pallet_support::{
		ecdsa_verify,
		helpers::{get_expiry_timestamp, sig_u256_to_sig_felt},
		traits::{
			AssetInterface, FieldElementExt, FixedI128Ext, Hashable, MarketInterface,
			PricesInterface, RiskManagementInterface, TradingAccountInterface,
			TradingFeesInterface, TradingInterface, U256Ext,
		},
		types::{
			AccountInfo, BalanceChangeReason, BaseFeeAggregate, Direction, FeeRates,
			ForceClosureFlag, FundModifyType, MarginInfo, Market, Order, OrderSide, OrderType,
			Position, PositionExtended, Side, SignatureInfo, TimeInForce, VolumeType,
		},
		Signature,
	};
	use primitive_types::U256;
	use sp_arithmetic::{
		traits::{One, Zero},
		FixedI128, FixedPointNumber,
	};
	use sp_runtime::traits::SaturatedConversion;

	static LEVERAGE_ONE: FixedI128 = FixedI128::from_inner(1000000000000000000);
	static FOUR_WEEKS: u64 = 2419200;
	static CLEANUP_COUNT: u64 = 120;
	// Block interval at which offchain workers will be executed for clearing order details
	const BLOCK_INTERVAL: u32 = 130;
	// Block interval at which offchain workers will be executed for closing delisted market
	// positions
	const BLOCK_INTERVAL_FOR_DELISTING: u32 = 120;
	// No.of positions to be closed for a delisted market
	static POSITIONS_CLOSE_COUNT: u32 = 100;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: CreateSignedTransaction<Call<Self>> + frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type AssetPallet: AssetInterface;
		type MarketPallet: MarketInterface;
		type TradingAccountPallet: TradingAccountInterface;
		type TradingFeesPallet: TradingFeesInterface;
		type PricesPallet: PricesInterface;
		type RiskManagementPallet: RiskManagementInterface;
		type TimeProvider: UnixTime;
		type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
	}

	#[pallet::storage]
	#[pallet::getter(fn batch_status)]
	// k1 - batch id, v - true/false
	pub(super) type BatchStatusMap<T: Config> = StorageMap<_, Twox64Concat, U256, bool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn order_state)]
	// k1 - order id, v - (portion executed, isCancelled flag)
	pub(super) type OrderStateMap<T: Config> =
		StorageMap<_, Twox64Concat, U256, (FixedI128, bool), ValueQuery>;

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
	#[pallet::getter(fn market_to_user)]
	// k1 - (market_id, direction), k2 - account_id, v - account_id
	// It is not being used anymore. It is to be removed.
	pub(super) type MarketToUserMap<T: Config> =
		StorageDoubleMap<_, Twox64Concat, (u128, Direction), Twox64Concat, U256, U256, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn market_to_account)]
	// k1 - (market_id, direction), k2 - account_id, v - (account_id, group_index)
	pub(super) type MarketToAccountMap<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		(u128, Direction),
		Twox64Concat,
		U256,
		(U256, u128),
		OptionQuery,
	>;

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
	#[pallet::getter(fn deleveragable_amount)]
	// Here, k1 - account_id,  k2 -  collateral_id, v -  amount_to_be_sold
	pub(super) type DeleveragableMap<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, U256, Blake2_128Concat, u128, FixedI128, ValueQuery>;

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
	pub(super) type OrderHashMap<T: Config> = StorageMap<_, Twox64Concat, U256, U256, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn liquidator_signers)]
	// Array of U256 signers
	pub(super) type LiquidatorSigners<T: Config> = StorageValue<_, Vec<U256>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn is_liquidator_signer_valid)]
	// k1 - signer, v - bool
	pub(super) type IsLiquidatorSignerWhitelisted<T: Config> =
		StorageMap<_, Twox64Concat, U256, bool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn orders)]
	// k1 - timestamp, v - vector of order_ids
	pub(super) type OrdersMap<T: Config> = StorageMap<_, Twox64Concat, u64, Vec<U256>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn batches)]
	// k1 - timestamp, v - vector of batch_ids
	pub(super) type BatchesMap<T: Config> =
		StorageMap<_, Twox64Concat, u64, Vec<U256>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn start_timestamp)]
	// The beginning timestamp for which batch_id and order_id info are stored
	pub(super) type StartTimestamp<T: Config> = StorageValue<_, u64, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn trading_fee)]
	// k1 - collateral id, v - trading fee
	pub(super) type TradingFeeMap<T: Config> =
		StorageMap<_, Twox64Concat, u128, FixedI128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn liquidation_fee)]
	// k1 - collateral id, v - liquidation fee
	pub(super) type LiquidationFeeMap<T: Config> =
		StorageMap<_, Twox64Concat, u128, FixedI128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn matching_time_limit)]
	pub(super) type MatchingTimeLimit<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn withdrawal_signer)]
	// Array of U256 signers
	pub(super) type WithdrawalSigner<T: Config> = StorageValue<_, U256, ValueQuery>;

	#[pallet::genesis_config]
	#[derive(frame_support::DefaultNoBound)]
	pub struct GenesisConfig<T: Config> {
		pub matching_time_limit: u64,
		#[serde(skip)]
		pub _config: sp_std::marker::PhantomData<T>,
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			MatchingTimeLimit::<T>::put(&self.matching_time_limit);
		}
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Balance not enough to open the position
		TradeBatchError501,
		/// Invalid value for leverage (less than min or greater than currently allowed leverage)
		TradeBatchError502,
		/// Invalid quantity locked w.r.t step size
		TradeBatchError503,
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
		/// Taker order could not be executed since all makers do not satisfy slippage limit
		TradeBatchError514,
		/// Taker order is post only
		TradeBatchError515,
		/// FoK Orders should be filled completely
		TradeBatchError516,
		/// Order size is invalid w.r.t to step size
		TradeBatchError517,
		/// Maker order can only be limit order
		TradeBatchError518,
		/// Slippage must be between 0 and 15
		TradeBatchError521,
		/// Quantity locked must be > 0
		TradeBatchError522,
		/// Maker order skipped since quantity_executed = quantity_locked for the batch
		TradeBatchError523,
		/// Order is trying to close an empty position
		TradeBatchError524,
		/// Batch id already used
		TradeBatchError525,
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
		/// Invalid public key - publickey u256 could not be converted to Field Element
		TradeBatchError538,
		/// When force closure flag is Liquidate or Deleverage, order type can only be Forced
		TradeBatchError539,
		/// If taker is forced, force closure flag must be present
		TradeBatchError540,
		/// Order hash mismatch for a particular order id
		TradeBatchError541,
		/// When a non-whitelisted pub key is used in liquidation
		TradeBatchError542,
		/// When a cancelled order is sent for execution
		TradeBatchError543,
		/// Order is older than 4 weeks
		TradeBatchError544,
		/// Batch is older than expected
		TradeBatchError545,
		/// Trade Volume Calculation Error
		TradeBatchError546,
		/// Insufficient num of orders in the batch
		TradeBatchError547,
		// The resulting position size is larger than the max size allowed in the market
		TradeBatchError548,
		/// FoK, IoC error when maker has some issue
		IocFokError(u8, u16),
		/// When a zero signer is being added
		ZeroSigner,
		/// When a duplicate signer is being added
		DuplicateSigner,
		/// When the order is signed with a pub key that is not whitelisted
		SignerNotWhitelisted,
		/// When order id passed is zero
		ZeroOrderId,
		/// Start timestamp is not set
		StartTimestampEmpty,
		/// When mark price for ADS is not available
		ADSPriceNotAvailable,
		/// Error while adding deferred deposit
		DeferredDepositError,
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
		OrderError { order_id: U256, error_code: u16, account_id: U256 },
		/// Order of a user executed successfully
		OrderExecuted {
			account_id: U256,
			order_id: U256,
			market_id: u128,
			size: FixedI128,
			direction: u8,
			side: u8,
			order_type: u8,
			execution_price: FixedI128,
			pnl: FixedI128,
			fee: FixedI128,
			is_final: bool,
			is_maker: bool,
		},
		/// Force closure flag updation event
		ForceClosureFlagsChanged { account_id: U256, collateral_id: u128, force_closure_flag: u8 },
		/// Liquidator signer added
		LiquidatorSignerAdded { signer: U256 },
		/// Liquidator signer removed
		LiquidatorSignerRemoved { signer: U256 },
		/// Liquidation pnl for the system
		LiquidationPNL {
			account_id: U256,
			order_id: U256,
			market_id: u128,
			amount: FixedI128,
			block_number: BlockNumberFor<T>,
		},
		MasterFeeShareUpdated {
			master_account_address: U256,
			referral_account_address: U256,
			order_volume: FixedI128,
			collateral_id: u128,
			fee_share: FixedI128,
		},
	}

	// Pallet callable functions
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// External function to populate market to user map
		#[pallet::weight(0)]
		pub fn update_market_to_user_map(origin: OriginFor<T>, market_id: u128) -> DispatchResult {
			ensure_signed(origin)?;

			for (account_id, (market, direction), _) in PositionsMap::<T>::iter() {
				if market_id == market {
					MarketToAccountMap::<T>::set(
						(market_id, direction),
						account_id,
						Some((account_id, 0)),
					);
				}
			}

			Ok(())
		}

		/// External function to be called for trade execution
		#[pallet::weight(0)]
		pub fn execute_trade(
			origin: OriginFor<T>,
			batch_id: U256,
			quantity_locked: FixedI128,
			market_id: u128,
			oracle_price: FixedI128,
			orders: Vec<Order>,
			batch_timestamp: u64,
		) -> DispatchResult {
			// Make sure the caller is from a signed origin
			ensure_signed(origin)?;

			ensure!(!BatchStatusMap::<T>::contains_key(batch_id), Error::<T>::TradeBatchError525);

			// Get current timestamp
			let current_timestamp: u64 = T::TimeProvider::now().as_secs();
			// Get matching timelimit
			let matching_time_limit = MatchingTimeLimit::<T>::get();
			// Converting timestamp in milliseconds to seconds
			let batch_timestamp = batch_timestamp / 1000;
			let timestamp_limit = current_timestamp - batch_timestamp;
			// Check whether the batch is older than expected time
			ensure!(timestamp_limit <= matching_time_limit, Error::<T>::TradeBatchError545);

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

			//Update last traded price
			let last_traded_price = T::PricesPallet::get_last_oracle_price(market_id);
			if last_traded_price == FixedI128::zero() {
				T::PricesPallet::update_last_oracle_price(market_id, oracle_price);
			}

			let collateral_id: u128 = market.asset_collateral;
			let initial_taker_locked_quantity: FixedI128;

			ensure!(quantity_locked > FixedI128::zero(), Error::<T>::TradeBatchError522);
			ensure!(
				quantity_locked.into_inner() % market.step_size.into_inner() == 0_i128,
				Error::<T>::TradeBatchError503
			);

			ensure!(orders.len() > 1, Error::<T>::TradeBatchError547);

			// Calculate quantity that can be executed for the taker, before starting with the maker
			// orders
			// the unwrap won't fail as we are checking it in the previous line
			let taker_order = &orders.last().unwrap();
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

			let market_fees: BaseFeeAggregate =
				T::TradingFeesPallet::get_all_fees(market_id, collateral_id);

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
			let mut min_timestamp: u64 = batch_timestamp;
			let mut maker_error_codes = Vec::<u16>::new();

			for (index, element) in orders.iter().enumerate() {
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
				let order_pnl: FixedI128;
				let new_realized_pnl: FixedI128;
				let fee: FixedI128;
				let order_side: OrderSide;
				let mut created_timestamp: u64 = current_timestamp;

				let validation_response = Self::perform_validations(
					element,
					oracle_price,
					&market,
					collateral_id,
					current_timestamp,
				);
				match validation_response {
					Ok(()) => (),
					Err(e) => {
						// if maker order, emit event and process next order
						if element.order_id != taker_order.order_id {
							if taker_order.time_in_force == TimeInForce::IOC ||
								taker_order.time_in_force == TimeInForce::FOK
							{
								return Err(Error::<T>::IocFokError(
									index as u8,
									Self::get_error_code(&e),
								)
								.into())
							} else {
								Self::handle_maker_error(
									element.order_id,
									element.account_id,
									e,
									&mut maker_error_codes,
								);
								continue
							}
						} else {
							// if taker order, revert with error
							return Err(e.into())
						}
					},
				}

				let (order_portion_executed, _) = OrderStateMap::<T>::get(element.order_id);
				let position_details =
					PositionsMap::<T>::get(&element.account_id, (market_id, element.direction));
				let current_margin_locked =
					T::TradingAccountPallet::get_locked_margin(element.account_id, collateral_id);

				// Maker Order
				if element.order_id != taker_order.order_id {
					let validation_response = Self::validate_maker(
						orders[0].direction,
						orders[0].side,
						element.direction,
						element.side,
						element.order_type,
						element.price,
						oracle_price,
						&taker_order,
						tick_precision,
					);
					match validation_response {
						Ok(()) => (),
						Err(e) => {
							if taker_order.time_in_force == TimeInForce::IOC ||
								taker_order.time_in_force == TimeInForce::FOK
							{
								return Err(Error::<T>::IocFokError(
									index as u8,
									Self::get_error_code(&e),
								)
								.into())
							} else {
								Self::handle_maker_error(
									element.order_id,
									element.account_id,
									e,
									&mut maker_error_codes,
								);
								continue
							}
						},
					}
					// Calculate quantity left to be executed
					let quantity_remaining = initial_taker_locked_quantity - quantity_executed;

					// Calculate quantity that needs to be executed for the current maker
					let maker_quantity_to_execute_response = Self::calculate_quantity_to_execute(
						order_portion_executed,
						collateral_id,
						&position_details,
						element,
						quantity_remaining,
					);
					match maker_quantity_to_execute_response {
						Ok(quantity) => quantity_to_execute = quantity,
						Err(e) => {
							if taker_order.time_in_force == TimeInForce::IOC ||
								taker_order.time_in_force == TimeInForce::FOK
							{
								return Err(Error::<T>::IocFokError(
									index as u8,
									Self::get_error_code(&e),
								)
								.into())
							} else {
								Self::handle_maker_error(
									element.order_id,
									element.account_id,
									e,
									&mut maker_error_codes,
								);
								continue
							}
						},
					}

					// For a maker execution price will always be the price in its order object
					execution_price = element.price;

					order_side = OrderSide::Maker;
				} else {
					// Taker Order
					let validation_response = Self::validate_taker(
						orders[0].direction,
						orders[0].side,
						element.direction,
						element.side,
						element.post_only,
						element.order_type,
						element.slippage,
					);
					match validation_response {
						Ok(()) => (),
						Err(e) => return Err(e.into()),
					}

					// Taker quantity to be executed will be sum of maker quantities executed
					quantity_to_execute = quantity_executed;
					if quantity_to_execute == FixedI128::zero() {
						// If all makers failed due to slippage error, it means that
						// no orders can be currently matched with taker from OB
						// So revert with 514
						let are_all_slippage_errors =
							Self::are_all_errors_same(&maker_error_codes, 506);
						if are_all_slippage_errors {
							ensure!(false, Error::<T>::TradeBatchError514);
						}
						Self::deposit_event(Event::TradeExecutionFailed { batch_id });
						return Ok(())
					}

					// Handle FoK order
					if element.time_in_force == TimeInForce::FOK {
						ensure!(
							quantity_to_execute == element.size,
							Error::<T>::TradeBatchError516
						);
					}

					// Calculate execution price for taker
					execution_price = total_order_volume / quantity_to_execute;

					order_side = OrderSide::Taker;

					taker_execution_price = execution_price;
					taker_quantity = quantity_to_execute;
				}

				new_portion_executed = order_portion_executed + quantity_to_execute;

				let mut is_final: bool;
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
						collateral_token_decimal,
						&position_details,
						&market_fees,
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
							fee = trading_fee;
						},
						Err(e) => {
							// if maker order, emit event and process next order
							if element.order_id != taker_order.order_id {
								if taker_order.time_in_force == TimeInForce::IOC ||
									taker_order.time_in_force == TimeInForce::FOK
								{
									return Err(Error::<T>::IocFokError(
										index as u8,
										Self::get_error_code(&e),
									)
									.into())
								} else {
									Self::handle_maker_error(
										element.order_id,
										element.account_id,
										e,
										&mut maker_error_codes,
									);
									continue
								}
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
					new_realized_pnl = position_details.realized_pnl - fee;
					order_pnl = FixedI128::zero() - fee;

					// If the user previously does not have any position in this market
					// then add the market to CollateralToMarketMap
					// Also add the entry in MarketToUser if user is opening a new position
					// for a new market and direction
					if position_details.size == FixedI128::zero() {
						// Previous position size = 0 means that user does not have a position
						// in this market and direction
						MarketToAccountMap::<T>::insert(
							(market_id, element.direction),
							element.account_id,
							(element.account_id, 0),
						);

						let opposite_direction = Self::get_opposite_direction(element.direction);
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
					} else {
						created_timestamp = position_details.created_timestamp;
					}

					updated_position = Position {
						market_id,
						direction: element.direction,
						avg_execution_price,
						size: new_position_size,
						margin_amount,
						borrowed_amount,
						leverage: new_leverage,
						created_timestamp,
						modified_timestamp: current_timestamp,
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

					// For taker order, is_final should be true if not all the makers failed
					// but whatever makers failed, the reason is slippage
					// If all makers failed with slippage error, flow will not reach here
					if element.order_id == taker_order.order_id {
						let are_all_slippage_errors =
							Self::are_all_errors_same(&maker_error_codes, 506);
						if are_all_slippage_errors {
							is_final = true;
						}
					}
				} else {
					// SELL order
					let response = Self::process_close_orders(
						element,
						quantity_to_execute,
						order_side,
						execution_price,
						collateral_id,
						collateral_token_decimal,
						&position_details,
						&market_fees,
					);
					match response {
						Ok((
							margin,
							borrowed,
							average_execution,
							_balance,
							margin_lock,
							current_pnl,
							trading_fee,
						)) => {
							margin_amount = margin;
							borrowed_amount = borrowed;
							avg_execution_price = average_execution;
							margin_lock_amount = margin_lock;
							realized_pnl = current_pnl;
							fee = trading_fee;
						},
						Err(e) => {
							// if maker order, emit event and process next order
							if element.order_id != taker_order.order_id {
								if taker_order.time_in_force == TimeInForce::IOC ||
									taker_order.time_in_force == TimeInForce::FOK
								{
									return Err(Error::<T>::IocFokError(
										index as u8,
										Self::get_error_code(&e),
									)
									.into())
								} else {
									Self::handle_maker_error(
										element.order_id,
										element.account_id,
										e,
										&mut maker_error_codes,
									);
									continue
								}
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
						let deleveragable_amount =
							DeleveragableMap::<T>::get(element.account_id, collateral_id);
						let new_deleverage_amount = deleveragable_amount - quantity_to_execute;

						if new_deleverage_amount == FixedI128::zero() {
							DeleveragableMap::<T>::remove(element.account_id, collateral_id);

							// Remove the liquidation flag and check for deferred deposits
							Self::reset_force_closure_flags(element.account_id, collateral_id)?;
						} else {
							DeleveragableMap::<T>::insert(
								element.account_id,
								collateral_id,
								new_deleverage_amount,
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
					order_pnl = realized_pnl - fee;

					// If the user does not have any position in this market
					// then remove the market from CollateralToMarketMap
					// Remove the user from MarketToAccountMap also if position is
					// completely closed in a market and direction combination
					if new_position_size == FixedI128::zero() {
						// If the new position size is 0, it means that the position in a certain
						// market and direction is completely closed
						MarketToAccountMap::<T>::remove(
							(market_id, element.direction),
							element.account_id,
						);

						let opposite_direction = Self::get_opposite_direction(element.direction);
						let opposite_position = PositionsMap::<T>::get(
							&element.account_id,
							(market_id, opposite_direction),
						);
						if opposite_position.size == FixedI128::zero() {
							let mut markets =
								CollateralToMarketMap::<T>::get(&element.account_id, collateral_id);

							markets.retain(|&market| market != market_id);

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
						created_timestamp = position_details.created_timestamp;
						// To do - Calculate pnl
						updated_position = Position {
							market_id,
							direction: element.direction,
							avg_execution_price,
							size: new_position_size,
							margin_amount,
							borrowed_amount,
							leverage: new_leverage,
							created_timestamp,
							modified_timestamp: current_timestamp,
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

					// For taker order, is_final should be true if not all the makers failed
					// but whatever makers failed, the reason is slippage
					// If all makers failed with slippage error, flow will not reach here
					if element.order_id == taker_order.order_id {
						let are_all_slippage_errors =
							Self::are_all_errors_same(&maker_error_codes, 506);
						if are_all_slippage_errors {
							is_final = true;
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

				// Update quantity_executed and total_order_volume
				quantity_executed = quantity_executed + quantity_to_execute;
				total_order_volume = total_order_volume + (element.price * quantity_to_execute);

				// Update locked margin and portion executed
				T::TradingAccountPallet::set_locked_margin(
					element.account_id,
					collateral_id,
					new_margin_locked,
				);
				OrderStateMap::<T>::insert(element.order_id, (new_portion_executed, false));

				BatchStatusMap::<T>::insert(batch_id, true);

				// Add order_id to timestamp map
				if order_portion_executed == FixedI128::zero() {
					// Convert timestamp from milliseconds to seconds
					let order_timestamp = element.timestamp / 1000;
					let orders_by_timestamp = OrdersMap::<T>::get(order_timestamp);
					let mut orders_list;
					if orders_by_timestamp.is_none() {
						orders_list = Vec::<U256>::new();
					} else {
						orders_list = orders_by_timestamp.unwrap();
					}
					orders_list.push(element.order_id);
					OrdersMap::<T>::insert(order_timestamp, orders_list);

					if order_timestamp < min_timestamp {
						min_timestamp = order_timestamp;
					}
				}

				// Store the trading fee
				let current_trading_fee = TradingFeeMap::<T>::get(collateral_id);
				TradingFeeMap::<T>::insert(collateral_id, current_trading_fee + fee);

				Self::deposit_event(Event::OrderExecuted {
					account_id: element.account_id,
					order_id: element.order_id,
					market_id: element.market_id,
					size: quantity_to_execute,
					direction: element.direction.into(),
					side: element.side.into(),
					order_type: element.order_type.into(),
					execution_price,
					pnl: order_pnl,
					fee,
					is_final,
					is_maker: element.order_id != taker_order.order_id,
				});
			}

			// Update open interest
			let actual_open_interest = open_interest;
			let current_open_interest = OpenInterestMap::<T>::get(market_id);
			OpenInterestMap::<T>::insert(market_id, current_open_interest + actual_open_interest);

			// Update initial margin locked
			InitialMarginMap::<T>::insert((market_id, Direction::Long), initial_margin_locked_long);
			InitialMarginMap::<T>::insert(
				(market_id, Direction::Short),
				initial_margin_locked_short,
			);

			BatchStatusMap::<T>::insert(batch_id, true);

			// Add batch_id to timestamp map
			let batches_by_timestamp = BatchesMap::<T>::get(batch_timestamp);
			let mut batches;
			if batches_by_timestamp.is_none() {
				batches = Vec::<U256>::new();
			} else {
				batches = batches_by_timestamp.unwrap();
			}
			batches.push(batch_id);
			BatchesMap::<T>::insert(batch_timestamp, batches);

			// Modify start timestamp
			let start_timestamp = StartTimestamp::<T>::get();
			if (start_timestamp.is_some() && min_timestamp < start_timestamp.unwrap()) ||
				start_timestamp.is_none()
			{
				StartTimestamp::<T>::put(min_timestamp);
			}

			// Emit trade executed event
			Self::deposit_event(Event::TradeExecuted {
				batch_id,
				market_id,
				size: taker_quantity,
				execution_price: taker_execution_price,
				direction: taker_order.direction.into(),
				side: taker_order.side.into(),
			});

			Ok(())
		}

		// TODO(merkle-groot): To add origin restriction in production
		#[pallet::weight(0)]
		pub fn add_liquidator_signer(origin: OriginFor<T>, pub_key: U256) -> DispatchResult {
			ensure_root(origin)?;

			// The pub key cannot be 0
			ensure!(pub_key != U256::zero(), Error::<T>::ZeroSigner);

			// Ensure that the pub_key is not already whitelisted
			ensure!(!IsLiquidatorSignerWhitelisted::<T>::get(pub_key), Error::<T>::DuplicateSigner);

			// Store the new signer
			Self::add_liquidator_signer_internal(pub_key);

			// Return ok
			Ok(())
		}

		// TODO(merkle-groot): To add origin restriction in production
		#[pallet::weight(0)]
		pub fn remove_liquidator_signer(origin: OriginFor<T>, pub_key: U256) -> DispatchResult {
			ensure_root(origin)?;

			// Check if the signer exists
			ensure!(
				IsLiquidatorSignerWhitelisted::<T>::get(pub_key),
				Error::<T>::SignerNotWhitelisted
			);

			// Update the state
			Self::remove_liquidator_signer_internal(pub_key);

			// Return ok
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn cancel_order(origin: OriginFor<T>, order_id: U256) -> DispatchResult {
			ensure_signed(origin)?;

			// TODO: Add signature verification

			// The order_id cannot be 0
			ensure!(order_id != U256::zero(), Error::<T>::ZeroOrderId);

			let (order_portion_executed, _) = OrderStateMap::<T>::get(order_id);

			// Mark the order as cancelled order
			OrderStateMap::<T>::insert(order_id, (order_portion_executed, true));

			// Return ok
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn perform_cleanup(origin: OriginFor<T>) -> DispatchResult {
			// Make sure the caller is from a signed origin
			ensure_signed(origin)?;

			let start_timestamp =
				StartTimestamp::<T>::get().ok_or(Error::<T>::StartTimestampEmpty)?;
			let current_timestamp: u64 = T::TimeProvider::now().as_secs();
			let timestamp_limit = current_timestamp - FOUR_WEEKS;
			let mut cleanup_count = CLEANUP_COUNT;

			for timestamp in start_timestamp..timestamp_limit {
				if cleanup_count == 0 {
					StartTimestamp::<T>::put(timestamp);
					return Ok(())
				}
				let batches = BatchesMap::<T>::get(timestamp);
				if batches.is_some() {
					for batch in batches.unwrap() {
						BatchStatusMap::<T>::remove(batch);
					}
					BatchesMap::<T>::remove(timestamp);
				}

				let orders = OrdersMap::<T>::get(timestamp);
				if orders.is_some() {
					for order in orders.unwrap() {
						OrderStateMap::<T>::remove(order);
						OrderHashMap::<T>::remove(order);
					}
					OrdersMap::<T>::remove(timestamp);
				}
				cleanup_count -= 1;
			}
			if start_timestamp < timestamp_limit {
				StartTimestamp::<T>::put(timestamp_limit);
			}

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn set_matching_time_limit(origin: OriginFor<T>, time_limit: u64) -> DispatchResult {
			// Make sure the caller is a sudo user
			ensure_root(origin)?;
			MatchingTimeLimit::<T>::put(time_limit);
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn close_delisted_market_positions(
			origin: OriginFor<T>,
			market_id: u128,
		) -> DispatchResult {
			// Make sure the caller is from a signed origin
			ensure_signed(origin)?;

			// Validate market
			let market = T::MarketPallet::get_market(market_id);
			ensure!(market.is_some(), Error::<T>::TradeBatchError509);
			let market = market.unwrap();
			// Market should be non tradable
			ensure!(market.is_tradable == false, Error::<T>::TradeBatchError509);

			// Get collateral_token_decimal
			let collateral_asset = T::AssetPallet::get_asset(market.asset_collateral).unwrap();
			let collateral_token_decimal = collateral_asset.decimals;

			// Get collateral id
			let collateral_id: u128 = market.asset_collateral;

			// Get all fees
			let market_fees: BaseFeeAggregate =
				T::TradingFeesPallet::get_all_fees(market_id, collateral_id);

			// Fetch execution price
			let execution_price = match T::PricesPallet::get_mark_price_for_ads(market_id) {
				Some(price) => price,
				None => return Err(Error::<T>::ADSPriceNotAvailable.into()),
			};

			let mut initial_margin_locked_long: FixedI128 =
				InitialMarginMap::<T>::get((market_id, Direction::Long));
			let mut initial_margin_locked_short: FixedI128 =
				InitialMarginMap::<T>::get((market_id, Direction::Short));
			let mut current_open_interest = OpenInterestMap::<T>::get(market_id);
			let mut current_trading_fee = TradingFeeMap::<T>::get(collateral_id);
			let mut current_liquidation_fee = LiquidationFeeMap::<T>::get(collateral_id);

			let mut positions_close_count = POSITIONS_CLOSE_COUNT;
			let long_users_count =
				MarketToAccountMap::<T>::iter_prefix_values((market_id, Direction::Long)).count();

			if long_users_count != 0 {
				// Iterate through all long users who have open positions in a delisted market
				for (long_user, _) in
					MarketToAccountMap::<T>::iter_prefix_values((market_id, Direction::Long))
				{
					if positions_close_count == 0 {
						break;
					}
					let position_details =
						PositionsMap::<T>::get(&long_user, (market_id, Direction::Long));
					let price_diff: FixedI128 =
						execution_price - position_details.avg_execution_price;
					let fee: FixedI128;

					let response = Self::close_position(
						long_user,
						market_id,
						Direction::Long,
						collateral_id,
						collateral_token_decimal,
						&position_details,
						price_diff,
						execution_price,
						&market_fees,
						&mut current_liquidation_fee,
					);
					match response {
						Ok(trading_fee) => {
							fee = trading_fee;
						},
						Err(e) => return Err(e.into()),
					}
					initial_margin_locked_long =
						initial_margin_locked_long - position_details.margin_amount;
					current_open_interest = current_open_interest - position_details.size;
					current_trading_fee = current_trading_fee + fee;

					positions_close_count -= 1;
				}
			}

			let short_users_count =
				MarketToAccountMap::<T>::iter_prefix_values((market_id, Direction::Short)).count();

			if short_users_count != 0 && positions_close_count != 0 {
				// Iterate through all short users who have open positions in a delisted market
				for (short_user, _) in
					MarketToAccountMap::<T>::iter_prefix_values((market_id, Direction::Short))
				{
					if positions_close_count == 0 {
						break;
					}
					let position_details =
						PositionsMap::<T>::get(&short_user, (market_id, Direction::Short));
					let price_diff: FixedI128 =
						position_details.avg_execution_price - execution_price;
					let fee: FixedI128;

					let response = Self::close_position(
						short_user,
						market_id,
						Direction::Short,
						collateral_id,
						collateral_token_decimal,
						&position_details,
						price_diff,
						execution_price,
						&market_fees,
						&mut current_liquidation_fee,
					);
					match response {
						Ok(trading_fee) => {
							fee = trading_fee;
						},
						Err(e) => return Err(e.into()),
					}
					initial_margin_locked_short =
						initial_margin_locked_short - position_details.margin_amount;
					current_open_interest = current_open_interest - position_details.size;
					current_trading_fee = current_trading_fee + fee;

					positions_close_count -= 1;
				}
			}

			// Update trading fee for a collateral
			TradingFeeMap::<T>::insert(collateral_id, current_trading_fee);

			// Update open interest for a market
			OpenInterestMap::<T>::insert(market_id, current_open_interest);

			// Update initial margin locked for a market with Long direction
			InitialMarginMap::<T>::insert((market_id, Direction::Long), initial_margin_locked_long);

			// Update initial margin locked for a market with Short direction
			InitialMarginMap::<T>::insert(
				(market_id, Direction::Short),
				initial_margin_locked_short,
			);

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
			let (order_portion_executed, _) = OrderStateMap::<T>::get(order.order_id);

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

					// If order type is Forced, force closure flag will always be
					// one of Deleverage or Liquidate
					match force_closure_flag.unwrap() {
						ForceClosureFlag::Deleverage => {
							let deleveragable_amount =
								DeleveragableMap::<T>::get(&order.account_id, collateral_id);
							quantity_to_execute =
								FixedI128::min(quantity_to_execute, deleveragable_amount);
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
			current_timestamp: u64,
		) -> Result<(), Error<T>> {
			// Validate that the user is registered
			let is_registered = T::TradingAccountPallet::is_registered_user(order.account_id);
			ensure!(is_registered, Error::<T>::TradeBatchError510);

			// Check whether the order is a cancelled order
			let (_, is_cancelled) = OrderStateMap::<T>::get(order.order_id);
			ensure!(!is_cancelled, Error::<T>::TradeBatchError543);

			// Check whether the order is older than 4 weeks
			let timestamp_limit = current_timestamp - FOUR_WEEKS;
			ensure!(order.timestamp >= timestamp_limit, Error::<T>::TradeBatchError544);

			// Validate that if force closure flag is set
			// order type can only be 'Forced'
			if order.order_type != OrderType::Forced {
				let force_closure_flag =
					ForceClosureFlagMap::<T>::get(order.account_id, collateral_id);
				ensure!(force_closure_flag.is_none(), Error::<T>::TradeBatchError539);
			}

			// Validate that size of BUY order is >= min quantity for market
			// And If the order is SELL order size should be > 0
			if order.side == Side::Buy {
				ensure!(order.size >= market.minimum_order_size, Error::<T>::TradeBatchError505);
			} else {
				ensure!(order.size > FixedI128::zero(), Error::<T>::TradeBatchError505);
			}

			// Validate the size of an order is multiple of step size
			ensure!(
				order.size.into_inner() % market.step_size.into_inner() == 0_i128,
				Error::<T>::TradeBatchError517
			);

			// Validate that market matched and market in order are same
			ensure!(market.id == order.market_id, Error::<T>::TradeBatchError504);

			// Validate leverage value
			ensure!(
				order.leverage >= LEVERAGE_ONE &&
					order.leverage <= market.currently_allowed_leverage,
				Error::<T>::TradeBatchError502
			);

			Self::validate_signature(&order)?;

			Ok(())
		}

		fn validate_signature(order: &Order) -> Result<(), Error<T>> {
			let SignatureInfo { liquidator_pub_key, hash_type, sig_r, sig_s } =
				&order.signature_info;

			// Hash the order
			let order_hash = order.hash(hash_type).map_err(|_| Error::<T>::TradeBatchError534)?;

			// Convert to FieldElement
			let (sig_r_felt, sig_s_felt) =
				sig_u256_to_sig_felt(sig_r, sig_s).map_err(|_| Error::<T>::TradeBatchError535)?;

			let sig = Signature { r: sig_r_felt, s: sig_s_felt };

			let verification_result = match order.order_type {
				OrderType::Forced => {
					ensure!(
						IsLiquidatorSignerWhitelisted::<T>::get(liquidator_pub_key),
						Error::<T>::TradeBatchError542
					);
					let liquidator_pub_key_felt = liquidator_pub_key
						.try_to_felt()
						.map_err(|_| Error::<T>::TradeBatchError538)?;

					ecdsa_verify(&liquidator_pub_key_felt, &order_hash, &sig)
				},
				_ => {
					let public_key_felt =
						T::TradingAccountPallet::get_public_key(&order.account_id)
							.and_then(|key| key.try_to_felt().ok())
							.ok_or(Error::<T>::TradeBatchError538)?;

					ecdsa_verify(&public_key_felt, &order_hash, &sig)
				},
			};

			// Signature verification returned error or false
			ensure!(
				verification_result.is_ok() && verification_result.unwrap(),
				Error::<T>::TradeBatchError536
			);

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
			maker_price: FixedI128,
			oracle_price: FixedI128,
			taker_order: &Order,
			tick_precision: u8,
		) -> Result<(), Error<T>> {
			let opposite_direction = Self::get_opposite_direction(maker1_direction);
			let opposite_side = Self::get_opposite_side(maker1_side);

			ensure!(
				(current_direction == maker1_direction && current_side == maker1_side) ||
					(current_direction == opposite_direction && current_side == opposite_side),
				Error::<T>::TradeBatchError512
			);

			ensure!(order_type == OrderType::Limit, Error::<T>::TradeBatchError518);

			if taker_order.order_type == OrderType::Limit {
				// Check whether the maker price is valid with respect to taker limit price
				Self::validate_limit_price(
					taker_order.price,
					maker_price,
					taker_order.direction,
					taker_order.side,
				)?;
			} else if taker_order.order_type == OrderType::Market {
				// Check whether the maker price is valid with respect to taker slippage
				Self::validate_within_slippage(
					taker_order.slippage,
					oracle_price,
					maker_price,
					taker_order.direction,
					taker_order.side,
					tick_precision,
				)?;
			}

			Ok(())
		}

		fn validate_taker(
			maker1_direction: Direction,
			maker1_side: Side,
			current_direction: Direction,
			current_side: Side,
			post_only: bool,
			order_type: OrderType,
			slippage: FixedI128,
		) -> Result<(), Error<T>> {
			if order_type == OrderType::Market {
				ensure!(
					slippage >= FixedI128::zero() &&
						slippage <= FixedI128::from_inner(150000000000000000),
					Error::<T>::TradeBatchError521
				);
			}

			let opposite_direction = Self::get_opposite_direction(maker1_direction);
			let opposite_side = Self::get_opposite_side(maker1_side);

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
			tick_precision: u8,
		) -> Result<(), Error<T>> {
			let mut threshold = slippage * oracle_price;
			threshold = threshold.round_to_precision(tick_precision.into());

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
			collateral_token_decimal: u8,
			position_details: &Position,
			market_fees: &BaseFeeAggregate,
		) -> Result<(FixedI128, FixedI128, FixedI128, FixedI128, FixedI128, FixedI128), Error<T>> {
			let margin_amount: FixedI128;
			let borrowed_amount: FixedI128;
			let average_execution_price: FixedI128;

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

			// Get the market details
			let market = T::MarketPallet::get_market(market_id).unwrap();
			ensure!(
				position_details.size + order_size <= market.maximum_position_size,
				Error::<T>::TradeBatchError548
			);

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

			let current_volume =
				(order_size * execution_price).round_to_precision(collateral_token_decimal.into());
			let (total_30day_volume, master_30day_volume) =
				T::TradingAccountPallet::update_and_get_user_and_master_volume(
					order.account_id,
					order.market_id,
					current_volume,
				)
				.or_else(|_| Err(Error::<T>::TradeBatchError546))?;

			let (fee_rate, _) = Self::get_fee_rate(
				order.account_id,
				&market_fees,
				Side::Buy,
				order_side,
				total_30day_volume,
			);

			let mut fee = fee_rate * current_volume;
			fee = fee.round_to_precision(collateral_token_decimal.into());

			let fee_share_amount = Self::update_and_get_fee_share(
				order.account_id,
				collateral_id,
				master_30day_volume,
				current_volume,
				fee,
				collateral_token_decimal,
			);

			ensure!(fee <= available_margin, Error::<T>::TradeBatchError501);
			if fee != FixedI128::zero() {
				T::TradingAccountPallet::transfer_from(
					order.account_id,
					collateral_id,
					order.market_id,
					fee,
					BalanceChangeReason::Fee,
				);

				T::TradingAccountPallet::handle_fee_split(
					order.account_id,
					collateral_id,
					order.market_id,
					fee,
					fee_share_amount,
				);
			}

			Ok((
				margin_amount,
				borrowed_amount,
				average_execution_price,
				available_margin,
				margin_order_value,
				fee,
			))
		}

		fn process_close_orders(
			order: &Order,
			order_size: FixedI128,
			order_side: OrderSide,
			execution_price: FixedI128,
			collateral_id: u128,
			collateral_token_decimal: u8,
			position_details: &Position,
			market_fees: &BaseFeeAggregate,
		) -> Result<
			(FixedI128, FixedI128, FixedI128, FixedI128, FixedI128, FixedI128, FixedI128),
			Error<T>,
		> {
			let actual_execution_price: FixedI128;
			let price_diff: FixedI128;

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
			let mut borrowed_amount_to_return =
				position_details.borrowed_amount * ratio_of_position;
			borrowed_amount_to_return =
				borrowed_amount_to_return.round_to_precision(collateral_token_decimal.into());
			let mut margin_amount_to_reduce = position_details.margin_amount * ratio_of_position;
			margin_amount_to_reduce =
				margin_amount_to_reduce.round_to_precision(collateral_token_decimal.into());

			// Calculate pnl
			let mut pnl = order_size * price_diff;
			pnl = pnl.round_to_precision(collateral_token_decimal.into());
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

			let balance = T::TradingAccountPallet::get_balance(order.account_id, collateral_id);

			// Get the Liquidation fee
			let current_liquidation_fee = LiquidationFeeMap::<T>::get(collateral_id);

			// If the order type is Liquidate or Deleverage
			if order.order_type == OrderType::Forced {
				// Check if user is under water, ie,
				// user has lost some borrowed funds
				if margin_plus_pnl.is_negative() {
					let pnl_abs = pnl.saturating_abs();

					// Check if user's balance can cover the deficit
					if pnl_abs > balance {
						if balance.is_negative() {
							// Complete funds lost by user should be taken from insurance fund
							T::TradingAccountPallet::handle_insurance_fund_update(
								collateral_id,
								order.market_id,
								pnl_abs,
								FundModifyType::Decrease,
							);
							LiquidationFeeMap::<T>::insert(
								collateral_id,
								current_liquidation_fee - pnl_abs,
							);
							Self::deposit_event(Event::LiquidationPNL {
								account_id: order.account_id,
								order_id: order.order_id,
								market_id: order.market_id,
								amount: pnl,
								block_number: <frame_system::Pallet<T>>::block_number(),
							});
						} else {
							// Some amount of lost funds can be taken from user available balance
							// Rest of the funds should be taken from insurance fund
							T::TradingAccountPallet::handle_insurance_fund_update(
								collateral_id,
								order.market_id,
								pnl_abs - balance,
								FundModifyType::Decrease,
							);

							LiquidationFeeMap::<T>::insert(
								collateral_id,
								current_liquidation_fee - (pnl_abs - balance),
							);
							Self::deposit_event(Event::LiquidationPNL {
								account_id: order.account_id,
								order_id: order.order_id,
								market_id: order.market_id,
								amount: balance + pnl,
								block_number: <frame_system::Pallet<T>>::block_number(),
							});
						}
					}

					// Deduct loss from user
					T::TradingAccountPallet::transfer_from(
						order.account_id,
						collateral_id,
						order.market_id,
						pnl.saturating_abs(),
						BalanceChangeReason::PnlRealization,
					);
				} else {
					// User is not underwater, order type is Liquidate or Deleverage
					let force_closure_flag =
						ForceClosureFlagMap::<T>::get(order.account_id, collateral_id);

					// If order type is Forced, force closure flag will always be
					// one of Deleverage or Liquidate
					match force_closure_flag.unwrap() {
						// Liquidation case when user is not underwater
						ForceClosureFlag::Liquidate => {
							// if balance >= margin amount, deposit remaining margin in insurance
							if margin_amount_to_reduce <= balance {
								// Deposit margin_plus_pnl to insurance fund
								T::TradingAccountPallet::handle_insurance_fund_update(
									collateral_id,
									order.market_id,
									margin_plus_pnl,
									FundModifyType::Increase,
								);
								LiquidationFeeMap::<T>::insert(
									collateral_id,
									current_liquidation_fee + margin_plus_pnl,
								);

								Self::deposit_event(Event::LiquidationPNL {
									account_id: order.account_id,
									order_id: order.order_id,
									market_id: order.market_id,
									amount: margin_plus_pnl,
									block_number: <frame_system::Pallet<T>>::block_number(),
								});
							} else {
								if balance.is_negative() {
									// Deduct margin_amount_to_reduce from insurance fund
									T::TradingAccountPallet::handle_insurance_fund_update(
										collateral_id,
										order.market_id,
										margin_amount_to_reduce,
										FundModifyType::Decrease,
									);

									LiquidationFeeMap::<T>::insert(
										collateral_id,
										current_liquidation_fee - margin_amount_to_reduce,
									);

									Self::deposit_event(Event::LiquidationPNL {
										account_id: order.account_id,
										order_id: order.order_id,
										market_id: order.market_id,
										amount: -margin_amount_to_reduce,
										block_number: <frame_system::Pallet<T>>::block_number(),
									});
								} else {
									// if user has some balance
									let pnl_abs = pnl.saturating_abs();
									if balance <= pnl_abs {
										// Deduct (pnl_abs -  balance) from insurance fund
										T::TradingAccountPallet::handle_insurance_fund_update(
											collateral_id,
											order.market_id,
											pnl_abs - balance,
											FundModifyType::Decrease,
										);

										LiquidationFeeMap::<T>::insert(
											collateral_id,
											current_liquidation_fee - (pnl_abs - balance),
										);

										Self::deposit_event(Event::LiquidationPNL {
											account_id: order.account_id,
											order_id: order.order_id,
											market_id: order.market_id,
											amount: balance + pnl,
											block_number: <frame_system::Pallet<T>>::block_number(),
										});
									} else {
										// Deposit (balance - pnl_abs) to insurance fund
										T::TradingAccountPallet::handle_insurance_fund_update(
											collateral_id,
											order.market_id,
											balance - pnl_abs,
											FundModifyType::Increase,
										);
										LiquidationFeeMap::<T>::insert(
											collateral_id,
											current_liquidation_fee + (balance - pnl_abs),
										);

										Self::deposit_event(Event::LiquidationPNL {
											account_id: order.account_id,
											order_id: order.order_id,
											market_id: order.market_id,
											amount: balance + pnl,
											block_number: <frame_system::Pallet<T>>::block_number(),
										});
									}
								}
							}
							// Deduct proportionate margin amount from user
							T::TradingAccountPallet::transfer_from(
								order.account_id,
								collateral_id,
								order.market_id,
								margin_amount_to_reduce,
								BalanceChangeReason::Liquidation,
							);
						},
						ForceClosureFlag::Deleverage => {
							pnl = FixedI128::zero();
						},
					}
				}
			} else {
				// User is in loss and order type is Market or Limit
				if pnl.is_negative() {
					// Loss cannot be covered by the user
					if pnl.saturating_abs() > balance {
						ensure!(false, Error::<T>::TradeBatchError532);
					} else {
						// Deduct required funds from user
						T::TradingAccountPallet::transfer_from(
							order.account_id,
							collateral_id,
							order.market_id,
							pnl.saturating_abs(),
							BalanceChangeReason::PnlRealization,
						);
					}
				} else {
					// User is in profit
					// Transfer the profit to user
					T::TradingAccountPallet::transfer(
						order.account_id,
						collateral_id,
						order.market_id,
						pnl,
						BalanceChangeReason::PnlRealization,
					);
				}
			}

			let current_volume =
				(order_size * execution_price).round_to_precision(collateral_token_decimal.into());
			let (total_30day_volume, master_30day_volume) =
				T::TradingAccountPallet::update_and_get_user_and_master_volume(
					order.account_id,
					order.market_id,
					current_volume,
				)
				.or_else(|_| Err(Error::<T>::TradeBatchError546))?;

			let fee = if order.order_type != OrderType::Forced {
				let (fee_rate, _) = Self::get_fee_rate(
					order.account_id,
					&market_fees,
					Side::Sell,
					order_side,
					total_30day_volume,
				);

				let mut fee = fee_rate * current_volume;
				fee = fee.round_to_precision(collateral_token_decimal.into());

				let fee_share_amount = Self::update_and_get_fee_share(
					order.account_id,
					collateral_id,
					master_30day_volume,
					current_volume,
					fee,
					collateral_token_decimal,
				);

				// Deduct fee while closing a position
				if fee != FixedI128::zero() {
					// Transfer fee from the user
					T::TradingAccountPallet::transfer_from(
						order.account_id,
						collateral_id,
						order.market_id,
						fee,
						BalanceChangeReason::Fee,
					);

					T::TradingAccountPallet::handle_fee_split(
						order.account_id,
						collateral_id,
						order.market_id,
						fee,
						fee_share_amount,
					);
				}

				fee
			} else {
				FixedI128::zero()
			};

			Ok((
				margin_amount,
				borrowed_amount,
				position_details.avg_execution_price,
				balance,
				margin_amount_to_reduce,
				pnl,
				fee,
			))
		}

		fn update_and_get_fee_share(
			account_id: U256,
			collateral_id: u128,
			master_30day_volume: FixedI128,
			order_volume: FixedI128,
			fee: FixedI128,
			collateral_token_decimal: u8,
		) -> FixedI128 {
			let mut fee_share = FixedI128::zero();
			if let Some(referral_details) =
				T::TradingAccountPallet::get_account_address_and_referral_details(account_id)
			{
				let account_level = T::TradingAccountPallet::get_master_account_level(
					referral_details.master_account_address,
				);
				let fee_share_rate = T::TradingFeesPallet::get_fee_share(
					account_level,
					collateral_id,
					master_30day_volume,
				);
				fee_share =
					(fee * fee_share_rate).round_to_precision(collateral_token_decimal.into());
				T::TradingAccountPallet::update_master_fee_share(
					referral_details.master_account_address,
					collateral_id,
					fee_share,
				);

				// This unwrap won't fail since user registration was checked
				let referral_account_address =
					T::TradingAccountPallet::get_account(&account_id).unwrap().account_address;
				Self::deposit_event(Event::MasterFeeShareUpdated {
					master_account_address: referral_details.master_account_address,
					referral_account_address,
					order_volume,
					collateral_id,
					fee_share,
				});
			}
			return fee_share;
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

			let mark_price = T::PricesPallet::get_mark_price(market_id);

			(maintenance_requirement, mark_price)
		}

		fn get_error_code(error: &Error<T>) -> u16 {
			match &error {
				Error::<T>::TradeBatchError501 => 501,
				Error::<T>::TradeBatchError502 => 502,
				Error::<T>::TradeBatchError503 => 503,
				Error::<T>::TradeBatchError504 => 504,
				Error::<T>::TradeBatchError505 => 505,
				Error::<T>::TradeBatchError506 => 506,
				Error::<T>::TradeBatchError507 => 507,
				Error::<T>::TradeBatchError508 => 508,
				Error::<T>::TradeBatchError509 => 509,
				Error::<T>::TradeBatchError510 => 510,
				Error::<T>::TradeBatchError511 => 511,
				Error::<T>::TradeBatchError512 => 512,
				Error::<T>::TradeBatchError513 => 513,
				Error::<T>::TradeBatchError514 => 514,
				Error::<T>::TradeBatchError515 => 515,
				Error::<T>::TradeBatchError516 => 516,
				Error::<T>::TradeBatchError517 => 517,
				Error::<T>::TradeBatchError518 => 518,
				Error::<T>::TradeBatchError521 => 521,
				Error::<T>::TradeBatchError522 => 522,
				Error::<T>::TradeBatchError523 => 523,
				Error::<T>::TradeBatchError524 => 524,
				Error::<T>::TradeBatchError525 => 525,
				Error::<T>::TradeBatchError531 => 531,
				Error::<T>::TradeBatchError532 => 532,
				Error::<T>::TradeBatchError533 => 533,
				Error::<T>::TradeBatchError534 => 534,
				Error::<T>::TradeBatchError535 => 535,
				Error::<T>::TradeBatchError536 => 536,
				Error::<T>::TradeBatchError538 => 538,
				Error::<T>::TradeBatchError539 => 539,
				Error::<T>::TradeBatchError540 => 540,
				Error::<T>::TradeBatchError541 => 541,
				Error::<T>::TradeBatchError542 => 542,
				Error::<T>::TradeBatchError543 => 543,
				Error::<T>::TradeBatchError544 => 544,
				Error::<T>::TradeBatchError545 => 545,
				Error::<T>::TradeBatchError546 => 546,
				Error::<T>::TradeBatchError547 => 547,
				Error::<T>::TradeBatchError548 => 548,
				_ => 500,
			}
		}

		fn add_liquidator_signer_internal(pub_key: U256) {
			// Store the new signer
			LiquidatorSigners::<T>::append(pub_key);
			IsLiquidatorSignerWhitelisted::<T>::insert(pub_key, true);

			// Emit the SignerAdded event
			Self::deposit_event(Event::LiquidatorSignerAdded { signer: pub_key });
		}

		fn remove_liquidator_signer_internal(pub_key: U256) {
			// Read the state of signers
			let signers_array = LiquidatorSigners::<T>::get();

			// remove the signer from the array
			let updated_array: Vec<U256> =
				signers_array.into_iter().filter(|&signer| signer != pub_key).collect();

			// Update the state
			IsLiquidatorSignerWhitelisted::<T>::insert(pub_key, false);
			LiquidatorSigners::<T>::put(updated_array);

			// Emit the SignerRemoved event
			Self::deposit_event(Event::LiquidatorSignerRemoved { signer: pub_key });
		}

		fn order_hash_check(order_id: U256, order_hash: U256) -> bool {
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

		fn are_all_errors_same(error_codes: &Vec<u16>, error_code: u16) -> bool {
			if error_codes.len() > 0 {
				for &code in error_codes {
					if code != error_code {
						return false
					}
				}
			} else {
				return false
			}
			true
		}

		fn get_opposite_direction(direction: Direction) -> Direction {
			if direction == Direction::Long {
				Direction::Short
			} else {
				Direction::Long
			}
		}

		fn get_opposite_side(side: Side) -> Side {
			if side == Side::Buy {
				Side::Sell
			} else {
				Side::Buy
			}
		}

		fn handle_maker_error(
			order_id: U256,
			account_id: U256,
			e: Error<T>,
			maker_error_codes: &mut Vec<u16>,
		) {
			Self::deposit_event(Event::OrderError {
				order_id,
				error_code: Self::get_error_code(&e),
				account_id,
			});
			maker_error_codes.push(Self::get_error_code(&e));
		}

		fn close_position(
			account_id: U256,
			market_id: u128,
			direction: Direction,
			collateral_id: u128,
			collateral_token_decimal: u8,
			position_details: &Position,
			price_diff: FixedI128,
			execution_price: FixedI128,
			market_fees: &BaseFeeAggregate,
			current_liquidation_fee: &mut FixedI128,
		) -> Result<FixedI128, Error<T>> {
			let order_size: FixedI128 = position_details.size;
			// Total value of asset at current price
			let leveraged_order_value = order_size * execution_price;

			// Calculate pnl
			let mut pnl = order_size * price_diff;
			pnl = pnl.round_to_precision(collateral_token_decimal.into());

			let balance = T::TradingAccountPallet::get_balance(account_id, collateral_id);

			// If user is in loss
			if pnl.is_negative() {
				let pnl_abs = pnl.saturating_abs();

				// If user's balance cannot cover the loss
				if pnl_abs > balance {
					if balance.is_negative() {
						// Complete funds lost by user should be taken from insurance fund
						T::TradingAccountPallet::handle_insurance_fund_update(
							collateral_id,
							market_id,
							pnl_abs,
							FundModifyType::Decrease,
						);
						LiquidationFeeMap::<T>::insert(
							collateral_id,
							*current_liquidation_fee - pnl_abs,
						);
						Self::deposit_event(Event::LiquidationPNL {
							account_id,
							order_id: U256::zero(),
							market_id,
							amount: pnl,
							block_number: <frame_system::Pallet<T>>::block_number(),
						});
					} else {
						// Some amount of lost funds can be taken from user available balance
						// Rest of the funds should be taken from insurance fund
						T::TradingAccountPallet::handle_insurance_fund_update(
							collateral_id,
							market_id,
							pnl_abs - balance,
							FundModifyType::Decrease,
						);

						LiquidationFeeMap::<T>::insert(
							collateral_id,
							*current_liquidation_fee - (pnl_abs - balance),
						);
						Self::deposit_event(Event::LiquidationPNL {
							account_id,
							order_id: U256::zero(),
							market_id,
							amount: balance + pnl,
							block_number: <frame_system::Pallet<T>>::block_number(),
						});
					}
				}
				// Deduct loss from user
				T::TradingAccountPallet::transfer_from(
					account_id,
					collateral_id,
					market_id,
					pnl_abs,
					BalanceChangeReason::PnlRealization,
				);
			} else {
				// User is in profit
				// Transfer the profit to user
				T::TradingAccountPallet::transfer(
					account_id,
					collateral_id,
					market_id,
					pnl,
					BalanceChangeReason::PnlRealization,
				);
			}

			let current_volume =
				(order_size * execution_price).round_to_precision(collateral_token_decimal.into());
			let (total_30day_volume, master_30day_volume) =
				T::TradingAccountPallet::update_and_get_user_and_master_volume(
					account_id,
					market_id,
					current_volume,
				)
				.or_else(|_| Err(Error::<T>::TradeBatchError546))?;

			// OrderSide::Maker is hardcoded, becuase all open positions of delisted market
			// are considered as Maker orders for closing
			let (fee_rate, _) = Self::get_fee_rate(
				account_id,
				&market_fees,
				Side::Sell,
				OrderSide::Maker,
				total_30day_volume,
			);

			let mut fee = fee_rate * leveraged_order_value;
			fee = fee.round_to_precision(collateral_token_decimal.into());

			let fee_share_amount = Self::update_and_get_fee_share(
				account_id,
				collateral_id,
				master_30day_volume,
				current_volume,
				fee,
				collateral_token_decimal,
			);

			// Deduct fee while closing a position
			if fee != FixedI128::zero() {
				T::TradingAccountPallet::transfer_from(
					account_id,
					collateral_id,
					market_id,
					fee,
					BalanceChangeReason::Fee,
				);

				T::TradingAccountPallet::handle_fee_split(
					account_id,
					collateral_id,
					market_id,
					fee,
					fee_share_amount,
				);
			}

			// Since new position size is 0, it means that the position in a certain
			// market and direction is completely closed
			MarketToAccountMap::<T>::remove((market_id, direction), account_id);

			let opposite_direction = Self::get_opposite_direction(direction);
			let opposite_position =
				PositionsMap::<T>::get(&account_id, (market_id, opposite_direction));
			if opposite_position.size == FixedI128::zero() {
				let mut markets = CollateralToMarketMap::<T>::get(&account_id, collateral_id);

				markets.retain(|&market| market != market_id);

				CollateralToMarketMap::<T>::insert(&account_id, collateral_id, &markets);

				let force_closure_flag = ForceClosureFlagMap::<T>::get(account_id, collateral_id);
				// If delisted market position was the only position that was left to be closed for
				// liquidation, then on closure of this position force closure flag should be reset
				if force_closure_flag.is_some() &&
					force_closure_flag.unwrap() == ForceClosureFlag::Liquidate &&
					markets.is_empty()
				{
					// Remove the liquidation flag and check for deferred deposits
					let validation_response =
						Self::reset_force_closure_flags(account_id, collateral_id);
					match validation_response {
						Ok(_) => (),
						Err(_) => return Err(Error::<T>::DeferredDepositError),
					}
				}
			}
			PositionsMap::<T>::remove(account_id, (market_id, direction));

			// Update locked margin for an account
			let current_margin_locked =
				T::TradingAccountPallet::get_locked_margin(account_id, collateral_id);
			let new_margin_lock_amount: FixedI128 =
				current_margin_locked - position_details.margin_amount;
			T::TradingAccountPallet::set_locked_margin(
				account_id,
				collateral_id,
				new_margin_lock_amount,
			);

			// Emit order executed event
			Self::deposit_event(Event::OrderExecuted {
				account_id,
				order_id: U256::zero(),
				market_id,
				size: order_size,
				direction: direction.into(),
				side: Side::Sell.into(),
				order_type: OrderType::ADS.into(),
				execution_price,
				pnl,
				fee,
				is_final: true,
				is_maker: true,
			});

			Ok(fee)
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
			amount_to_be_sold: FixedI128,
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
				DeleveragableMap::<T>::insert(account_id, collateral_id, amount_to_be_sold);
			}

			// Emit event
			Self::deposit_event(Event::ForceClosureFlagsChanged {
				account_id,
				collateral_id,
				force_closure_flag: force_closure_flag.into(),
			});
		}

		fn get_deleveragable_amount(account_id: U256, collateral_id: u128) -> FixedI128 {
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
					let (maintenance_requirement, mark_price) =
						Self::get_maintenance_requirement(element, &long_pos);
					let position_extended =
						PositionExtended::new(long_pos, maintenance_requirement, mark_price);
					pos_vec.push(position_extended);
				}
				if short_pos.size != FixedI128::zero() {
					let (maintenance_requirement, mark_price) =
						Self::get_maintenance_requirement(element, &short_pos);
					let position_extended =
						PositionExtended::new(short_pos, maintenance_requirement, mark_price);
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
				_,
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
			}
		}

		fn get_account_info(account_id: U256, collateral_id: u128) -> AccountInfo {
			let (_, total_margin, available_margin, _, _, _) =
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
					let (maintenance_requirement, mark_price) =
						Self::get_maintenance_requirement(element, &long_pos);
					let position_extended =
						PositionExtended::new(long_pos, maintenance_requirement, mark_price);
					positions.push(position_extended);
				}
				if short_pos.size != FixedI128::zero() {
					let (maintenance_requirement, mark_price) =
						Self::get_maintenance_requirement(element, &short_pos);
					let position_extended =
						PositionExtended::new(short_pos, maintenance_requirement, mark_price);
					positions.push(position_extended);
				}
			}

			let collateral_balance =
				T::TradingAccountPallet::get_balance(account_id, collateral_id);

			let force_closure_flag = ForceClosureFlagMap::<T>::get(account_id, collateral_id);
			let unused_balance =
				T::TradingAccountPallet::get_unused_balance(account_id, collateral_id);

			AccountInfo {
				positions,
				available_margin,
				total_margin,
				collateral_balance,
				force_closure_flag,
				unused_balance,
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

		fn get_all_fee_rates(
			account_id: U256,
			market_id: u128,
			collateral_id: u128,
			volume: FixedI128,
		) -> FeeRates {
			let fees_details = T::TradingFeesPallet::get_all_fees(market_id, collateral_id);

			FeeRates {
				maker_buy: Self::get_fee_rate(
					account_id,
					&fees_details,
					Side::Buy,
					OrderSide::Maker,
					volume,
				)
				.0,
				maker_sell: Self::get_fee_rate(
					account_id,
					&fees_details,
					Side::Sell,
					OrderSide::Maker,
					volume,
				)
				.0,
				taker_buy: Self::get_fee_rate(
					account_id,
					&fees_details,
					Side::Buy,
					OrderSide::Taker,
					volume,
				)
				.0,
				taker_sell: Self::get_fee_rate(
					account_id,
					&fees_details,
					Side::Sell,
					OrderSide::Taker,
					volume,
				)
				.0,
			}
		}

		fn get_fee_rate(
			account_id: U256,
			base_fees: &BaseFeeAggregate,
			side: Side,
			order_side: OrderSide,
			volume: FixedI128,
		) -> (FixedI128, u8) {
			// Get fee data for given side and orderside
			let fee_details = match (order_side, side) {
				(OrderSide::Maker, Side::Buy) => &base_fees.maker_buy,
				(OrderSide::Maker, Side::Sell) => &base_fees.maker_sell,
				(OrderSide::Taker, Side::Buy) => &base_fees.taker_buy,
				(OrderSide::Taker, Side::Sell) => &base_fees.taker_sell,
			};

			// If no fee_tiers are set, return 0 as fees and tier as 0
			if fee_details.is_empty() {
				return (FixedI128::zero(), 0);
			}

			// Get fee dicsount
			let fee_discount = T::TradingAccountPallet::get_fee_discount(account_id);

			// Get tier one fee details
			let tier_1_fee_details = &fee_details[0];
			let mut fee = tier_1_fee_details.fee;
			let mut fee_tier: u8 = 1_u8;

			// Find the appropriate fee tier for the user
			for (index, tier) in fee_details.iter().enumerate().rev() {
				if volume >= tier.volume {
					fee = tier.fee;
					fee_tier = (index + 1) as u8;
					break;
				}
			}

			let fee = fee * (FixedI128::one() - fee_discount);

			(fee, fee_tier)
		}

		fn get_fee(account_id: U256, market_id: u128) -> (FeeRates, u64) {
			let zero = FixedI128::zero();
			let is_registered = T::TradingAccountPallet::is_registered_user(account_id);
			if !is_registered {
				return (FeeRates::new(zero, zero, zero, zero), 0)
			}

			// Get monetary address from account_id
			// Unwrap won't fail here as user is valid
			let monetary_address =
				T::TradingAccountPallet::get_account(&account_id).unwrap().account_address;
			let last_30day_volume: FixedI128;
			match T::TradingAccountPallet::get_30day_volume(
				monetary_address,
				market_id,
				VolumeType::UserVolume,
			) {
				Ok(value) => last_30day_volume = value,
				Err(_) => return (FeeRates::new(zero, zero, zero, zero), 0),
			}

			let market = T::MarketPallet::get_market(market_id).unwrap();

			let fee_rates = Self::get_all_fee_rates(
				account_id,
				market_id,
				market.asset_collateral,
				last_30day_volume,
			);

			let expires_at: u64 = get_expiry_timestamp(T::TimeProvider::now().as_secs());
			(fee_rates, expires_at)
		}

		fn get_withdrawable_amount(account_id: U256, collateral_id: u128) -> FixedI128 {
			T::TradingAccountPallet::get_amount_to_withdraw(account_id, collateral_id)
		}

		fn get_remaining_trading_cleanup_calls() -> u64 {
			let start_timestamp = match StartTimestamp::<T>::get() {
				Some(timestamp) => timestamp,
				None => return 0_u64,
			};

			let current_timestamp: u64 = T::TimeProvider::now().as_secs();
			let timestamp_limit: u64 = current_timestamp - FOUR_WEEKS;
			let cleanup_count = CLEANUP_COUNT;

			if start_timestamp < timestamp_limit {
				let remaining_time = timestamp_limit - start_timestamp;
				let cleanup_calls = remaining_time / cleanup_count;
				return if remaining_time % cleanup_count != 0 {
					cleanup_calls + 1
				} else {
					cleanup_calls
				};
			}

			0_u64
		}

		fn get_no_of_delisted_market_positions(market_id: u128) -> u32 {
			let long_users_count =
				MarketToAccountMap::<T>::iter_prefix_values((market_id, Direction::Long)).count()
					as u32;
			let short_users_count =
				MarketToAccountMap::<T>::iter_prefix_values((market_id, Direction::Short)).count()
					as u32;
			return long_users_count + short_users_count;
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// Offchain worker entry point
		fn offchain_worker(block_number: BlockNumberFor<T>) {
			let signer = Signer::<T, T::AuthorityId>::all_accounts();
			if !signer.can_sign() {
				log::info!(
					"No local accounts available. Consider adding one via `author_insertKey` RPC."
				);
			}

			// Converts block number to u32 format
			let block_number = block_number.saturated_into::<u32>();
			// Calls perform_cleanup extrinsic after every block interval
			if block_number % BLOCK_INTERVAL == 0 {
				// Call perform trading clean up only when there are orders to clean up
				let cleanup_calls = Self::get_remaining_trading_cleanup_calls();
				if cleanup_calls != 0 {
					let results =
						signer.send_signed_transaction(|_account| Call::perform_cleanup {});
					for (acc, res) in &results {
						match res {
							Ok(()) => log::info!("[{:?}]: Submit transaction success.", acc.id),
							Err(e) => log::info!(
								"[{:?}]: Submit transaction failure. Reason: {:?}",
								acc.id,
								e
							),
						}
					}
				}
			}

			// Calls close_delisted_market_positions extrinsic after every
			// BLOCK_INTERVAL_FOR_DELISTING
			if block_number % BLOCK_INTERVAL_FOR_DELISTING == 0 {
				// Get all markets which are non tradable
				let markets = T::MarketPallet::get_all_markets_by_state(false, false);
				if markets.len() != 0 {
					for market_id in markets {
						// Get no.of open positions of a delisted market
						let delisted_market_positions_count =
							Self::get_no_of_delisted_market_positions(market_id);
						if delisted_market_positions_count != 0 {
							let results = signer.send_signed_transaction(|_account| {
								Call::close_delisted_market_positions { market_id }
							});
							for (acc, res) in &results {
								match res {
									Ok(()) =>
										log::info!("[{:?}]: Submit transaction success.", acc.id),
									Err(e) => log::info!(
										"[{:?}]: Submit transaction failure. Reason: {:?}",
										acc.id,
										e
									),
								}
							}
							// Breaking the loop, so that multiple markets if delisted won't be
							// called at the same time
							break;
						}
					}
				}
			}
		}
	}
}
