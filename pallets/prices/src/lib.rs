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
	use sp_arithmetic::{fixed_point::FixedI128, traits::Zero};
	use zkx_support::{
		traits::{MarketInterface, PricesInterface},
		types::{MultiplePrices, Price},
	};

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
		StorageMap<_, Twox64Concat, u128, Price, ValueQuery>;

	/// Maps market id to the IndexPrice struct.
	#[pallet::storage]
	#[pallet::getter(fn index_price)]
	pub(super) type IndexPricesMap<T: Config> =
		StorageMap<_, Twox64Concat, u128, Price, ValueQuery>;

	#[pallet::error]
	pub enum Error<T> {
		/// Invalid value for market price
		InvalidMarketPrice,
		/// Invalid value for Market Id
		MarketNotFound,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Market price was successfully updated
		MarketPriceUpdated { market_id: u128, price: Price },

		/// Multiple market prices were successfully updated
		MultipleMarketPricesUpdated { market_prices: Vec<MultiplePrices> },

		/// Multiple index prices were successfully updated
		MultipleIndexPricesUpdated { index_prices: Vec<MultiplePrices> },
	}

	// Pallet callable functions
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// update multiple market prices
		#[pallet::weight(0)]
		pub fn update_market_prices(
			origin: OriginFor<T>,
			market_prices: Vec<MultiplePrices>,
		) -> DispatchResult {
			// Make sure the caller is from a signed origin
			ensure_signed(origin)?;

			// Get the current timestamp
			let current_timestamp: u64 = T::TimeProvider::now().as_secs();

			// Iterate through the vector of markets and add to market map
			for curr_market in &market_prices {
				ensure!(curr_market.price >= FixedI128::zero(), Error::<T>::InvalidMarketPrice);

				// Get Market from the corresponding Id
				let market = match T::MarketPallet::get_market(curr_market.market_id) {
					Some(m) => m,
					None => return Err(Error::<T>::MarketNotFound.into()),
				};

				// Create a struct object for the market price
				let new_market_price: Price = Price {
					asset_id: market.asset,
					collateral_id: market.asset_collateral,
					timestamp: current_timestamp,
					price: curr_market.price,
				};

				MarketPricesMap::<T>::insert(curr_market.market_id, new_market_price);
			}

			// Emits event
			Self::deposit_event(Event::MultipleMarketPricesUpdated { market_prices });

			Ok(())
		}

		/// update multiple index prices
		#[pallet::weight(0)]
		pub fn update_index_prices(
			origin: OriginFor<T>,
			index_prices: Vec<MultiplePrices>,
		) -> DispatchResult {
			// Make sure the caller is from a signed origin
			ensure_signed(origin)?;

			// Get the current timestamp
			let current_timestamp: u64 = T::TimeProvider::now().as_secs();

			// Iterate through the vector of markets and add to market map
			for curr_market in &index_prices {
				ensure!(curr_market.price >= FixedI128::zero(), Error::<T>::InvalidMarketPrice);

				// Get Market from the corresponding Id
				let market = match T::MarketPallet::get_market(curr_market.market_id) {
					Some(m) => m,
					None => return Err(Error::<T>::MarketNotFound.into()),
				};

				// Create a struct object for the index price
				let new_index_price: Price = Price {
					asset_id: market.asset,
					collateral_id: market.asset_collateral,
					timestamp: current_timestamp,
					price: curr_market.price,
				};

				IndexPricesMap::<T>::insert(curr_market.market_id, new_index_price);
			}

			// Emits event
			Self::deposit_event(Event::MultipleIndexPricesUpdated { index_prices });

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
			let index_price = IndexPricesMap::<T>::get(market_id);
			// Get the current timestamp
			let current_timestamp: u64 = T::TimeProvider::now().as_secs();

			// Get Market from the corresponding Id
			let market = T::MarketPallet::get_market(market_id).unwrap();

			let time_difference = current_timestamp - index_price.timestamp;
			if time_difference > market.ttl.into() {
				FixedI128::zero()
			} else {
				index_price.price
			}
		}

		fn update_market_price(market_id: u128, price: FixedI128) {
			// Get the current timestamp
			let current_timestamp: u64 = T::TimeProvider::now().as_secs();

			// Get Market from the corresponding Id
			let market = T::MarketPallet::get_market(market_id);
			let market = market.unwrap();

			// Create a struct object for the market prices
			let new_market_price: Price = Price {
				asset_id: market.asset,
				collateral_id: market.asset_collateral,
				timestamp: current_timestamp,
				price,
			};

			// Updates market_price
			MarketPricesMap::<T>::insert(market_id, new_market_price);

			// Emits event
			Self::deposit_event(Event::MarketPriceUpdated { market_id, price: new_market_price });
		}
	}
}
