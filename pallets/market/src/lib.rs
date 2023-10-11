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
	use frame_support::pallet_prelude::{DispatchResult, *};
	use frame_system::pallet_prelude::*;
	use sp_arithmetic::fixed_point::FixedI128;
	use zkx_support::traits::{AssetInterface, MarketInterface};
	use zkx_support::types::Market;

	static DELETION_LIMIT: u32 = 100;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type AssetPallet: AssetInterface;
	}

	#[pallet::storage]
	#[pallet::getter(fn markets_count)]
	pub(super) type MarketsCount<T: Config> = StorageValue<_, u64, ValueQuery>;

	/// Maps the Market struct to the unique_id.
	#[pallet::storage]
	#[pallet::getter(fn markets)]
	pub(super) type MarketMap<T: Config> = StorageMap<_, Twox64Concat, u128, Market>;

	#[pallet::error]
	pub enum Error<T> {
		/// Each market must have a unique identifier
		DuplicateMarket,
		/// The total supply of markets can't exceed the u64 limit
		BoundsOverflow,
		/// Invalid value for Market Id
		InvalidMarket,
		/// Invalid value for is_tradable field
		InvalidTradableFlag,
		/// Asset not created
		AssetNotFound,
		/// Asset provided as collateral is not marked as collateral in the system
		AssetNotCollateral,
		/// Invalid value for max leverage or currently allowed leverage
		InvalidLeverage,
		/// Unauthorized attempt to update markets
		NotAdmin,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Markets were successfully created
		MarketsCreated { length: u64 },
		/// Market successfully created
		MarketCreated { market: Market },
		/// Market successfully updated
		MarketUpdated { market: Market },
		/// Market successfully removed
		MarketRemoved { market: Market },
	}

	// Pallet callable functions
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// TODO(merkle-groot): To be removed in production
		/// Replace all markets
		#[pallet::weight(0)]
		pub fn replace_all_markets(origin: OriginFor<T>, markets: Vec<Market>) -> DispatchResult {
			ensure_signed(origin)?;

			// Clear market map
			let _ = MarketMap::<T>::clear(DELETION_LIMIT, None);

			let length: u64 = u64::try_from(markets.len()).unwrap();

			// Iterate through the vector of markets and add to market map
			for element in markets {
				// Check if the market exists in the storage map
				ensure!(!MarketMap::<T>::contains_key(element.id), Error::<T>::DuplicateMarket);
				// Check market id is non zero
				ensure!(element.id > 0, Error::<T>::InvalidMarket);
				// Validate asset and asset collateral
				let asset = T::AssetPallet::get_asset(element.asset);
				ensure!(asset.is_some(), Error::<T>::AssetNotFound);
				let asset_collateral = T::AssetPallet::get_asset(element.asset_collateral);
				ensure!(asset_collateral.is_some(), Error::<T>::AssetNotFound);
				ensure!(asset_collateral.unwrap().is_collateral, Error::<T>::AssetNotCollateral);
				ensure!(
					element.maximum_leverage >= element.minimum_leverage,
					Error::<T>::InvalidLeverage
				);
				ensure!(
					(element.minimum_leverage..element.maximum_leverage + FixedI128::from_inner(1))
						.contains(&element.currently_allowed_leverage),
					Error::<T>::InvalidLeverage
				);

				MarketMap::<T>::insert(element.id, element.clone());
			}

			MarketsCount::<T>::put(length);

			Self::deposit_event(Event::MarketsCreated { length });

			Ok(())
		}

		// TODO(merkle-groot): To be removed in production
		#[pallet::weight(0)]
		pub fn add_market(origin: OriginFor<T>, market: Market) -> DispatchResult {
			ensure_signed(origin)?;

			// Check if the market exists in the storage map
			ensure!(!MarketMap::<T>::contains_key(market.id), Error::<T>::DuplicateMarket);

			// Validate the market details
			Self::validate_market_details(&market)?;

			// Add the market
			Self::add_market_internal(market);

			Ok(())
		}

		// TODO(merkle-groot): To be removed in production
		#[pallet::weight(0)]
		pub fn update_market(origin: OriginFor<T>, market: Market) -> DispatchResult {
			ensure_signed(origin)?;

			// Check if the market exists in the storage map
			ensure!(MarketMap::<T>::contains_key(market.id), Error::<T>::InvalidMarket);

			// Validate the market details
			Self::validate_market_details(&market)?;

			// Add the market
			Self::update_market_internal(market);

			Ok(())
		}

		// TODO(merkle-groot): To be removed in production
		#[pallet::weight(0)]
		pub fn remove_market(origin: OriginFor<T>, id: u128) -> DispatchResult {
			ensure_signed(origin)?;

			// Check if the market exists in the storage map
			ensure!(MarketMap::<T>::contains_key(id), Error::<T>::InvalidMarket);

			// Remove the market
			Self::remove_market_internal(id);
			Ok(())
		}
	}

	impl<T: Config> MarketInterface for Pallet<T> {
		fn add_market_internal(market: Market) {
			// Add market to the market map
			MarketMap::<T>::insert(market.id, market.clone());

			// Increase the market count
			// Get the number of markets available
			let length: u64 = MarketsCount::<T>::get();
			MarketsCount::<T>::put(length + 1);

			// Emit the market created event
			Self::deposit_event(Event::MarketCreated { market });
		}

		fn update_market_internal(market: Market) {
			// Replace the market in the market map
			MarketMap::<T>::insert(market.id, market.clone());

			// Emit the market updated event
			Self::deposit_event(Event::MarketUpdated { market });
		}

		fn remove_market_internal(id: u128) {
			// Get the market to be emitted in the event
			let market = MarketMap::<T>::get(id).unwrap();

			// Remove market from the market map
			MarketMap::<T>::remove(id);

			// Decrease the market count
			// Get the number of markets available
			let length: u64 = MarketsCount::<T>::get();
			MarketsCount::<T>::put(length - 1);

			// Emit the market removed event
			Self::deposit_event(Event::MarketRemoved { market });
		}

		fn validate_market_details(market: &Market) -> DispatchResult {
			// Check Market properties
			// Check market id is non zero
			ensure!(market.id > 0, Error::<T>::InvalidMarket);
			ensure!(
				market.maximum_leverage >= market.minimum_leverage,
				Error::<T>::InvalidLeverage
			);
			ensure!(
				(market.minimum_leverage..market.maximum_leverage + FixedI128::from_inner(1))
					.contains(&market.currently_allowed_leverage),
				Error::<T>::InvalidLeverage
			);

			// Check Asset properties
			// Validate asset and asset collateral
			let asset = T::AssetPallet::get_asset(market.asset);
			ensure!(asset.is_some(), Error::<T>::AssetNotFound);
			let asset_collateral = T::AssetPallet::get_asset(market.asset_collateral);
			ensure!(asset_collateral.is_some(), Error::<T>::AssetNotFound);
			ensure!(asset_collateral.unwrap().is_collateral, Error::<T>::AssetNotCollateral);

			Ok(())
		}

		fn get_market(id: u128) -> Option<Market> {
			let result = MarketMap::<T>::try_get(id);
			match result {
				Ok(result) => return Some(result),
				Err(_) => return None,
			};
		}
	}
}
