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
		types::{CurrentPrice, HistoricalPrice, MarketPrice, MultiplePrices},
	};
	use sp_arithmetic::{fixed_point::FixedI128, traits::Zero};

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type MarketPallet: MarketInterface;
		type TimeProvider: UnixTime;
	}

	/// Maps market id to the MarketPrice struct.
	#[pallet::storage]
	#[pallet::getter(fn market_price)]
	pub(super) type MarketPricesMap<T: Config> =
		StorageMap<_, Twox64Concat, u128, MarketPrice, ValueQuery>;

	/// Maps market id to the index and mark prices
	#[pallet::storage]
	#[pallet::getter(fn current_price)]
	pub(super) type CurrentPricesMap<T: Config> =
		StorageMap<_, Twox64Concat, u128, CurrentPrice, ValueQuery>;

	/// Maps index and mark prices according to timestamp
	#[pallet::storage]
	#[pallet::getter(fn historical_price)]
	pub(super) type HistoricalPricesMap<T: Config> =
		StorageMap<_, Twox64Concat, u64, HistoricalPrice, ValueQuery>;

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
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Market price was successfully updated
		MarketPriceUpdated { market_id: u128, price: MarketPrice },

		/// Multiple prices were successfully updated
		MultiplePricesUpdated { prices: Vec<MultiplePrices> },
	}

	// Pallet callable functions
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// update index and mark prices for several markets
		#[pallet::weight(0)]
		pub fn update_prices(origin: OriginFor<T>, prices: Vec<MultiplePrices>) -> DispatchResult {
			// Make sure the caller is from a signed origin
			ensure_signed(origin)?;

			// Get the current timestamp and last timestamp for which prices were updated
			let current_timestamp: u64 = T::TimeProvider::now().as_secs();
			let last_timestamp: u64 = LastTimestamp::<T>::get();
			let price_interval: u64 = PriceInterval::<T>::get();

			// Iterate through the vector of markets and add to market map
			for curr_market in &prices {
				ensure!(curr_market.index_price >= FixedI128::zero(), Error::<T>::InvalidPrice);
				ensure!(curr_market.mark_price >= FixedI128::zero(), Error::<T>::InvalidPrice);

				// Get Market from the corresponding Id
				let market = match T::MarketPallet::get_market(curr_market.market_id) {
					Some(m) => m,
					None => return Err(Error::<T>::MarketNotFound.into()),
				};

				// Create a struct object for the current price
				let new_price: CurrentPrice = CurrentPrice {
					timestamp: current_timestamp,
					index_price: curr_market.index_price,
					mark_price: curr_market.mark_price,
				};

				CurrentPricesMap::<T>::insert(curr_market.market_id, new_price);

				if (last_timestamp == 0) || (last_timestamp + price_interval <= current_timestamp) {
					// Update historical price
					let historical_price = HistoricalPrice {
						index_price: curr_market.index_price,
						mark_price: curr_market.mark_price,
					};
					HistoricalPricesMap::<T>::insert(current_timestamp, historical_price);

					PriceTimestamps::<T>::append(current_timestamp);
				}
			}

			// Emits event
			Self::deposit_event(Event::MultiplePricesUpdated { prices });

			Ok(())
		}
	}

	impl<T: Config> PricesInterface for Pallet<T> {
		fn get_market_price(market_id: u128) -> FixedI128 {
			let market_price = MarketPricesMap::<T>::get(market_id);
			// Get the current timestamp
			let current_timestamp: u64 = T::TimeProvider::now().as_secs();

			// Get Market from the corresponding Id
			let market = T::MarketPallet::get_market(market_id).unwrap();
			let time_difference = current_timestamp - market_price.timestamp;
			if time_difference > market.ttl.into() {
				FixedI128::zero()
			} else {
				market_price.price
			}
		}

		fn get_mark_price(market_id: u128) -> FixedI128 {
			let mark_price = CurrentPricesMap::<T>::get(market_id);
			// Get the current timestamp
			let current_timestamp: u64 = T::TimeProvider::now().as_secs();

			// Get Market from the corresponding Id
			let market = T::MarketPallet::get_market(market_id).unwrap();

			let time_difference = current_timestamp - mark_price.timestamp;
			if time_difference > market.ttl.into() {
				FixedI128::zero()
			} else {
				mark_price.mark_price
			}
		}

		fn get_index_price(market_id: u128) -> FixedI128 {
			let mark_price = CurrentPricesMap::<T>::get(market_id);
			// Get the current timestamp
			let current_timestamp: u64 = T::TimeProvider::now().as_secs();

			// Get Market from the corresponding Id
			let market = T::MarketPallet::get_market(market_id).unwrap();

			let time_difference = current_timestamp - mark_price.timestamp;
			if time_difference > market.ttl.into() {
				FixedI128::zero()
			} else {
				mark_price.index_price
			}
		}

		fn update_market_price(market_id: u128, price: FixedI128) {
			// Get the current timestamp
			let current_timestamp: u64 = T::TimeProvider::now().as_secs();

			// Create a struct object for the market prices
			let new_market_price: MarketPrice = MarketPrice { timestamp: current_timestamp, price };

			// Updates market_price
			MarketPricesMap::<T>::insert(market_id, new_market_price);

			// Emits event
			Self::deposit_event(Event::MarketPriceUpdated { market_id, price: new_market_price });
		}
	}
}
