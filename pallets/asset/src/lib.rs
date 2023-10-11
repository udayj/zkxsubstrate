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
	use zkx_support::traits::AssetInterface;
	use zkx_support::types::Asset;

	static DELETION_LIMIT: u32 = 100;
	static DEFAULT_ASSET: u128 = 1431520323;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	/// Stores the number of valid assets in the system
	#[pallet::storage]
	#[pallet::getter(fn assets_count)]
	pub(super) type AssetsCount<T: Config> = StorageValue<_, u64, ValueQuery>;

	/// Maps the Assets struct to the unique_id.
	#[pallet::storage]
	#[pallet::getter(fn assets)]
	pub(super) type AssetMap<T: Config> = StorageMap<_, Twox64Concat, u128, Asset>;

	/// Stores the default collateral in the system
	#[pallet::storage]
	#[pallet::getter(fn default_collateral_asset)]
	pub(super) type DefaultCollateralAsset<T: Config> = StorageValue<_, u8, ValueQuery>;

	#[pallet::error]
	pub enum Error<T> {
		/// Each asset must have a unique identifier
		DuplicateAsset,
		/// The total supply of assets can't exceed the u64 limit
		BoundsOverflow,
		/// Invalid value for id or token decimal
		InvalidAsset,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Assets were successfully created
		AssetsCreated {
			length: u64,
		},
		AssetCreated {
			asset: Asset,
		},
		AssetUpdated {
			asset: Asset,
		},
		AssetRemoved {
			asset: Asset,
		},
	}

	// Pallet callable functions
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// TODO(merkle-groot): To be removed in production
		/// Replace all assets
		#[pallet::weight(0)]
		pub fn replace_all_assets(origin: OriginFor<T>, assets: Vec<Asset>) -> DispatchResult {
			ensure_signed(origin)?;

			// Clear asset map
			let _ = AssetMap::<T>::clear(DELETION_LIMIT, None);

			let length: u64 = u64::try_from(assets.len()).unwrap();

			// Iterate through the vector of assets and add to asset map
			for element in assets {
				// Check if the asset exists in the storage map
				ensure!(!AssetMap::<T>::contains_key(element.id), Error::<T>::DuplicateAsset);
				// Validate asset
				ensure!((0..19).contains(&element.decimals), Error::<T>::InvalidAsset);
				AssetMap::<T>::insert(element.id, element.clone());
			}

			AssetsCount::<T>::put(length);

			Self::deposit_event(Event::AssetsCreated { length });

			Ok(())
		}

		// TODO(merkle-groot): To be removed in production
		#[pallet::weight(0)]
		pub fn remove_asset(origin: OriginFor<T>, id: u128) -> DispatchResult {
			ensure_signed(origin)?;

			// Check if the asset exists
			if let None = Self::get_asset(id) {
				return Err(Error::<T>::InvalidAsset.into());
			}

			// Remove the asset
			Self::remove_asset_internal(id);
			Ok(())
		}

		// TODO(merkle-groot): To be removed in production
		#[pallet::weight(0)]
		pub fn update_asset(origin: OriginFor<T>, asset: Asset) -> DispatchResult {
			ensure_signed(origin)?;

			// Check if the asset exists
			if let None = Self::get_asset(asset.id) {
				return Err(Error::<T>::InvalidAsset.into());
			}

			// Validate asset
			ensure!((0..19).contains(&asset.decimals), Error::<T>::InvalidAsset);

			// Update the asset
			Self::update_asset_internal(asset);
			Ok(())
		}

		// TODO(merkle-groot): To be removed in production
		#[pallet::weight(0)]
		pub fn add_asset(origin: OriginFor<T>, asset: Asset) -> DispatchResult {
			ensure_signed(origin)?;

			// Check if the asset exists
			if let Some(_) = Self::get_asset(asset.id) {
				return Err(Error::<T>::DuplicateAsset.into());
			}

			// Validate asset
			ensure!((0..19).contains(&asset.decimals), Error::<T>::InvalidAsset);

			// Add the asset
			Self::add_asset_internal(asset);
			Ok(())
		}
	}

	impl<T: Config> AssetInterface for Pallet<T> {
		fn add_asset_internal(asset: Asset) {
			// Add asset to the asset map
			AssetMap::<T>::insert(asset.id, asset.clone());

			// Get the number of assets available
			// Increase the asset count
			let length: u64 = AssetsCount::<T>::get();
			AssetsCount::<T>::put(length + 1);

			// Emit the asset created event
			Self::deposit_event(Event::AssetCreated { asset });
		}
		
		fn update_asset_internal(asset: Asset) {
			// Replace the asset in the asset map
			AssetMap::<T>::insert(asset.id, asset.clone());

			// Emit the asset updated event
			Self::deposit_event(Event::AssetUpdated { asset });
		}

		fn remove_asset_internal(id: u128) {
			// Get the asset to be emitted in the event
			let asset = AssetMap::<T>::get(id).unwrap();

			// Remove asset from the asset map
			AssetMap::<T>::remove(id);

			// Get the number of assets available
			let length: u64 = AssetsCount::<T>::get();

			// Decrease the asset count
			AssetsCount::<T>::put(length - 1);

			// Emit the asset removed event
			Self::deposit_event(Event::AssetRemoved { asset });
		}

		fn get_default_collateral() -> u128 {
			DEFAULT_ASSET
		}

		fn get_asset(id: u128) -> Option<Asset> {
			let result = AssetMap::<T>::try_get(id);
			match result {
				Ok(result) => return Some(result),
				Err(_) => return None,
			};
		}
	}
}
