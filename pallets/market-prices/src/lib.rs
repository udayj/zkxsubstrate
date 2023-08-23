#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

// #[cfg(test)]
// mod tests;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use core::option::Option;
	use frame_support::inherent::Vec;
	use frame_support::pallet_prelude::*;
	use frame_support::traits::UnixTime;
	use frame_system::pallet_prelude::*;
	use primitive_types::U256;
	use sp_arithmetic::fixed_point::FixedI128;
	use zkx_support::traits::{MarketInterface, MarketPricesInterface};
	use zkx_support::types::{MarketPrice, MultipleMarketPrices};

	static DELETION_LIMIT: u32 = 100;

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
		StorageMap<_, Twox64Concat, U256, MarketPrice, ValueQuery>;

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
		MarketPriceUpdated { market_id: U256, price: MarketPrice },

		/// Multiple market prices were successfully updated
		MultipleMarketPricesUpdated { market_prices: Vec<MultipleMarketPrices> },
	}

	// Pallet callable functions
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// update market price
		#[pallet::weight(0)]
		pub fn update_market_price(
			origin: OriginFor<T>,
			market_id: U256,
			price: FixedI128,
		) -> DispatchResult {
			// Make sure the caller is from a signed origin
			let _ = ensure_signed(origin)?;

			ensure!(price >= 0.into(), Error::<T>::InvalidMarketPrice);

			// Get the current timestamp
			let current_timestamp: u64 = T::TimeProvider::now().as_secs();

			// Get Market from the corresponding Id
			let market = T::MarketPallet::get_market(market_id).unwrap();
			ensure!(market.asset == 0.into(), Error::<T>::MarketNotFound);

			// Create a struct object for the market prices
			let new_market_price: MarketPrice = MarketPrice {
				asset_id: market.asset,
				collateral_id: market.asset_collateral,
				timestamp: current_timestamp,
				price,
			};

			MarketPricesMap::<T>::insert(market_id, new_market_price);

			Self::deposit_event(Event::MarketPriceUpdated { market_id, price: new_market_price });

			Ok(())
		}

		/// update multiple market prices
		#[pallet::weight(0)]
		pub fn update_multiple_market_prices(
			origin: OriginFor<T>,
			market_prices: Vec<MultipleMarketPrices>,
		) -> DispatchResult {
			// Make sure the caller is from a signed origin
			let _ = ensure_signed(origin)?;

			// Clear market prices map
			let _ = MarketPricesMap::<T>::clear(DELETION_LIMIT, None);

			// Get the current timestamp
			let current_timestamp: u64 = T::TimeProvider::now().as_secs();

			// Iterate through the vector of markets and add to market map
			for curr_market in &market_prices {
				ensure!(curr_market.price >= 0.into(), Error::<T>::InvalidMarketPrice);

				// Get Market from the corresponding Id
				let market = T::MarketPallet::get_market(curr_market.market_id).unwrap();
				ensure!(market.asset == 0.into(), Error::<T>::MarketNotFound);

				// Create a struct object for the market price
				let new_market_price: MarketPrice = MarketPrice {
					asset_id: market.asset,
					collateral_id: market.asset_collateral,
					timestamp: current_timestamp,
					price: curr_market.price,
				};

				MarketPricesMap::<T>::insert(curr_market.market_id, new_market_price);
			}

			Self::deposit_event(Event::MultipleMarketPricesUpdated { market_prices });

			Ok(())
		}
	}

	impl<T: Config> MarketPricesInterface for Pallet<T> {
		fn get_market_price(market_id: U256) -> FixedI128 {
			let market_price = MarketPricesMap::<T>::get(market_id);
			// Get the current timestamp
			let current_timestamp: u64 = T::TimeProvider::now().as_secs();

			// Get Market from the corresponding Id
			let market = T::MarketPallet::get_market(market_id).unwrap();
			let ttl = market.ttl;
			let timestamp = market_price.timestamp;
			let time_difference = current_timestamp - timestamp;
			if time_difference > ttl.into() {
				0.into()
			} else {
				market_price.price
			}
		}
	}
}
