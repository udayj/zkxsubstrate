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
	use frame_system::pallet_prelude::*;
	use pallet_support::{
		helpers::{fixed_pow, ln, max},
		traits::{
			AssetInterface, FixedI128Ext, MarketInterface, PricesInterface,
			TradingAccountInterface, TradingInterface,
		},
		types::{
			ABRDetails, ABRState, BalanceChangeReason, CurrentPrice, Direction, HistoricalPrice,
			LastTradedPrice, MultiplePrices, PositionExtended,
		},
	};
	use primitive_types::U256;
	use sp_arithmetic::{fixed_point::FixedI128, traits::Zero};

	// ////////////
	// Constants //
	// ////////////

	// To do checks for bollinger width
	const BOLLINGER_WIDTH_15: FixedI128 = FixedI128::from_inner(1500000000000000000);
	const BOLLINGER_WIDTH_20: FixedI128 = FixedI128::from_inner(2000000000000000000);
	const BOLLINGER_WIDTH_25: FixedI128 = FixedI128::from_inner(2500000000000000000);

	// To do checks for base_abr_rate
	const BASE_ABR_MIN: FixedI128 = FixedI128::from_inner(12500000000000);
	const BASE_ABR_MAX: FixedI128 = FixedI128::from_inner(100000000000000);

	// Minimum ABR interval
	const ABR_INTERVAL_MIN: u64 = 3600;
	// Price interval with which historical prices should be stored for ABR
	const ABR_PRICE_INTERVAL: u64 = 60;
	// To convert milliseconds to seconds
	const MILLIS_PER_SECOND: u64 = 1000;

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
	#[pallet::getter(fn last_traded_price)]
	// k1 - market_id, v - LastTradedPrice
	pub(super) type LastTradedPricesMap<T: Config> =
		StorageMap<_, Twox64Concat, u128, LastTradedPrice, ValueQuery>;

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
	pub(super) type UsersPerBatch<T: Config> = StorageValue<_, u128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn epoch_to_timestamp)]
	/// key - Epoch, value - timestamp
	pub(super) type EpochToTimestampMap<T: Config> =
		StorageMap<_, Twox64Concat, u64, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn no_of_batches_for_epoch)]
	/// key - Epoch, value - No.of batches
	pub(super) type NoOfBatchesForEpochMap<T: Config> =
		StorageMap<_, Twox64Concat, u64, u128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn batches_fetched_for_epoch)]
	/// key - Epoch, value - No.of batches fetched
	pub(super) type BatchesFetchedForEpochMap<T: Config> =
		StorageMap<_, Twox64Concat, u64, u128, ValueQuery>;

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
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Last traded price was successfully updated
		LastTradedPriceUpdated { market_id: u128, price: LastTradedPrice },
		/// ABR timestamp set successfully
		AbrTimestampSet { epoch: u64, timestamp: u64 },
		/// ABR state changed successfully
		AbrStateChanged { epoch: u64, state: ABRState },
		/// ABR value set successfully
		AbrValueSet { epoch: u64, market_id: u128, abr_value: FixedI128, abr_last_price: FixedI128 },
		/// ABR payment made successfully
		AbrPaymentMade { epoch: u64, batch_id: u128 },
	}

	// Pallet callable functions
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// External function to be called for setting ABR interval
		#[pallet::weight(0)]
		pub fn set_abr_interval(origin: OriginFor<T>, new_abr_interval: u64) -> DispatchResult {
			// Make sure the caller is from a signed origin
			ensure_signed(origin)?;

			//  ABR interval must be >= one hour
			ensure!(new_abr_interval >= ABR_INTERVAL_MIN, Error::<T>::InvalidAbrInterval);

			AbrInterval::<T>::put(new_abr_interval);
			Ok(())
		}

		/// External function to be called for setting base ABR
		#[pallet::weight(0)]
		pub fn set_base_abr(origin: OriginFor<T>, new_base_abr: FixedI128) -> DispatchResult {
			// Make sure the caller is from a signed origin
			ensure_signed(origin)?;

			//  Base ABR must be >= BASE_ABR_MIN and <= BASE_ABR_MAX
			ensure!(
				(new_base_abr <= BASE_ABR_MAX) && (new_base_abr >= BASE_ABR_MIN),
				Error::<T>::InvalidBaseAbr
			);

			BaseAbr::<T>::put(new_base_abr);
			Ok(())
		}

		/// External function to be called for setting bollinger width
		#[pallet::weight(0)]
		pub fn set_bollinger_width(
			origin: OriginFor<T>,
			new_bollinger_width: FixedI128,
		) -> DispatchResult {
			// Make sure the caller is from a signed origin
			ensure_signed(origin)?;

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
			Ok(())
		}

		/// External function to be called for setting no.of users per batch
		#[pallet::weight(0)]
		pub fn set_no_of_users_per_batch(
			origin: OriginFor<T>,
			new_no_of_users_per_batch: u128,
		) -> DispatchResult {
			// Make sure the caller is from a signed origin
			ensure_signed(origin)?;

			// No of users in a batch must be > 0
			ensure!(new_no_of_users_per_batch > 0, Error::<T>::InvalidUsersPerBatch);

			UsersPerBatch::<T>::put(new_no_of_users_per_batch);
			Ok(())
		}

		/// External function to be called for setting ABR timestamp
		#[pallet::weight(0)]
		pub fn set_abr_timestamp(origin: OriginFor<T>, new_timestamp: u64) -> DispatchResult {
			// Make sure the caller is from a signed origin
			ensure_signed(origin)?;

			// Get current state, epoch and ABR interval
			let current_state = AbrState::<T>::get();
			let current_epoch = AbrEpoch::<T>::get();
			let current_abr_interval = AbrInterval::<T>::get();

			// ABR must be in state 0
			ensure!(current_state == ABRState::State0, Error::<T>::InvalidState);

			let last_timestamp = Self::get_last_abr_timestamp();

			// Enforces last_abr_timestamp + abr_interval < new_timestamp
			ensure!(
				last_timestamp + current_abr_interval <= new_timestamp,
				Error::<T>::InvalidTimestamp
			);

			let new_epoch;
			if current_epoch == 0 {
				new_epoch = current_epoch + 1;
				AbrEpoch::<T>::put(new_epoch);
			} else {
				new_epoch = current_epoch;
			}

			AbrState::<T>::put(ABRState::State1);
			EpochToTimestampMap::<T>::insert(new_epoch, new_timestamp);

			// Get no of users in a batch
			let users_per_batch = UsersPerBatch::<T>::get();

			// Get the no of batches
			let no_of_batches = Self::calculate_no_of_batches(users_per_batch);

			// Write the no of batches for this epoch
			NoOfBatchesForEpochMap::<T>::insert(new_epoch, no_of_batches);

			// Emit ABR timestamp set event
			Self::deposit_event(
				Event::AbrTimestampSet { epoch: new_epoch, timestamp: new_timestamp }
			);

			// Emit ABR state changed event
			Self::deposit_event(
				Event::AbrStateChanged { epoch: new_epoch, state: ABRState::State1 }
			);

			Ok(())
		}

		/// External function to be called for setting ABR value
		#[pallet::weight(0)]
		pub fn set_abr_value(origin: OriginFor<T>, market_id: u128) -> DispatchResult {
			// Make sure the caller is from a signed origin
			ensure_signed(origin)?;

			// Get current state and epoch
			let current_state = AbrState::<T>::get();
			let current_epoch = AbrEpoch::<T>::get();
			let market_status = AbrMarketStatusMap::<T>::get(current_epoch, market_id);

			// Validate market
			let market = T::MarketPallet::get_market(market_id).unwrap();
			ensure!(market.is_tradable == true, Error::<T>::MarketNotTradable);

			// ABR must be in state 1
			ensure!(current_state == ABRState::State1, Error::<T>::InvalidState);

			// Check if the market's abr is already set
			ensure!(market_status == false, Error::<T>::AbrValueAlreadySet);

			// Fetch index and mark prices
			let (index_prices, mark_prices) = Self::get_prices_for_abr(market_id);

			// Fetch base ABR and bollinger width
			let base_abr = BaseAbr::<T>::get();
			let bollinger_width = BollingerWidth::<T>::get();

			// Calculate ABR
			let (abr_value, abr_last_price) =
				Self::calculate_abr(mark_prices, index_prices, base_abr, bollinger_width, 8);

			// Set the market's ABR value as true
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

			// Get the current timestamp and last timestamp for which prices were updated
			let timestamp = timestamp / MILLIS_PER_SECOND;

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
					let new_price: CurrentPrice =
						CurrentPrice {
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

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
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
				if current_epoch == 0 {
					return EpochToTimestampMap::<T>::get(current_epoch)
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
			let account_list = T::TradingAccountPallet::get_account_list(lower_limit, upper_limit);

			// Increment batches_fetched
			let new_batches_fetched = batches_fetched + 1;
			BatchesFetchedForEpochMap::<T>::insert(current_epoch, new_batches_fetched);

			// Emit ABR Payment made event
			Self::deposit_event(
				Event::AbrPaymentMade { epoch: current_epoch, batch_id: batches_fetched }
			);

			// If all batches are fetched, increment state and epoch
			if new_batches_fetched as u128 == no_of_batches {
				// Emit ABR state changed event
				Self::deposit_event(
					Event::AbrStateChanged { epoch: current_epoch, state: ABRState::State0 }
				);
				AbrState::<T>::put(ABRState::State0);
				AbrEpoch::<T>::put(current_epoch + 1);
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
					positions.retain(
						|position: &PositionExtended| position.created_timestamp <= timestamp
					);

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

						// Find if the abr_rate is +ve or -ve
						let mut payment_amount = abr_value * abr_last_price * position.size;
						if payment_amount < FixedI128::zero() {
							payment_amount.neg();
						};
						payment_amount =
							payment_amount.round_to_precision(collateral_token_decimal.into());
						// If the abr is negative
						if abr_value <= FixedI128::zero() {
							if position.direction == Direction::Short {
								Self::user_pays(user, collateral, payment_amount);
							} else {
								Self::user_receives(user, collateral, payment_amount);
							}
						} else {
							if position.direction == Direction::Short {
								Self::user_receives(user, collateral, payment_amount);
							} else {
								Self::user_pays(user, collateral, payment_amount);
							}
						}
					}
				}
			}
		}

		pub fn get_prices_for_abr(market_id: u128) -> (Vec<FixedI128>, Vec<FixedI128>) {
			let mut index_prices = Vec::<FixedI128>::new();
			let mut mark_prices = Vec::<FixedI128>::new();

			let current_epoch = AbrEpoch::<T>::get();
			let epoch_start_timestamp = EpochToTimestampMap::<T>::get(current_epoch);
			let abr_interval = AbrInterval::<T>::get();
			let epoch_end_timestamp = epoch_start_timestamp + abr_interval;
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
			let mut abr_last_price: FixedI128 = FixedI128::zero();
			if mark_prices.len() != 0 {
				abr_last_price = mark_prices[mark_prices.len() - 1];
			}
			return (abr_value, abr_last_price)
		}

		pub fn user_pays(user: U256, collateral: u128, payment_amount: FixedI128) {
			T::TradingAccountPallet::transfer_from(
				user,
				collateral,
				payment_amount,
				BalanceChangeReason::ABR,
			);
		}

		pub fn user_receives(user: U256, collateral: u128, payment_amount: FixedI128) {
			T::TradingAccountPallet::transfer(
				user,
				collateral,
				payment_amount,
				BalanceChangeReason::ABR,
			);
		}
	}

	impl<T: Config> PricesInterface for Pallet<T> {
		fn get_last_traded_price(market_id: u128) -> FixedI128 {
			let last_traded_price = LastTradedPricesMap::<T>::get(market_id);

			Self::get_price(market_id, last_traded_price.timestamp, last_traded_price.price)
		}

		fn get_mark_price(market_id: u128) -> FixedI128 {
			let price = CurrentPricesMap::<T>::get(market_id);

			Self::get_price(market_id, price.timestamp, price.mark_price)
		}

		fn get_index_price(market_id: u128) -> FixedI128 {
			let price = CurrentPricesMap::<T>::get(market_id);

			Self::get_price(market_id, price.timestamp, price.index_price)
		}

		fn update_last_traded_price(market_id: u128, price: FixedI128) {
			// Get the current timestamp
			let current_timestamp: u64 = T::TimeProvider::now().as_secs();

			let new_last_traded_price = LastTradedPrice { timestamp: current_timestamp, price };

			// Update last traded price
			LastTradedPricesMap::<T>::insert(market_id, new_last_traded_price);

			// Emits event
			Self::deposit_event(
				Event::LastTradedPriceUpdated { market_id, price: new_last_traded_price }
			);
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

		fn get_no_of_batches_for_current_epoch() -> u128 {
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
					if current_epoch == 0 {
						EpochToTimestampMap::<T>::get(current_epoch)
					} else {
						EpochToTimestampMap::<T>::get(current_epoch - 1)
					},
				_ => EpochToTimestampMap::<T>::get(current_epoch),
			}
		}

		fn get_remaining_pay_abr_calls() -> u128 {
			let current_epoch = AbrEpoch::<T>::get();
			let no_of_batches = NoOfBatchesForEpochMap::<T>::get(current_epoch);
			let batches_fetched: u128 = BatchesFetchedForEpochMap::<T>::get(current_epoch).into();

			match AbrState::<T>::get() {
				ABRState::State2 => no_of_batches - batches_fetched,
				_ => 0,
			}
		}

		fn get_next_abr_timestamp() -> u64 {
			let current_abr_interval = AbrInterval::<T>::get();
			let last_timestamp = Self::get_last_abr_timestamp();
			last_timestamp + current_abr_interval
		}

		fn get_previous_abr_values(
			starting_epoch: u64,
			market_id: u128,
			n: u64,
		) -> Vec<ABRDetails> {
			let mut abr_details = Vec::<ABRDetails>::new();
			let current_epoch = AbrEpoch::<T>::get();
			if (n == 0) || (current_epoch <= 1) {
				return abr_details
			}

			let mut epoch_iterator = starting_epoch;
			for iterator in 0..n {
				if current_epoch <= epoch_iterator {
					return abr_details
				}

				let abr_value = EpochMarketToAbrValueMap::<T>::get(epoch_iterator, market_id);
				let abr_timestamp = EpochToTimestampMap::<T>::get(epoch_iterator);
				abr_details.push(ABRDetails { abr_value, abr_timestamp });
				epoch_iterator += 1;
			}
			abr_details
		}
	}
}
