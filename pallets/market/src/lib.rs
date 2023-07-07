#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

// #[cfg(test)]
// mod mock;
//
// #[cfg(test)]
// mod tests;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use core::option::Option;
	use frame_support::pallet_prelude::*;
	use frame_support::inherent::Vec;
	use frame_system::pallet_prelude::*;
	use sp_arithmetic::fixed_point::FixedI128;
	use scale_info::prelude::string::String;
	// use zkx_support::traits::AssetInterface;
	use zkx_support::str_to_felt;

	static DELETION_LIMIT: u32 = 100;

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[derive(Clone, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
	pub struct Market {
		pub id: u64,
		pub asset: u64,
		pub asset_collateral: u64,
		pub is_tradable: bool,
		pub is_archived: bool,
		pub ttl: u32,
		pub tick_size: u64,
		pub tick_precision: u8,
		pub step_size: u64,
		pub step_precision: u8,
		pub minimum_order_size: u64,
		pub minimum_leverage: u8,
		pub maximum_leverage: u8,
		pub currently_allowed_leverage: u8,
		pub maintenance_margin_fraction: u64,
		pub initial_margin_fraction: u64,
		pub incremental_initial_margin_fraction: u64,
		pub incremental_position_size: u64,
		pub baseline_position_size: u64,
		pub maximum_position_size: u64,
	}

	#[pallet::storage]
	#[pallet::getter(fn marketss_count)]
	pub(super) type MarketsCount<T: Config> = StorageValue<_, u64, ValueQuery>;

	/// Maps the Market struct to the unique_id.
	#[pallet::storage]
	#[pallet::getter(fn markets)]
	pub(super) type MarketMap<T: Config> = StorageMap<_, Twox64Concat, u64, Market>;

	#[pallet::error]
	pub enum Error<T> {
		/// Each market must have a unique identifier
		DuplicateMarket,
		/// The total supply of markets can't exceed the u64 limit
		BoundsOverflow,
		/// Invalid value for one of the fields
		InvalidMarket,
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
			let sender = ensure_signed(origin)?;

			// Clear market map
			let _ = MarketMap::<T>::clear(DELETION_LIMIT, None);

			let length: u64 = u64::try_from(markets.len()).unwrap();

			// Iterate through the vector of markets and add to market map
			for element in markets {
				// Check if the market exists in the storage map
				ensure!(!MarketMap::<T>::contains_key(element.id), Error::<T>::DuplicateMarket);
				MarketMap::<T>::insert(element.id, element.clone());
			}

			MarketsCount::<T>::put(length);

			Self::deposit_event(Event::MarketsCreated { length });

			Ok(())
		}
	}
}
