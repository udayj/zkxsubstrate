#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use frame_system::PalletId;
	use zkx_support::traits::SyncFacadeInterface;
	use zkx_support::types::{FailedBatch, FailedOrder, ExecutedOrder, ExecutedBatch};

	// Pallet ID of Trading
	const TRADING_PALLET_ID: PalletId = PalletId(*b"trading");

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::error]
	pub enum Error<T> {
		FailedBatch: FailedBatch
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		ExecutedOrder: ExecutedOrder,
		FailedOrder: FailedOrder, 
		ExecutedBatch: ExecutedBatch
	}

	// Pallet callable functions
	impl<T: Config> SyncFacadeInterface for Pallet<T> {
		/// External function to be called to emit events
		#[pallet::weight(0)]
		pub fn syncfacade_emit(
			origin: OriginFor<T>,
			executed_orders: Vec<ExecutedOrder>,
			failed_orders: Vec<FailedOrder>,
			executed_batch: ExecutedBatch,
		) {
			// Check if the caller is the trading pallet
			ensure!(<frame_system::Pallet<T>>::origin() == Some(TRADING_PALLET_ID.into()), "Only trading pallet can call this function");
			
			Self::deposit_event(ExecutedBatch);

			for executed_order in &executed_orders {
				Self::deposit_event(executed_order);
			}

			for failed_order in &failed_orders {
				Self::deposit_event(executed_order);
			}

		}
	}
}
