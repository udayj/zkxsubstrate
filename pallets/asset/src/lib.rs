#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use core::option::Option;
	use frame_support::pallet_prelude::*;
	use frame_support::inherent::Vec;
	use frame_system::pallet_prelude::*;
	use zkx_support::traits::AssetInterface;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[derive(Clone, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
	pub struct Asset {
		pub id: u8,
		pub name: Vec<u8>,
		pub is_tradable: bool,
		pub is_collateral: bool,
		pub token_decimal: u8,
	}

	#[pallet::storage]
	#[pallet::getter(fn assets_count)]
	pub(super) type AssetsCount<T: Config> = StorageValue<_, u64, ValueQuery>;

	/// Maps the Assets struct to the unique_id.
	#[pallet::storage]
	#[pallet::getter(fn assets)]
	pub(super) type AssetMap<T: Config> = StorageMap<_, Twox64Concat, u8, Asset>;

	#[pallet::storage]
	#[pallet::getter(fn default_collateral_asset)]
	pub(super) type DefaultCollateralAsset<T: Config> = StorageValue<_, u8, ValueQuery>;

	#[pallet::error]
	pub enum Error<T> {
		/// Each asset must have a unique identifier
		DuplicateAsset,
		/// The total supply of collectibles can't exceed the u64 limit
		BoundsOverflow,
		/// Asset does not exist
		AssetNotFound,
		/// Asset is not a collateral
		AssetNotCollateral,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new asset was successfully created
		AssetCreated { id: u8, asset: Asset },
		/// Default collateral asset modified
		DefaultCollateralModified { id: u8 },
	}

	// pub trait AssetInterface {
	// 	fn get_default_collateral() -> u8;
	// }

	// Pallet callable functions
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new asset.
		#[pallet::weight(0)]
		pub fn add_asset(
			origin: OriginFor<T>,
			id: u8,
			name: Vec<u8>,
			is_tradable: bool,
			is_collateral: bool,
			token_decimal: u8,
		) -> DispatchResult {
			// Make sure the caller is from a signed origin
			let sender = ensure_signed(origin)?;

			// Create a new object
			let asset = Asset { id, name, is_tradable, is_collateral, token_decimal };

			// Check if the asset exists in the storage map
			ensure!(!AssetMap::<T>::contains_key(&asset.id), Error::<T>::DuplicateAsset);

			// Increment the count of the asset
			let count = AssetsCount::<T>::get();
			let new_count = count.checked_add(1).ok_or(Error::<T>::BoundsOverflow)?;

			// Write new asset to storage and update the count
			AssetMap::<T>::insert(asset.id, asset.clone());
			AssetsCount::<T>::put(new_count);

			// Deposit the "AssetCreated" event.
			Self::deposit_event(Event::AssetCreated { id, asset });

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn modify_default_collateral(
			origin: OriginFor<T>,
			id: u8,
		) -> DispatchResult {
			// Make sure the caller is from a signed origin
			let sender = ensure_signed(origin)?;

			// Check if the asset exists in the storage map
			ensure!(AssetMap::<T>::contains_key(id), Error::<T>::AssetNotFound);

			// Get asset using id and set it as default collateral
			let asset = AssetMap::<T>::get(id).unwrap();
			ensure!(asset.is_collateral == true, Error::<T>::AssetNotCollateral);
			DefaultCollateralAsset::<T>::put(id);

			// Deposit the "AssetCreated" event.
			Self::deposit_event(Event::DefaultCollateralModified { id });

			Ok(())
		}
	}
	
	impl<T: Config> AssetInterface for Pallet<T> {
		fn get_default_collateral() -> u8 {
			DefaultCollateralAsset::<T>::get()
		}
	}
}
