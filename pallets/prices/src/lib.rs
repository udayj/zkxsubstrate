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
		pallet_prelude::{DispatchResult, *},
		traits::UnixTime,
	};
	use frame_system::{ensure_signed, pallet_prelude::*};
	use pallet_support::{
		helpers::{fixed_pow, ln, max},
		traits::{
			AssetInterface, FixedI128Ext, MarketInterface, PricesInterface,
			TradingAccountInterface, TradingInterface,
		},
		types::{
			ABRDetails, ABRState, BalanceChangeReason, CurrentPrice, Direction, FundModifyType,
			HistoricalPrice, LastOraclePrice, MultiplePrices, PositionExtended,
		},
	};
	use primitive_types::U256;
	use sp_arithmetic::{fixed_point::FixedI128, traits::Zero, FixedPointNumber};

	// ////////////
	// Constants //
	// ////////////

	// To do checks for bollinger width
	const BOLLINGER_WIDTH_15: FixedI128 = FixedI128::from_inner(1500000000000000000); //1.5
	const BOLLINGER_WIDTH_20: FixedI128 = FixedI128::from_inner(2000000000000000000); //2.0
	const BOLLINGER_WIDTH_25: FixedI128 = FixedI128::from_inner(2500000000000000000); //2.5

	// To do checks for base_abr_rate
	const BASE_ABR_MIN: FixedI128 = FixedI128::from_inner(12500000000000); // 0.0000125
	const BASE_ABR_MAX: FixedI128 = FixedI128::from_inner(100000000000000); // 0.0001

	// Minimum ABR interval
	const ABR_INTERVAL_MIN: u64 = 3600;
	// Price interval with which historical prices should be stored for ABR
	const ABR_PRICE_INTERVAL: u64 = 60;
	// To convert milliseconds to seconds
	const MILLIS_PER_SECOND: u64 = 1000;
	// Duration for which price data is available
	static FOUR_WEEKS: u64 = 2419200;
	// Clear limit for the historical prices map
	static CLEAR_LIMIT: u32 = 1000;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type AssetPallet: AssetInterface;
		type MarketPallet: MarketInterface;
		type TradingAccountPallet: TradingAccountInterface;
		type TradingPallet: TradingInterface;
		type TimeProvider: UnixTime;
	}

	#[pallet::storage]
	#[pallet::getter(fn last_oracle_price)]
	// k1 - market_id, v - LastOraclePrice
	pub(super) type LastOraclePricesMap<T: Config> =
		StorageMap<_, Twox64Concat, u128, LastOraclePrice, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn current_price)]
	// k1 - market_id, v - CurrentPrice
	pub(super) type CurrentPricesMap<T: Config> =
		StorageMap<_, Twox64Concat, u128, CurrentPrice, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn historical_price)]
	// k1 - timestamp, k2 - market_id, v - HistoricalPrice
	pub(super) type HistoricalPricesMap<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		u64,
		Blake2_128Concat,
		u128,
		HistoricalPrice,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn prices_start_timestamp)]
	// The beginning timestamp for which prices data is stored
	pub(super) type PricesStartTimestamp<T: Config> = StorageValue<_, u64, OptionQuery>;

	/// Stores the timestamp at which substrate was initialised
	#[pallet::storage]
	#[pallet::getter(fn initialisation_timestamp)]
	pub(super) type InitialisationTimestamp<T: Config> = StorageValue<_, u64, ValueQuery>;

	/// Stores the state of ABR
	#[pallet::storage]
	#[pallet::getter(fn abr_state)]
	pub(super) type AbrState<T: Config> = StorageValue<_, ABRState, ValueQuery>;

	/// Stores the epoch value
	#[pallet::storage]
	#[pallet::getter(fn abr_epoch)]
	pub(super) type AbrEpoch<T: Config> = StorageValue<_, u64, ValueQuery>;

	/// Stores the ABR Interval
	#[pallet::storage]
	#[pallet::getter(fn abr_interval)]
	pub(super) type AbrInterval<T: Config> = StorageValue<_, u64, ValueQuery>;

	/// Stores the no of users per batch
	#[pallet::storage]
	#[pallet::getter(fn users_per_batch)]
	pub(super) type UsersPerBatch<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn epoch_to_timestamp)]
	/// key - Epoch, value - timestamp
	pub(super) type EpochToTimestampMap<T: Config> =
		StorageMap<_, Twox64Concat, u64, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn no_of_batches_for_epoch)]
	/// key - Epoch, value - No.of batches
	pub(super) type NoOfBatchesForEpochMap<T: Config> =
		StorageMap<_, Twox64Concat, u64, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn batches_fetched_for_epoch)]
	/// key - Epoch, value - No.of batches fetched
	pub(super) type BatchesFetchedForEpochMap<T: Config> =
		StorageMap<_, Twox64Concat, u64, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn epoch_market_to_abr_value)]
	/// key1 - Epoch, Key2 - Market_id, value - ABR value
	pub(super) type EpochMarketToAbrValueMap<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, u64, Blake2_128Concat, u128, FixedI128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn epoch_market_to_last_price)]
	/// key1 - Epoch, Key2 - Market_id, value - Last market price
	pub(super) type EpochMarketToLastPriceMap<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, u64, Blake2_128Concat, u128, FixedI128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn abr_market_status)]
	/// key1 - Epoch, Key2 - Market_id, value - Status of the market
	pub(super) type AbrMarketStatusMap<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, u64, Blake2_128Concat, u128, bool, ValueQuery>;

	/// Stores the base ABR
	#[pallet::storage]
	#[pallet::getter(fn base_abr)]
	pub(super) type BaseAbr<T: Config> = StorageValue<_, FixedI128, ValueQuery>;

	/// Stores the bollinger width
	#[pallet::storage]
	#[pallet::getter(fn bollinger_width)]
	pub(super) type BollingerWidth<T: Config> = StorageValue<_, FixedI128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn max_abr)]
	// k1 - market_id, v - Maximum ABR allowed
	pub(super) type MaxABRPerMarket<T: Config> =
		StorageMap<_, Twox64Concat, u128, FixedI128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn default_max)]
	// v - Default maximum ABR allowed
	pub(super) type MaxABRDefault<T: Config> = StorageValue<_, FixedI128, ValueQuery>;

	#[pallet::genesis_config]
	#[derive(frame_support::DefaultNoBound)]
	pub struct GenesisConfig<T: Config> {
		pub abr_interval: u64,
		pub base_abr: FixedI128,
		pub bollinger_width: FixedI128,
		pub users_per_batch: u64,
		pub max_abr_default: FixedI128,
		#[serde(skip)]
		pub _config: sp_std::marker::PhantomData<T>,
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			AbrInterval::<T>::put(&self.abr_interval);
			BaseAbr::<T>::put(&self.base_abr);
			BollingerWidth::<T>::put(&self.bollinger_width);
			UsersPerBatch::<T>::put(&self.users_per_batch);
			MaxABRDefault::<T>::put(&self.max_abr_default);
		}
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Invalid value for price
		InvalidPrice,
		/// Invalid value for Market Id
		MarketNotFound,
		/// Price interval should be >= 1 second
		InvalidPriceInterval,
		/// When no.of users per batch provided is invalid
		InvalidUsersPerBatch,
		/// When ABR interval provided is invalid
		InvalidAbrInterval,
		/// When timestamp provided is invalid
		InvalidTimestamp,
		/// When ABR state is invalid
		InvalidState,
		/// When ABR value is already set for the market
		AbrValueAlreadySet,
		/// When market provided is not tradable
		MarketNotTradable,
		/// When interval provided for setting ABR price is invalid
		InvalidAbrPriceInterval,
		/// When Base ABR provided is not within the range
		InvalidBaseAbr,
		/// When bollinger width provided is invalid
		InvalidBollingerWidth,
		/// Invalid value for initialisation timestamp
		InvalidInitialisationTimestamp,
		/// Set ABR value called before the abr interval is met
		EarlyAbrCall,
		/// When initialisation timestamp is already set
		InitialisationTimestampAlreadySet,
		/// When negative max value is passed to set_max_abr
		NegativeMaxValue,
		/// When timestamp provided is not yet met
		FutureTimestampPriceUpdate,
		/// Prices Start timestamp is not set
		PricesStartTimestampEmpty,
		/// When Price availability duration provided is invalid
		InvalidPriceAvailabilityDuration,
		/// When cleanup count per batch provided is invalid
		InvalidCountPerBatch,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Last traded price was successfully updated
		LastOraclePriceUpdated { market_id: u128, price: LastOraclePrice },
		/// ABR timestamp set successfully
		AbrTimestampSet { epoch: u64, timestamp: u64 },
		/// ABR state changed successfully
		AbrStateChanged { epoch: u64, state: ABRState },
		/// ABR value set successfully
		AbrValueSet { epoch: u64, market_id: u128, abr_value: FixedI128, abr_last_price: FixedI128 },
		/// ABR payment made successfully
		AbrPaymentMade { epoch: u64, batch_id: u64 },
		/// ABR payment for a user made successfully
		UserAbrPayment {
			account_id: U256,
			market_id: u128,
			collateral_id: u128,
			abr_value: FixedI128,
			abr_timestamp: u64,
			amount: FixedI128,
			modify_type: FundModifyType,
			position_size: FixedI128,
		},
		/// ABR interval updated successfully
		AbrIntervalUpdated { abr_interval: u64 },
		/// Default Max ABR value updated successfully
		DefaultMaxAbrUpdated { max_abr_value: FixedI128 },
		/// Max ABR value of a market updated successfully
		MaxAbrForMarketUpdated { market_id: u128, max_abr_value: FixedI128 },
		/// Initialisation timestamp updated successfully
		InitialisationTimestampUpdated { timestamp: u64 },
		/// No of users per batch updated successfully
		NoOfUsersPerBatchUpdated { no_of_users_per_batch: u64 },
		/// Base ABR updated successfully
		BaseAbrUpdated { base_abr: FixedI128 },
		/// Bollinger width updated successfully
		BollingerWidthUpdated { bollinger_width: FixedI128 },
		/// Index/mark prices updated successfully
		PricesUpdated { timestamp: u64, prices: Vec<MultiplePrices> },
		/// Price availability duration updated successfully
		PriceAvailabilityDurationUpdated { price_availability_duration: u64 },
	}

	// Pallet callable functions
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// External function to be called for setting Initialisation timestamp
		#[pallet::weight(0)]
		pub fn set_initialisation_timestamp(
			origin: OriginFor<T>,
			timestamp: u64,
		) -> DispatchResult {
			// Make sure the caller is from a signed origin
			ensure_root(origin)?;

			let timestamp = Self::convert_to_seconds(timestamp);

			ensure!(timestamp > 0, Error::<T>::InvalidInitialisationTimestamp);
			ensure!(
				InitialisationTimestamp::<T>::get() == 0,
				Error::<T>::InitialisationTimestampAlreadySet
			);

			InitialisationTimestamp::<T>::put(timestamp);

			// Emit Initialisation Timestamp updated event
			Self::deposit_event(Event::InitialisationTimestampUpdated { timestamp });
			Ok(())
		}

		/// External function to be called for setting the default max abr
		#[pallet::weight(0)]
		pub fn set_default_max_abr(
			origin: OriginFor<T>,
			max_abr_value: FixedI128,
		) -> DispatchResult {
			// Make sure the caller is from a signed origin
			ensure_root(origin)?;

			// Validate the value
			ensure!(!max_abr_value.is_negative(), Error::<T>::NegativeMaxValue);

			// Set the given abr value
			MaxABRDefault::<T>::set(max_abr_value);

			// Emit Default Max ABR value updated event
			Self::deposit_event(Event::DefaultMaxAbrUpdated { max_abr_value });
			Ok(())
		}

		/// External function to be called for setting max abr per market
		#[pallet::weight(0)]
		pub fn set_max_abr(
			origin: OriginFor<T>,
			market_id: u128,
			max_abr_value: FixedI128,
		) -> DispatchResult {
			// Make sure the caller is from a signed origin
			ensure_root(origin)?;

			// Check if the market exists and is tradable
			let market = T::MarketPallet::get_market(market_id);
			ensure!(market.is_some(), Error::<T>::MarketNotFound);
			let market = market.unwrap();

			ensure!(market.is_tradable == true, Error::<T>::MarketNotTradable);

			// Validate the value
			ensure!(!max_abr_value.is_negative(), Error::<T>::NegativeMaxValue);

			// Set the given abr value
			MaxABRPerMarket::<T>::insert(market_id, max_abr_value);

			// Emit Max ABR updated for a market event
			Self::deposit_event(Event::MaxAbrForMarketUpdated { market_id, max_abr_value });
			Ok(())
		}

		/// External function to be called for setting ABR interval
		#[pallet::weight(0)]
		pub fn set_abr_interval(origin: OriginFor<T>, new_abr_interval: u64) -> DispatchResult {
			// Make sure the caller is from a signed origin
			ensure_root(origin)?;

			//  ABR interval must be >= one hour
			ensure!(new_abr_interval >= ABR_INTERVAL_MIN, Error::<T>::InvalidAbrInterval);

			AbrInterval::<T>::put(new_abr_interval);

			// Emit ABR interval updated event
			Self::deposit_event(Event::AbrIntervalUpdated { abr_interval: new_abr_interval });
			Ok(())
		}

		/// External function to be called for setting base ABR
		#[pallet::weight(0)]
		pub fn set_base_abr(origin: OriginFor<T>, new_base_abr: FixedI128) -> DispatchResult {
			// Make sure the caller is from a signed origin
			ensure_root(origin)?;

			//  Base ABR must be >= BASE_ABR_MIN and <= BASE_ABR_MAX
			ensure!(
				(new_base_abr <= BASE_ABR_MAX) && (new_base_abr >= BASE_ABR_MIN),
				Error::<T>::InvalidBaseAbr
			);

			BaseAbr::<T>::put(new_base_abr);

			// Emit Base ABR updated event
			Self::deposit_event(Event::BaseAbrUpdated { base_abr: new_base_abr });
			Ok(())
		}

		/// External function to be called for setting bollinger width
		#[pallet::weight(0)]
		pub fn set_bollinger_width(
			origin: OriginFor<T>,
			new_bollinger_width: FixedI128,
		) -> DispatchResult {
			// Make sure the caller is from a signed origin
			ensure_root(origin)?;

			let is_valid: bool;
			if (new_bollinger_width == BOLLINGER_WIDTH_15) ||
				(new_bollinger_width == BOLLINGER_WIDTH_20) ||
				(new_bollinger_width == BOLLINGER_WIDTH_25)
			{
				is_valid = true;
			} else {
				is_valid = false;
			}
			ensure!(is_valid, Error::<T>::InvalidBollingerWidth);

			BollingerWidth::<T>::put(new_bollinger_width);

			// Emit Bollinger width updated event
			Self::deposit_event(Event::BollingerWidthUpdated {
				bollinger_width: new_bollinger_width,
			});
			Ok(())
		}

		/// External function to be called for setting no.of users per batch
		#[pallet::weight(0)]
		pub fn set_no_of_users_per_batch(
			origin: OriginFor<T>,
			new_no_of_users_per_batch: u64,
		) -> DispatchResult {
			// Make sure the caller is from a signed origin
			ensure_root(origin)?;

			// No of users in a batch must be > 0
			ensure!(new_no_of_users_per_batch > 0, Error::<T>::InvalidUsersPerBatch);

			UsersPerBatch::<T>::put(new_no_of_users_per_batch);

			// Emit No.of users per batch updated event
			Self::deposit_event(Event::NoOfUsersPerBatchUpdated {
				no_of_users_per_batch: new_no_of_users_per_batch,
			});
			Ok(())
		}

		/// External function to be called for setting ABR value
		#[pallet::weight(0)]
		pub fn set_abr_value(origin: OriginFor<T>, market_id: u128) -> DispatchResult {
			// Make sure the caller is from a signed origin
			ensure_signed(origin)?;

			// Get current state and epoch
			let mut current_epoch = AbrEpoch::<T>::get();

			if AbrState::<T>::get() == ABRState::State0 {
				// This call transitions the state to State::1
				current_epoch = Self::set_abr_timestamp(current_epoch)?;
			}

			let current_state = AbrState::<T>::get();

			// ABR must be in state 1
			ensure!(current_state == ABRState::State1, Error::<T>::InvalidState);

			// Validate market
			let market = T::MarketPallet::get_market(market_id);
			ensure!(market.is_some(), Error::<T>::MarketNotFound);
			let market = market.unwrap();

			ensure!(market.is_tradable == true, Error::<T>::MarketNotTradable);

			// Check if the market's abr is already set
			let market_status = AbrMarketStatusMap::<T>::get(current_epoch, market_id);
			ensure!(market_status == false, Error::<T>::AbrValueAlreadySet);

			// Compute epoch start and end timestamps
			let epoch_end_timestamp = EpochToTimestampMap::<T>::get(current_epoch);
			let abr_interval = AbrInterval::<T>::get();
			let epoch_start_timestamp = epoch_end_timestamp - abr_interval;

			// Fetch index and mark prices
			let (index_prices, mark_prices) =
				Self::get_prices_for_abr(market_id, epoch_start_timestamp, epoch_end_timestamp);

			// Fetch base ABR and bollinger width
			let base_abr = BaseAbr::<T>::get();
			let bollinger_width = BollingerWidth::<T>::get();

			let mut abr_value = FixedI128::zero();
			let mut abr_last_price = FixedI128::zero();

			if index_prices.len() != 0 && mark_prices.len() != 0 {
				// Calculate ABR
				(abr_value, abr_last_price) =
					Self::calculate_abr(mark_prices, index_prices, base_abr, bollinger_width, 8);
			}

			// If it's larger than max, use max
			let abr_value = Self::get_adjusted_abr_value(market_id, abr_value);

			// Set the market's ABR status as true
			AbrMarketStatusMap::<T>::insert(current_epoch, market_id, true);

			// Update ABR value for the market
			EpochMarketToAbrValueMap::<T>::insert(current_epoch, market_id, abr_value);

			// Update Last price used while computing ABR
			EpochMarketToLastPriceMap::<T>::insert(current_epoch, market_id, abr_last_price);

			// Emit ABR Value set event
			Self::deposit_event(Event::AbrValueSet {
				epoch: current_epoch,
				market_id,
				abr_value,
				abr_last_price,
			});

			// Check if all markets are set, if yes change the state
			Self::check_abr_markets_status(current_epoch);

			Ok(())
		}

		/// External function to be called for making ABR payments
		#[pallet::weight(0)]
		pub fn make_abr_payments(origin: OriginFor<T>) -> DispatchResult {
			// Make sure the caller is from a signed origin
			ensure_signed(origin)?;

			// Get current state, epoch and timestamp
			let current_state = AbrState::<T>::get();
			let current_epoch = AbrEpoch::<T>::get();
			let current_timestamp = EpochToTimestampMap::<T>::get(current_epoch);

			// ABR must be in state 2
			ensure!(current_state == ABRState::State2, Error::<T>::InvalidState);

			let users_list = Self::get_current_batch(current_epoch);

			Self::pay_abr(current_epoch, users_list, current_timestamp);

			Ok(())
		}

		/// update index and mark prices for several markets
		#[pallet::weight(0)]
		pub fn update_prices(
			origin: OriginFor<T>,
			prices: Vec<MultiplePrices>,
			timestamp: u64,
		) -> DispatchResult {
			// Make sure the caller is from a signed origin
			ensure_signed(origin)?;

			// Get the current timestamp
			let current_timestamp: u64 = T::TimeProvider::now().as_secs();

			// Get the current timestamp and last timestamp for which prices were updated
			let timestamp = Self::convert_to_seconds(timestamp);

			ensure!(timestamp <= current_timestamp + 10, Error::<T>::FutureTimestampPriceUpdate);
			// Modify start timestamp
			let start_timestamp = PricesStartTimestamp::<T>::get();
			if (start_timestamp.is_some() && timestamp < start_timestamp.unwrap()) ||
				start_timestamp.is_none()
			{
				PricesStartTimestamp::<T>::put(timestamp);
			}

			// Iterate through the vector of markets and add to prices map
			for curr_market in &prices {
				ensure!(curr_market.index_price >= FixedI128::zero(), Error::<T>::InvalidPrice);
				ensure!(curr_market.mark_price >= FixedI128::zero(), Error::<T>::InvalidPrice);

				// Get Market from the corresponding Id
				match T::MarketPallet::get_market(curr_market.market_id) {
					Some(m) => m,
					None => return Err(Error::<T>::MarketNotFound.into()),
				};

				let current_price = CurrentPricesMap::<T>::get(curr_market.market_id);
				if timestamp > current_price.timestamp {
					// Create a struct object for the current price
					let new_price: CurrentPrice = CurrentPrice {
						timestamp,
						index_price: curr_market.index_price,
						mark_price: curr_market.mark_price,
					};

					CurrentPricesMap::<T>::insert(curr_market.market_id, new_price);
				}

				// Update historical price
				let historical_price = HistoricalPrice {
					index_price: curr_market.index_price,
					mark_price: curr_market.mark_price,
				};
				HistoricalPricesMap::<T>::insert(
					timestamp,
					curr_market.market_id,
					historical_price,
				);
			}

			// Emit index/mark prices updated event
			Self::deposit_event(Event::PricesUpdated { timestamp, prices });

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn perform_prices_cleanup(origin: OriginFor<T>) -> DispatchResult {
			// Make sure the caller is from a signed origin
			ensure_signed(origin)?;

			let start_timestamp =
				PricesStartTimestamp::<T>::get().ok_or(Error::<T>::PricesStartTimestampEmpty)?;
			let current_timestamp: u64 = T::TimeProvider::now().as_secs();
			let timestamp_limit = current_timestamp - FOUR_WEEKS;
			let mut cleanup_count = 3600;

			for timestamp in start_timestamp..timestamp_limit {
				if cleanup_count == 0 {
					PricesStartTimestamp::<T>::put(timestamp);
					break;
				}
				// we are passing None as 3rd argument as no.of prices stored for a particular
				// timestamp is limited and it doesn't need another call to remove
				let _ = HistoricalPricesMap::<T>::clear_prefix(timestamp, CLEAR_LIMIT, None);
				cleanup_count -= 1;
			}
			if cleanup_count != 0 && start_timestamp < timestamp_limit {
				PricesStartTimestamp::<T>::put(timestamp_limit);
			}

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn get_adjusted_abr_value(market_id: u128, value: FixedI128) -> FixedI128 {
			let max_abr_value = MaxABRPerMarket::<T>::get(market_id);
			let max = if max_abr_value == FixedI128::zero() {
				MaxABRDefault::<T>::get()
			} else {
				max_abr_value
			};

			if Self::get_absolute_value(value) > max {
				if value.is_negative() {
					-max
				} else {
					max
				}
			} else {
				value
			}
		}

		fn get_absolute_value(value: FixedI128) -> FixedI128 {
			if value.is_negative() {
				// If the value is negative, multiply by -1
				-value
			} else {
				// If the value is positive, return it as is
				value
			}
		}

		pub fn get_price(market_id: u128, timestamp: u64, price: FixedI128) -> FixedI128 {
			let market = T::MarketPallet::get_market(market_id);
			match market {
				Some(market) => {
					// Get the current timestamp
					let current_timestamp: u64 = T::TimeProvider::now().as_secs();

					let time_difference = current_timestamp - timestamp;
					if time_difference > market.ttl.into() {
						FixedI128::zero()
					} else {
						price
					}
				},
				None => FixedI128::zero(),
			}
		}

		fn get_last_abr_timestamp() -> u64 {
			// Get current state and epoch
			let current_state = AbrState::<T>::get();
			let current_epoch = AbrEpoch::<T>::get();

			if current_state == ABRState::State0 {
				if current_epoch == 0 || current_epoch == 1 {
					return InitialisationTimestamp::<T>::get()
				} else {
					return EpochToTimestampMap::<T>::get(current_epoch - 1)
				}
			} else {
				return EpochToTimestampMap::<T>::get(current_epoch)
			}
		}

		fn calculate_no_of_batches(users_per_batch: u128) -> u128 {
			// Get the count of accounts
			let accounts_count = T::TradingAccountPallet::get_accounts_count();

			let q = accounts_count / users_per_batch;
			let r = accounts_count % users_per_batch;

			if r == 0 {
				return q
			} else {
				return q + 1
			}
		}

		fn check_abr_markets_status(epoch: u64) {
			// get all the markets available in the system
			let markets = T::MarketPallet::get_all_markets_by_state(true, false);

			// Check the state of each market
			for market_id in markets {
				let market_status = AbrMarketStatusMap::<T>::get(epoch, market_id);
				if market_status == false {
					return
				}
			}

			// Emit ABR state changed event
			Self::deposit_event(Event::AbrStateChanged { epoch, state: ABRState::State2 });

			// Change the state if all the market's ABR value is set
			AbrState::<T>::put(ABRState::State2);
		}

		fn calculate_effective_abr(premiums: &[FixedI128]) -> FixedI128 {
			// Find the sum of the premium vec
			let premium_sum =
				premiums.iter().fold(FixedI128::zero(), |acc, &premium| acc + premium);

			// Return the calculated ABR without the base
			// py: sum(premiums)/len(premiums)*8
			premium_sum / (FixedI128::from((premiums.len() * 8) as i128))
		}

		fn calculate_price_diff_ratios(
			mark_prices: &[FixedI128],
			index_prices: &[FixedI128],
		) -> Vec<FixedI128> {
			// Initialize the result vector with the size of prices vector
			let total_len = mark_prices.len();
			let mut premiums = Vec::<FixedI128>::with_capacity(total_len);

			for iterator in 0..total_len {
				// TODO(merkle-groot): possibly check for division by zero error here
				// py: diff.append(mark[i] - index[i]/mark[i])
				premiums
					.push((mark_prices[iterator] - index_prices[iterator]) / mark_prices[iterator]);
			}

			premiums
		}

		fn calculate_jump(
			premiums: &mut [FixedI128],
			upper_band: &[FixedI128],
			lower_band: &[FixedI128],
			mark_prices: &[FixedI128],
			index_prices: &[FixedI128],
		) -> Vec<FixedI128> {
			// Use the total_len as the upper bound of iteration
			let total_len = mark_prices.len();

			for iterator in 0..total_len {
				// Calculate the jump from upper and lower halves
				//  py: upper_diff = max(mark[i] - upper[i], 0)
				//  py: lower_diff = max(lower[i] - mark[i], 0)
				let upper_diff =
					max(FixedI128::zero(), mark_prices[iterator] - upper_band[iterator]);
				let lower_diff =
					max(FixedI128::zero(), lower_band[iterator] - mark_prices[iterator]);

				// If there's a jump from the upper band
				// Add the jump to premium
				if upper_diff > FixedI128::zero() {
					// py: jump = max(log(upper_diff)/mark[i], 0)
					premiums[iterator] = premiums[iterator] +
						max(ln(upper_diff) / index_prices[iterator], FixedI128::zero());
				} else if lower_diff > FixedI128::zero() {
					// py: jump = max(log(lower_diff)/spot[i], 0)
					premiums[iterator] = premiums[iterator] -
						max(ln(lower_diff) / index_prices[iterator], FixedI128::zero());
				}
			}

			premiums.to_vec()
		}

		fn calculate_std(
			prices: &[FixedI128],
			mean: FixedI128,
			boll_width: FixedI128,
		) -> FixedI128 {
			// find the length of the prices vec
			let total_len = prices.len();

			// Handle Edge case
			if total_len == 0 {
				return FixedI128::zero()
			}

			// Find the sum of square of differences from mean
			let diff_sum = prices
				.iter()
				.fold(FixedI128::zero(), |acc, &price| acc + fixed_pow(price - mean, 2_u64));

			// We divide by n-1 to find the std, since it's a sample
			// If it's 1, we don't subtract
			let adjusted_total_len = if total_len > 1 { total_len - 1 } else { total_len };
			let std_dev = (diff_sum / FixedI128::from(adjusted_total_len as i128)).sqrt();

			boll_width * std_dev
		}

		fn calculate_bollinger_bands(
			prices: &[FixedI128],
			mean_prices: &[FixedI128],
			window: usize,
			boll_width: FixedI128,
		) -> (Vec<FixedI128>, Vec<FixedI128>) {
			// Initialize the upper and lower vectors with the size of prices vector
			let total_len = prices.len();
			let mut upper_band = Vec::<FixedI128>::with_capacity(total_len);
			let mut lower_band = Vec::<FixedI128>::with_capacity(total_len);

			// Handle Edge case
			if total_len == 0 || window == 0 {
				return (upper_band, lower_band)
			}

			for iterator in 0..total_len {
				// calculate the standarad deviation for each window
				let std = if iterator < window {
					Self::calculate_std(&prices[0..iterator + 1], mean_prices[iterator], boll_width)
				} else {
					Self::calculate_std(
						&prices[iterator - window + 1..iterator + 1],
						mean_prices[iterator],
						boll_width,
					)
				};

				// Add to lower and upper band vectors
				lower_band.push(mean_prices[iterator] - std);
				upper_band.push(mean_prices[iterator] + std);
			}

			(lower_band, upper_band)
		}

		pub fn calculate_sliding_mean(prices: &[FixedI128], window: usize) -> Vec<FixedI128> {
			// Initialize the result vector with the size of prices vector
			let total_len = prices.len();
			let mut result = Vec::<FixedI128>::with_capacity(total_len);

			// Handle Edge case
			if total_len == 0 || window == 0 {
				return result
			}

			// Initialize window_sum and convert window size to FixedI128
			let mut window_sum = FixedI128::zero();
			let window_fixed = FixedI128::from(window as i128);

			for iterator in 0..total_len {
				// Calculate the sliding mean till the iterator
				if iterator < window {
					// Add to window_sum;
					// since it's below window_size we don't need to remove the first price in the
					// sum
					window_sum = window_sum + prices[iterator];

					// Add to result array
					result.push(window_sum / FixedI128::from((iterator + 1) as i128));
				} else {
					// Add the current price and remove the first price in the sum
					window_sum = window_sum - prices[iterator - window] + prices[iterator];

					// Add to result array
					result.push(window_sum / window_fixed);
				}
			}

			result
		}

		pub fn set_abr_timestamp(current_epoch: u64) -> Result<u64, Error<T>> {
			let abr_interval = AbrInterval::<T>::get();
			let current_timestamp: u64 = T::TimeProvider::now().as_secs();

			let new_epoch: u64;
			let last_abr_timestamp: u64;

			if current_epoch == 0 {
				new_epoch = 1;
				AbrEpoch::<T>::put(new_epoch);
				last_abr_timestamp = InitialisationTimestamp::<T>::get();
			} else {
				new_epoch = current_epoch;
				last_abr_timestamp = EpochToTimestampMap::<T>::get(current_epoch - 1);
			}

			let next_abr_timestamp = last_abr_timestamp + abr_interval;

			ensure!(current_timestamp >= next_abr_timestamp, Error::<T>::EarlyAbrCall);

			AbrState::<T>::put(ABRState::State1);
			EpochToTimestampMap::<T>::insert(new_epoch, next_abr_timestamp);

			// Get no of users in a batch
			let users_per_batch = UsersPerBatch::<T>::get();
			ensure!(users_per_batch != 0, Error::<T>::InvalidUsersPerBatch);

			// Get the no of batches
			let no_of_batches: u64 =
				Self::calculate_no_of_batches(users_per_batch as u128).try_into().unwrap();

			// Write the no of batches for this epoch
			NoOfBatchesForEpochMap::<T>::insert(new_epoch, no_of_batches);

			// Emit ABR timestamp set event
			Self::deposit_event(Event::AbrTimestampSet {
				epoch: new_epoch,
				timestamp: next_abr_timestamp,
			});

			// Emit ABR state changed event
			Self::deposit_event(Event::AbrStateChanged {
				epoch: new_epoch,
				state: ABRState::State1,
			});

			Ok(new_epoch)
		}

		pub fn get_current_batch(current_epoch: u64) -> Vec<U256> {
			// Get the current batch details
			let no_of_users_per_batch = UsersPerBatch::<T>::get();
			let batches_fetched = BatchesFetchedForEpochMap::<T>::get(current_epoch);
			let no_of_batches = NoOfBatchesForEpochMap::<T>::get(current_epoch);

			// Get the lower index of the batch
			let lower_limit = batches_fetched * no_of_users_per_batch;
			// Get the upper index of the batch
			let upper_limit = lower_limit + no_of_users_per_batch;

			// Fetch the required batch from Trading account pallet
			let account_list =
				T::TradingAccountPallet::get_account_list(lower_limit as u128, upper_limit as u128);

			// Increment batches_fetched
			let new_batches_fetched = batches_fetched + 1;
			if no_of_batches != 0 {
				BatchesFetchedForEpochMap::<T>::insert(current_epoch, new_batches_fetched);
			}

			// Emit ABR Payment made event
			Self::deposit_event(Event::AbrPaymentMade {
				epoch: current_epoch,
				batch_id: batches_fetched,
			});

			// If all batches are fetched, increment state and epoch
			if no_of_batches == 0 || no_of_batches == new_batches_fetched {
				AbrState::<T>::put(ABRState::State0);
				AbrEpoch::<T>::put(current_epoch + 1);
				// Emit ABR state changed event
				Self::deposit_event(Event::AbrStateChanged {
					epoch: current_epoch + 1,
					state: ABRState::State0,
				});
			}
			account_list
		}

		pub fn pay_abr(epoch: u64, users_list: Vec<U256>, timestamp: u64) {
			for user in users_list {
				// Get all collaterals in the system
				let collaterals = T::TradingAccountPallet::get_collaterals_of_user(user);
				// Iterate through all collaterals
				for collateral in collaterals {
					// Get all the open positions of the user
					let mut positions: Vec<PositionExtended> =
						T::TradingPallet::get_positions(user, collateral);
					// Sort the positions which are within the timestamp
					positions.retain(|position: &PositionExtended| {
						position.created_timestamp <= timestamp
					});

					// Iterate through all open positions
					for position in positions {
						// This will always fit in u128
						let market_id: u128 = position.market_id.try_into().unwrap();
						let collateral_asset = T::AssetPallet::get_asset(collateral).unwrap();
						let collateral_token_decimal = collateral_asset.decimals;

						// Get the abr value
						let abr_value = EpochMarketToAbrValueMap::<T>::get(epoch, market_id);

						// Get the abr last price
						let abr_last_price = EpochMarketToLastPriceMap::<T>::get(epoch, market_id);

						// Get the abr timestamp
						let abr_timestamp = EpochToTimestampMap::<T>::get(epoch);

						// Find if the abr_rate is +ve or -ve
						let mut payment_amount = abr_value * abr_last_price * position.size;
						payment_amount = payment_amount.saturating_abs();

						payment_amount =
							payment_amount.round_to_precision(collateral_token_decimal.into());
						// If the abr is negative
						if abr_value <= FixedI128::zero() {
							if position.direction == Direction::Short {
								Self::user_pays(
									user,
									collateral,
									payment_amount,
									abr_value,
									abr_timestamp,
									market_id,
									position.size,
								);
							} else {
								Self::user_receives(
									user,
									collateral,
									payment_amount,
									abr_value,
									abr_timestamp,
									market_id,
									position.size,
								);
							}
						} else {
							if position.direction == Direction::Short {
								Self::user_receives(
									user,
									collateral,
									payment_amount,
									abr_value,
									abr_timestamp,
									market_id,
									position.size,
								);
							} else {
								Self::user_pays(
									user,
									collateral,
									payment_amount,
									abr_value,
									abr_timestamp,
									market_id,
									position.size,
								);
							}
						}
					}
				}
			}
		}

		pub fn get_prices_for_abr(
			market_id: u128,
			epoch_start_timestamp: u64,
			epoch_end_timestamp: u64,
		) -> (Vec<FixedI128>, Vec<FixedI128>) {
			let mut index_prices = Vec::<FixedI128>::new();
			let mut mark_prices = Vec::<FixedI128>::new();

			let mut timestamp = epoch_start_timestamp;
			while timestamp <= epoch_end_timestamp {
				let price = HistoricalPricesMap::<T>::get(timestamp, market_id);
				if price.index_price != FixedI128::zero() && price.mark_price != FixedI128::zero() {
					index_prices.push(price.index_price);
					mark_prices.push(price.mark_price);
					timestamp += ABR_PRICE_INTERVAL;
				} else {
					timestamp += 1;
				}
			}
			(index_prices, mark_prices)
		}

		pub fn calculate_abr(
			mark_prices: Vec<FixedI128>,
			index_prices: Vec<FixedI128>,
			base_abr_rate: FixedI128,
			boll_width: FixedI128,
			window: usize,
		) -> (FixedI128, FixedI128) {
			// Calculate the sliding mean of mark_prices
			let mean_prices = Self::calculate_sliding_mean(&mark_prices, window);

			// Find the lower band and upper band of the Bollinger bands
			let (lower_band, upper_band) =
				Self::calculate_bollinger_bands(&mark_prices, &mean_prices, window, boll_width);

			// Calculate the price diff ratio between mark_prices and index_prices
			let price_diff_ratios = Self::calculate_price_diff_ratios(&mark_prices, &index_prices);

			// Calculate the sliding mean of the price_diff_ratio vec
			let mut price_diff_ratio_mean =
				Self::calculate_sliding_mean(&price_diff_ratios, window);

			// Add the jumps to premiums
			let jumps_array = Self::calculate_jump(
				&mut price_diff_ratio_mean,
				&upper_band,
				&lower_band,
				&mark_prices,
				&index_prices,
			);

			// Find the effective ABR
			let abr_value = Self::calculate_effective_abr(&jumps_array) + base_abr_rate;
			let abr_last_price =
				mark_prices.last().map_or_else(|| FixedI128::zero(), |&value| value);
			return (abr_value, abr_last_price)
		}

		pub fn user_pays(
			user: U256,
			collateral: u128,
			payment_amount: FixedI128,
			abr_value: FixedI128,
			abr_timestamp: u64,
			market_id: u128,
			position_size: FixedI128,
		) {
			T::TradingAccountPallet::transfer_from(
				user,
				collateral,
				payment_amount,
				BalanceChangeReason::ABR,
			);

			Self::deposit_event(Event::UserAbrPayment {
				account_id: user,
				market_id,
				collateral_id: collateral,
				abr_value,
				abr_timestamp,
				amount: payment_amount,
				modify_type: FundModifyType::Decrease,
				position_size,
			});
		}

		pub fn user_receives(
			user: U256,
			collateral: u128,
			payment_amount: FixedI128,
			abr_value: FixedI128,
			abr_timestamp: u64,
			market_id: u128,
			position_size: FixedI128,
		) {
			T::TradingAccountPallet::transfer(
				user,
				collateral,
				payment_amount,
				BalanceChangeReason::ABR,
			);

			Self::deposit_event(Event::UserAbrPayment {
				account_id: user,
				market_id,
				collateral_id: collateral,
				abr_value,
				abr_timestamp,
				amount: payment_amount,
				modify_type: FundModifyType::Increase,
				position_size,
			});
		}

		pub fn get_epoch_of_timestamp(start_timestamp: u64) -> u64 {
			let mut high_epoch = AbrEpoch::<T>::get();
			let mut low_epoch = 1;
			let mut mid_epoch;
			let abr_interval = AbrInterval::<T>::get();
			while low_epoch <= high_epoch {
				mid_epoch = ((high_epoch - low_epoch) / 2) + low_epoch;
				let epoch_timestamp = EpochToTimestampMap::<T>::get(mid_epoch);
				let difference: i128 = epoch_timestamp as i128 - start_timestamp as i128;
				if difference <= abr_interval as i128 && difference >= 0 {
					return mid_epoch
				}
				if start_timestamp > epoch_timestamp {
					low_epoch = mid_epoch + 1;
					continue
				}
				if start_timestamp < epoch_timestamp {
					high_epoch = mid_epoch - 1;
					continue
				}
			}
			return 0
		}
	}

	impl<T: Config> PricesInterface for Pallet<T> {
		fn get_last_oracle_price(market_id: u128) -> FixedI128 {
			let last_oracle_price = LastOraclePricesMap::<T>::get(market_id);

			Self::get_price(market_id, last_oracle_price.timestamp, last_oracle_price.price)
		}

		fn get_mark_price(market_id: u128) -> FixedI128 {
			let price = CurrentPricesMap::<T>::get(market_id);

			Self::get_price(market_id, price.timestamp, price.mark_price)
		}

		fn get_index_price(market_id: u128) -> FixedI128 {
			let price = CurrentPricesMap::<T>::get(market_id);

			Self::get_price(market_id, price.timestamp, price.index_price)
		}

		fn update_last_oracle_price(market_id: u128, price: FixedI128) {
			// Get the current timestamp
			let current_timestamp: u64 = T::TimeProvider::now().as_secs();

			let new_last_oracle_price = LastOraclePrice { timestamp: current_timestamp, price };

			// Update last traded price
			LastOraclePricesMap::<T>::insert(market_id, new_last_oracle_price);

			// Emits event
			Self::deposit_event(Event::LastOraclePriceUpdated {
				market_id,
				price: new_last_oracle_price,
			});
		}

		fn get_remaining_markets() -> Vec<u128> {
			let current_epoch = AbrEpoch::<T>::get();

			let markets = T::MarketPallet::get_all_markets_by_state(true, false);
			let mut remaining_markets = Vec::<u128>::new();

			// According to ABR state, return remaining markets
			match AbrState::<T>::get() {
				ABRState::State0 => markets,
				ABRState::State1 => {
					for market_id in markets {
						let market_status = AbrMarketStatusMap::<T>::get(current_epoch, market_id);
						if !market_status {
							remaining_markets.push(market_id);
						}
					}
					remaining_markets
				},
				ABRState::State2 => remaining_markets,
			}
		}

		fn get_no_of_batches_for_current_epoch() -> u64 {
			let current_epoch = AbrEpoch::<T>::get();

			// Return number of batches only if state is 2
			match AbrState::<T>::get() {
				ABRState::State2 => NoOfBatchesForEpochMap::<T>::get(current_epoch),
				_ => 0,
			}
		}

		fn get_last_abr_timestamp() -> u64 {
			let current_epoch = AbrEpoch::<T>::get();

			match AbrState::<T>::get() {
				ABRState::State0 =>
					if current_epoch == 0 || current_epoch == 1 {
						InitialisationTimestamp::<T>::get()
					} else {
						EpochToTimestampMap::<T>::get(current_epoch - 1)
					},
				_ => EpochToTimestampMap::<T>::get(current_epoch),
			}
		}

		fn get_remaining_pay_abr_calls() -> u64 {
			let current_epoch = AbrEpoch::<T>::get();
			let no_of_batches = NoOfBatchesForEpochMap::<T>::get(current_epoch);
			let batches_fetched = BatchesFetchedForEpochMap::<T>::get(current_epoch);

			match AbrState::<T>::get() {
				ABRState::State2 => no_of_batches - batches_fetched,
				_ => 0,
			}
		}

		fn convert_to_seconds(time_in_milli: u64) -> u64 {
			time_in_milli / MILLIS_PER_SECOND
		}

		fn get_next_abr_timestamp() -> u64 {
			let current_abr_interval = AbrInterval::<T>::get();
			let last_timestamp = Self::get_last_abr_timestamp();
			last_timestamp + current_abr_interval
		}

		fn get_previous_abr_values(
			market_id: u128,
			start_timestamp: u64,
			end_timestamp: u64,
		) -> Vec<ABRDetails> {
			let mut abr_details = Vec::<ABRDetails>::new();
			let start_epoch = Self::get_epoch_of_timestamp(start_timestamp);
			let end_epoch = AbrEpoch::<T>::get();
			if start_epoch == 0 {
				return abr_details
			}

			for epoch in start_epoch..end_epoch + 1 {
				let epoch_timestamp = EpochToTimestampMap::<T>::get(epoch);
				if epoch_timestamp > end_timestamp {
					return abr_details
				}

				let abr_value = EpochMarketToAbrValueMap::<T>::get(epoch, market_id);
				let abr_timestamp = EpochToTimestampMap::<T>::get(epoch);
				abr_details.push(ABRDetails { abr_value, abr_timestamp });
			}
			abr_details
		}

		fn get_intermediary_abr_value(market_id: u128) -> FixedI128 {
			// Check whether market exists
			let market = T::MarketPallet::get_market(market_id);
			if market.is_some() == false {
				return FixedI128::zero()
			}

			// Check whether market is tradable
			let market = market.unwrap();
			if market.is_tradable == false {
				return FixedI128::zero()
			}

			let current_epoch = AbrEpoch::<T>::get();
			if current_epoch == 0 {
				return FixedI128::zero()
			}

			// Check if the market's abr is already set
			let market_status = AbrMarketStatusMap::<T>::get(current_epoch, market_id);
			if market_status == true {
				return EpochMarketToAbrValueMap::<T>::get(current_epoch, market_id)
			}

			// Compute start and end timestamp
			let epoch_end_timestamp = T::TimeProvider::now().as_secs();
			let epoch_start_timestamp;
			if current_epoch == 1 {
				epoch_start_timestamp = InitialisationTimestamp::<T>::get();
			} else {
				epoch_start_timestamp = EpochToTimestampMap::<T>::get(current_epoch - 1);
			}

			// Fetch index and mark prices for the market
			let (index_prices, mark_prices) =
				Self::get_prices_for_abr(market_id, epoch_start_timestamp, epoch_end_timestamp);
			if index_prices.len() == 0 || mark_prices.len() == 0 {
				return FixedI128::zero()
			}

			// Fetch base ABR and bollinger width
			let base_abr = BaseAbr::<T>::get();
			let bollinger_width = BollingerWidth::<T>::get();

			// Calculate ABR
			let (abr_value, _) =
				Self::calculate_abr(mark_prices, index_prices, base_abr, bollinger_width, 8);

			return abr_value
		}

		fn get_remaining_prices_cleanup_calls() -> u64 {
			let start_timestamp = match PricesStartTimestamp::<T>::get() {
				Some(timestamp) => timestamp,
				None => return 0_u64,
			};

			let current_timestamp: u64 = T::TimeProvider::now().as_secs();
			let timestamp_limit: u64 = current_timestamp - FOUR_WEEKS;
			let cleanup_count = 3600;

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
	}
}
