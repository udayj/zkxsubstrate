#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use core::option::Option;
	use frame_support::{dispatch::Vec, pallet_prelude::*, traits::UnixTime};
	use frame_system::pallet_prelude::*;
	use pallet_support::{
		traits::{MarketInterface, PricesInterface},
		types::{CurrentPrice, HistoricalPrice, LastTradedPrice, MultiplePrices},
	};
	use sp_arithmetic::{fixed_point::FixedI128, traits::Zero};

	const MILLIS_PER_SECOND: u64 = 1000;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type MarketPallet: MarketInterface;
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

	/// Vector of timestamps for which historical prices are stored
	#[pallet::storage]
	#[pallet::getter(fn price_timestamps)]
	pub(super) type PriceTimestamps<T: Config> = StorageValue<_, Vec<u64>, ValueQuery>;

	/// Last timestamp for which index and mark prices were stored
	#[pallet::storage]
	#[pallet::getter(fn last_timestamp)]
	pub(super) type LastTimestamp<T: Config> = StorageValue<_, u64, ValueQuery>;

	/// Interval with which index and mark prices need to be stored
	#[pallet::storage]
	#[pallet::getter(fn price_interval)]
	pub(super) type PriceInterval<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::error]
	pub enum Error<T> {
		/// Invalid value for price
		InvalidPrice,
		/// Invalid value for Market Id
		MarketNotFound,
		/// Price interval should be >= 1 second
		InvalidPriceInterval,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Last traded price was successfully updated
		LastTradedPriceUpdated { market_id: u128, price: LastTradedPrice },

		/// Multiple prices were successfully updated
		MultiplePricesUpdated { prices: Vec<MultiplePrices> },
	}

	// Pallet callable functions
	#[pallet::call]
	impl<T: Config> Pallet<T> {
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
			let last_timestamp: u64 = LastTimestamp::<T>::get();
			let price_interval: u64 = PriceInterval::<T>::get();

			// Check whether historical prices needs to be updated
			let needs_update =
				(last_timestamp == 0) || (last_timestamp + price_interval <= timestamp);

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

				if needs_update {
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
			}

			if needs_update {
				PriceTimestamps::<T>::append(timestamp);
				LastTimestamp::<T>::put(timestamp);
			}

			// Emits event
			Self::deposit_event(Event::MultiplePricesUpdated { prices });

			Ok(())
		}

		/// update price interval with which historical prices should be stored
		#[pallet::weight(0)]
		pub fn update_price_interval(origin: OriginFor<T>, price_interval: u64) -> DispatchResult {
			// Make sure the caller is from a signed origin
			ensure_signed(origin)?;

			let price_interval = price_interval / MILLIS_PER_SECOND;

			ensure!(price_interval > 0, Error::<T>::InvalidPriceInterval);

			PriceInterval::<T>::put(price_interval);

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
			Self::deposit_event(Event::LastTradedPriceUpdated {
				market_id,
				price: new_last_traded_price,
			});
		}
	}
}
