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
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use primitive_types::U256;
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
	pub(super) type MarketMap<T: Config> = StorageMap<_, Twox64Concat, U256, Market>;

	#[pallet::error]
	pub enum Error<T> {
		/// Each market must have a unique identifier
		DuplicateMarket,
		/// The total supply of markets can't exceed the u64 limit
		BoundsOverflow,
		/// Invalid value for Market Id
		InvalidMarketId,
		/// Invalid value for is_tradable field
		InvalidTradableFlag,
		/// Asset not created
		AssetNotFound,
		/// Asset provided as collateral is not marked as collateral in the system
		AssetNotCollateral,
		/// Invalid value for max leverage or currently allowed leverage
		InvalidLeverage,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Markets were successfully created
		MarketsCreated { length: u64 },
	}

	// Pallet callable functions
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Replace all markets
		#[pallet::weight(0)]
		pub fn replace_all_markets(origin: OriginFor<T>, markets: Vec<Market>) -> DispatchResult {
			// Make sure the caller is from a signed origin
			let _ = ensure_signed(origin)?;

			// Clear market map
			let _ = MarketMap::<T>::clear(DELETION_LIMIT, None);

			let length: u64 = u64::try_from(markets.len()).unwrap();

			// Iterate through the vector of markets and add to market map
			for element in markets {
				// Check if the market exists in the storage map
				ensure!(!MarketMap::<T>::contains_key(element.id), Error::<T>::DuplicateMarket);
				// Check market id is non zero
				ensure!(element.id > 0.into(), Error::<T>::InvalidMarketId);
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
	}

	impl<T: Config> MarketInterface for Pallet<T> {
		fn get_market(id: U256) -> Option<Market> {
			let result = MarketMap::<T>::try_get(id);
			match result {
				Ok(result) => return Some(result),
				Err(_) => return None,
			};
		}
	}
}
