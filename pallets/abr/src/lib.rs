#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use frame_support::{dispatch::Vec, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use pallet_support::{
		helpers::{fixed_pow, ln, max},
		traits::{MarketInterface, TradingAccountInterface},
		types::ABRState,
	};
	use sp_arithmetic::{fixed_point::FixedI128, traits::Zero};

	// Minimum ABR interval
	pub const ABR_INTERVAL_MIN: u64 = 3600;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type TradingAccountPallet: TradingAccountInterface;
		type MarketPallet: MarketInterface;
	}

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

	#[pallet::error]
	pub enum Error<T> {
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
	}
	
	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// ABR timestamp set successfully
		AbrTimestampSet { epoch: u64, timestamp: u64 },
		/// ABR state changed successfully
		AbrStateChanged { epoch: u64, state: ABRState },
		/// ABR value set successfully
		AbrValueSet { epoch: u64, market_id: u128, abr_value: FixedI128, abr_last_price: FixedI128 },
	}

	impl<T: Config> Pallet<T> {
		fn calculate_effective_abr(premiums: &[FixedI128]) -> FixedI128 {
			let mut premium_sum = FixedI128::zero();
			let total_len = premiums.len();

			for iterator in 0..total_len {
				premium_sum = premium_sum + premiums[iterator];
			}

			premium_sum / (FixedI128::from((total_len * 8) as i128))
		}

		fn calculate_premium(
			mark_prices: &[FixedI128],
			index_prices: &[FixedI128],
		) -> Vec<FixedI128> {
			// Initialize the result vector with the size of prices vector
			let total_len = mark_prices.len();
			let mut premiums = Vec::<FixedI128>::with_capacity(total_len);

			for iterator in 0..total_len {
				// TODO(merkle-groot): possibly check for division by zero error here
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
			let total_len = mark_prices.len();

			for iterator in 0..total_len {
				let upper_diff =
					max(FixedI128::zero(), mark_prices[iterator] - upper_band[iterator]);
				let lower_diff =
					max(FixedI128::zero(), lower_band[iterator] - mark_prices[iterator]);

				if upper_diff > FixedI128::zero() {
					premiums[iterator] = premiums[iterator] +
						max(ln(upper_diff) / index_prices[iterator], FixedI128::zero());
				} else if lower_diff > FixedI128::zero() {
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
			// Initialize the diff_sum
			let total_len = prices.len();
			let mut diff_sum = FixedI128::zero();

			// Handle Edge case
			if total_len == 0 {
				return FixedI128::zero();
			}

			for iterator in 0..total_len {
				let diff = prices[iterator] - mean;
				diff_sum = diff_sum + fixed_pow(diff, 2_u64);
			}

			boll_width * (diff_sum / FixedI128::from(total_len as i128)).sqrt()
		}

		fn calculate_bollinger_bands(
			prices: &[FixedI128],
			mean_prices: &[FixedI128],
			window: usize,
			boll_width: FixedI128,
		) -> (Vec<FixedI128>, Vec<FixedI128>) {
			// Initialize the result vector with the size of prices vector
			let total_len = prices.len();
			let mut upper_band = Vec::<FixedI128>::with_capacity(total_len);
			let mut lower_band = Vec::<FixedI128>::with_capacity(total_len);

			// Handle Edge case
			if total_len == 0 || window == 0 {
				return (upper_band, lower_band);
			}

			for iterator in 0..total_len {
				// Calculate the sliding mean till the iterator
				if iterator < window {
					// Calculate the standard deviation factor
					let std = Self::calculate_std(
						&prices[0..iterator + 1],
						mean_prices[iterator],
						boll_width,
					);

					// Add to lower and upper band vectors
					lower_band.push(mean_prices[iterator] - std);
					upper_band.push(mean_prices[iterator] + std);
				} else {
					// Calculate the standard deviation factor
					let std = Self::calculate_std(
						&prices[iterator - window + 1..iterator + 1],
						mean_prices[iterator],
						boll_width,
					);

					// Add to lower and upper band vectors
					lower_band.push(mean_prices[iterator] - std);
					upper_band.push(mean_prices[iterator] + std);
				}
			}

			(lower_band, upper_band)
		}

		pub fn calculate_sliding_mean(prices: &[FixedI128], window: usize) -> Vec<FixedI128> {
			// Initialize the result vector with the size of prices vector
			let total_len = prices.len();
			print!("the length is {:?}", total_len);
			let mut result = Vec::<FixedI128>::with_capacity(total_len);

			// Handle Edge case
			if total_len == 0 || window == 0 {
				return result;
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

		// fn calculate_abr(
		// 	mark_prices: Vec<FixedI128>,
		// 	index_prices: Vec<FixedI128>,
		// 	base_abr_rate: FixedI128,
		// 	boll_width: FixedI128,
		// 	window: usize,
		// )
		// // -> FixedI128
		// {
		// 	let mean_prices = Self::calculate_sliding_mean(&mark_prices, window);
		// 	print!("Mean prices {:?}", mean_prices);
		// 	// let (upper_band, lower_band) =
		// 	// 	Self::calculate_bollinger_bands(&mark_prices, &mean_prices, window, boll_width);
		// 	// let mut premiums = Self::calculate_premium(&mark_prices, &index_prices);
		// 	// let premiums_w_jumps = Self::calculate_jump(
		// 	// 	&mut premiums,
		// 	// 	&upper_band,
		// 	// 	&lower_band,
		// 	// 	&mark_prices,
		// 	// 	&index_prices,
		// 	// );

		// 	// Self::calculate_effective_abr(&premiums_w_jumps) + base_abr_rate
		// }
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
			Self::deposit_event(Event::AbrTimestampSet {
				epoch: new_epoch,
				timestamp: new_timestamp,
			});

			// Emit ABR state changed event
			Self::deposit_event(Event::AbrStateChanged {
				epoch: new_epoch,
				state: ABRState::State0,
			});

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

			// Calculate ABR
			let (abr_value, abr_last_price) = Self::calculate_abr(market_id);

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
	}

	impl<T: Config> Pallet<T> {
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

		fn calculate_abr(market_id: u128) -> (FixedI128, FixedI128) {
			return (FixedI128::zero(), FixedI128::zero())
		}

		fn check_abr_markets_status(epoch: u64) {
			// get all the markets available in the system
			let markets = T::MarketPallet::get_all_markets();

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
	}
}
