#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

// #[cfg(test)]
// mod tests;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use core::option::Option;
	use frame_support::{
		dispatch::Vec,
		pallet_prelude::{DispatchResult, *},
	};
	use frame_system::pallet_prelude::*;
	use pallet_support::{
		traits::{AssetInterface, MarketInterface, PricesInterface},
		types::{ExtendedMarket, Market},
	};
	use sp_arithmetic::fixed_point::FixedI128;

	static DELETION_LIMIT: u32 = 100;

	#[cfg(not(feature = "dev"))]
	pub const IS_DEV_ENABLED: bool = false;

	#[cfg(feature = "dev")]
	pub const IS_DEV_ENABLED: bool = true;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type AssetPallet: AssetInterface;
		type PricesPallet: PricesInterface;
	}

	#[pallet::storage]
	#[pallet::getter(fn markets_count)]
	pub(super) type MarketsCount<T: Config> = StorageValue<_, u64, ValueQuery>;

	/// Maps the Market struct to the unique_id.
	#[pallet::storage]
	#[pallet::getter(fn markets)]
	pub(super) type MarketMap<T: Config> = StorageMap<_, Twox64Concat, u128, ExtendedMarket>;

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
		/// Invalid Call to dev mode only function
		DevOnlyCall,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Market successfully created
		MarketCreated { market: ExtendedMarket },
		/// Market successfully updated
		MarketUpdated { market: ExtendedMarket },
		/// Market successfully removed
		MarketRemoved { market: ExtendedMarket },
	}

	// Pallet callable functions
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// TODO(merkle-groot): To be removed in production
		/// Replace all markets
		#[pallet::weight(0)]
		pub fn replace_all_markets(
			origin: OriginFor<T>,
			markets: Vec<ExtendedMarket>,
		) -> DispatchResult {
			if !IS_DEV_ENABLED {
				return Err(Error::<T>::DevOnlyCall.into());
			}
			ensure_signed(origin)?;

			// Clear market map
			let _ = MarketMap::<T>::clear(DELETION_LIMIT, None);

			let length: u64 = u64::try_from(markets.len()).unwrap();

			// Iterate through the vector of markets and add to market map
			for extended_market in markets {
				let current_market = extended_market.market.clone();
				// Check if the market exists in the storage map
				ensure!(
					!MarketMap::<T>::contains_key(current_market.id),
					Error::<T>::DuplicateMarket
				);
				// Check market id is non zero
				ensure!(current_market.id > 0, Error::<T>::InvalidMarket);
				// Validate asset and asset collateral
				let asset = T::AssetPallet::get_asset(current_market.asset);
				ensure!(asset.is_some(), Error::<T>::AssetNotFound);
				let asset_collateral = T::AssetPallet::get_asset(current_market.asset_collateral);
				ensure!(asset_collateral.is_some(), Error::<T>::AssetNotFound);
				ensure!(asset_collateral.unwrap().is_collateral, Error::<T>::AssetNotCollateral);
				ensure!(
					current_market.maximum_leverage >= current_market.minimum_leverage,
					Error::<T>::InvalidLeverage
				);
				ensure!(
					(current_market.minimum_leverage..
						current_market.maximum_leverage + FixedI128::from_inner(1))
						.contains(&current_market.currently_allowed_leverage),
					Error::<T>::InvalidLeverage
				);

				MarketMap::<T>::insert(current_market.id, extended_market.clone());

				Self::deposit_event(Event::MarketCreated { market: extended_market });
			}

			MarketsCount::<T>::put(length);

			Ok(())
		}

		// TODO(merkle-groot): To be removed in production
		#[pallet::weight(0)]
		pub fn add_market(origin: OriginFor<T>, extended_market: ExtendedMarket) -> DispatchResult {
			if !IS_DEV_ENABLED {
				return Err(Error::<T>::DevOnlyCall.into());
			}
			ensure_signed(origin)?;

			// Add the market
			Self::add_market_internal(extended_market)?;

			Ok(())
		}

		// TODO(merkle-groot): To be removed in production
		#[pallet::weight(0)]
		pub fn update_market(
			origin: OriginFor<T>,
			extended_market: ExtendedMarket,
		) -> DispatchResult {
			if !IS_DEV_ENABLED {
				return Err(Error::<T>::DevOnlyCall.into());
			}
			ensure_signed(origin)?;

			// Check if the market exists in the storage map
			ensure!(
				MarketMap::<T>::contains_key(extended_market.market.id),
				Error::<T>::InvalidMarket
			);

			// Add the market
			Self::update_market_internal(extended_market)?;

			Ok(())
		}

		// TODO(merkle-groot): To be removed in production
		#[pallet::weight(0)]
		pub fn remove_market(origin: OriginFor<T>, id: u128) -> DispatchResult {
			if !IS_DEV_ENABLED {
				return Err(Error::<T>::DevOnlyCall.into());
			}
			ensure_signed(origin)?;

			// Check if the market exists in the storage map
			ensure!(MarketMap::<T>::contains_key(id), Error::<T>::InvalidMarket);

			// Remove the market
			Self::remove_market_internal(id);
			Ok(())
		}
	}

	impl<T: Config> MarketInterface for Pallet<T> {
		fn add_market_internal(extended_market: ExtendedMarket) -> DispatchResult {
			// Check if the market exists in the storage map
			ensure!(
				!MarketMap::<T>::contains_key(extended_market.market.id),
				Error::<T>::DuplicateMarket
			);

			// Validate the market details
			Self::validate_market_details(&extended_market.market)?;

			// Add market to the market map
			MarketMap::<T>::insert(extended_market.market.id, extended_market.clone());

			// Increase the market count
			// Get the number of markets available
			let length: u64 = MarketsCount::<T>::get();
			MarketsCount::<T>::put(length + 1);

			// Emit the market created event
			Self::deposit_event(Event::MarketCreated { market: extended_market });

			Ok(())
		}

		fn update_market_internal(extended_market: ExtendedMarket) -> DispatchResult {
			// Validate the market details
			Self::validate_market_details(&extended_market.market)?;

			// Replace the market in the market map
			MarketMap::<T>::insert(extended_market.market.id, extended_market.clone());

			if extended_market.market.is_tradable == false {
				T::PricesPallet::set_mark_price_for_ads(extended_market.market.id)?;
			}

			// Emit the market updated event
			Self::deposit_event(Event::MarketUpdated { market: extended_market });

			Ok(())
		}

		fn remove_market_internal(id: u128) {
			// Get the market to be emitted in the event
			let extended_market = MarketMap::<T>::get(id).unwrap();

			// Remove market from the market map
			MarketMap::<T>::remove(id);

			// Decrease the market count
			// Get the number of markets available
			let length: u64 = MarketsCount::<T>::get();
			MarketsCount::<T>::put(length - 1);

			// Emit the market removed event
			Self::deposit_event(Event::MarketRemoved { market: extended_market });
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
				Ok(extended_market) => return Some(extended_market.market),
				Err(_) => return None,
			};
		}

		fn get_all_markets() -> Vec<u128> {
			let mut markets = Vec::<u128>::new();
			for (key, _) in MarketMap::<T>::iter() {
				markets.push(key);
			}
			markets
		}

		fn get_all_markets_by_state(is_tradable: bool, is_archived: bool) -> Vec<u128> {
			let mut markets = Vec::<u128>::new();
			for (key, value) in MarketMap::<T>::iter() {
				if value.market.is_tradable == is_tradable &&
					value.market.is_archived == is_archived
				{
					markets.push(key);
				}
			}
			markets
		}
	}
}
