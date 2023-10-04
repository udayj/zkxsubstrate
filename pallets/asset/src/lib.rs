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
	use scale_info::prelude::string::String;
	use zkx_support::traits::{AssetInterface, StringExt};
	use zkx_support::types::Asset;

	static DELETION_LIMIT: u32 = 100;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::storage]
	#[pallet::getter(fn assets_count)]
	pub(super) type AssetsCount<T: Config> = StorageValue<_, u64, ValueQuery>;

	/// Maps the Assets struct to the unique_id.
	#[pallet::storage]
	#[pallet::getter(fn assets)]
	pub(super) type AssetMap<T: Config> = StorageMap<_, Twox64Concat, u128, Asset>;

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
		AssetRemoved {
			asset: Asset,
		},
	}

	// Pallet callable functions
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Replace all assets
		#[pallet::weight(0)]
		pub fn replace_all_assets(origin: OriginFor<T>, assets: Vec<Asset>) -> DispatchResult {
			// Make sure the caller is from a signed origin
			let _ = ensure_signed(origin)?;

			// Clear asset map
			let _ = AssetMap::<T>::clear(DELETION_LIMIT, None);

			let length: u64 = u64::try_from(assets.len()).unwrap();

			// Iterate through the vector of assets and add to asset map
			for element in assets {
				// Check if the asset exists in the storage map
				ensure!(!AssetMap::<T>::contains_key(element.id), Error::<T>::DuplicateAsset);
				// Validate asset
				ensure!((0..19).contains(&element.token_decimal), Error::<T>::InvalidAsset);
				let name_string =
					String::from_utf8(element.name.to_vec()).expect("Found invalid UTF-8");
				let name_felt: u128 = name_string.as_str().to_felt_rep();
				ensure!(name_felt == element.id, Error::<T>::InvalidAsset);

				AssetMap::<T>::insert(element.id, element.clone());
			}

			AssetsCount::<T>::put(length);

			Self::deposit_event(Event::AssetsCreated { length });

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn add_asset(origin: OriginFor<T>, asset: Asset) -> DispatchResult {
			// Make sure the caller is from a signed origin
			let _ = ensure_signed(origin)?;

			// Get the number of assets available
			let length: u64 = AssetsCount::<T>::get();

			// Check if the asset exists in the storage map
			ensure!(!AssetMap::<T>::contains_key(asset.id), Error::<T>::DuplicateAsset);
			// Validate asset
			ensure!((0..19).contains(&asset.token_decimal), Error::<T>::InvalidAsset);
			let name_string = String::from_utf8(asset.name.to_vec()).expect("Found invalid UTF-8");
			let name_felt: u128 = name_string.as_str().to_felt_rep();
			ensure!(name_felt == asset.id, Error::<T>::InvalidAsset);

			// Add asset to the asset map
			AssetMap::<T>::insert(asset.id, asset.clone());

			// Increase the asset count
			AssetsCount::<T>::put(length + 1);

			Self::deposit_event(Event::AssetCreated { asset });

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn remove_asset(origin: OriginFor<T>, asset: Asset) -> DispatchResult {
			// Make sure the caller is from a signed origin
			let _ = ensure_signed(origin)?;

			// Get the number of assets available
			let length: u64 = AssetsCount::<T>::get();

			// Check if the asset exists in the storage map
			ensure!(AssetMap::<T>::contains_key(asset.id), Error::<T>::InvalidAsset);

			// Remove asset to the asset map
			AssetMap::<T>::remove(asset.id);

			// Decrease the asset count
			AssetsCount::<T>::put(length - 1);

			Self::deposit_event(Event::AssetRemoved { asset });

			Ok(())
		}
	}

	impl<T: Config> AssetInterface for Pallet<T> {
		fn get_default_collateral() -> u128 {
			1431520323_u128
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
