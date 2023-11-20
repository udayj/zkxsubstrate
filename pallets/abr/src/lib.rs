#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use pallet_support::{traits::TradingAccountInterface, types::ABRState};
	use sp_arithmetic::fixed_point::FixedI128;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type TradingAccountPallet: TradingAccountInterface;
	}

	/// Stores the state of ABR
	#[pallet::storage]
	#[pallet::getter(fn abr_state)]
	pub(super) type AbrState<T: Config> = StorageValue<_, ABRState, ValueQuery>;

	/// Stores the epoch value
	#[pallet::storage]
	#[pallet::getter(fn abr_epoch)]
	pub(super) type AbrEpoch<T: Config> = StorageValue<_, u64, ValueQuery>;

	/// Stores the ABR Interval
	#[pallet::storage]
	#[pallet::getter(fn abr_interval)]
	pub(super) type AbrInterval<T: Config> = StorageValue<_, u64, ValueQuery>;

	/// Stores the no of users per batch
	#[pallet::storage]
	#[pallet::getter(fn users_per_batch)]
	pub(super) type UsersPerBatch<T: Config> = StorageValue<_, u128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn epoch_to_timestamp)]
	/// key - Epoch, value - timestamp
	pub(super) type EpochToTimestampMap<T: Config> =
		StorageMap<_, Twox64Concat, u64, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn no_of_batches_for_epoch)]
	/// key - Epoch, value - No.of batches
	pub(super) type NoOfBatchesForEpochMap<T: Config> =
		StorageMap<_, Twox64Concat, u64, u128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn batches_fetched_for_epoch)]
	/// key - Epoch, value - No.of batches fetched
	pub(super) type BatchesFetchedForEpochMap<T: Config> =
		StorageMap<_, Twox64Concat, u64, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn epoch_market_to_abr_value)]
	/// key1 - Epoch, Key2 - Market_id, value - ABR value
	pub(super) type EpochMarketToAbrValueMap<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, u64, Blake2_128Concat, u128, FixedI128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn epoch_market_to_last_price)]
	/// key1 - Epoch, Key2 - Market_id, value - Last market price
	pub(super) type EpochMarketToLastPriceMap<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, u64, Blake2_128Concat, u128, FixedI128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn abr_market_status)]
	/// key1 - Epoch, Key2 - Market_id, value - Status of the market
	pub(super) type AbrMarketStatusMap<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, u64, Blake2_128Concat, u128, bool, ValueQuery>;

	#[pallet::error]
	pub enum Error<T> {
		/// When timestamp provided is invalid
		InvalidTimestamp,
		/// When ABR state is invalid while setting timestamp
		InvalidState,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// ABR timestamp set successfully
		AbrTimestampSet { epoch: u64, timestamp: u64 },
		/// ABR state changed successfully
		AbrStateChanged { epoch: u64, state: ABRState },
	}

	// Pallet callable functions
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// External function to be called for setting ABR timestamp
		#[pallet::weight(0)]
		pub fn set_abr_timestamp(origin: OriginFor<T>, new_timestamp: u64) -> DispatchResult {
			// Make sure the caller is from a signed origin
			ensure_signed(origin)?;

			// Get current state, epoch and ABR interval
			let current_state = AbrState::<T>::get();
			let current_epoch = AbrEpoch::<T>::get();
			let current_abr_interval = AbrInterval::<T>::get();

			// ABR must be in state 0
			ensure!(current_state == ABRState::State0, Error::<T>::InvalidState);

			let last_timestamp = Self::get_last_abr_timestamp();

			// Enforces last_abr_timestamp + abr_interval < new_timestamp
			ensure!(
				last_timestamp + current_abr_interval <= new_timestamp,
				Error::<T>::InvalidTimestamp
			);

			let new_epoch;
			if current_epoch == 0 {
				new_epoch = current_epoch + 1;
				AbrEpoch::<T>::put(new_epoch);
			} else {
				new_epoch = current_epoch;
			}

			AbrState::<T>::put(ABRState::State1);
			EpochToTimestampMap::<T>::insert(new_epoch, new_timestamp);

			// Get no of users in a batch
			let users_per_batch = UsersPerBatch::<T>::get();

			// Get the no of batches
			let no_of_batches = Self::calculate_no_of_batches(users_per_batch);

			// Write the no of batches for this epoch
			NoOfBatchesForEpochMap::<T>::insert(new_epoch, no_of_batches);

			// Emit ABR timestamp set event
			Self::deposit_event(Event::AbrTimestampSet {
				epoch: new_epoch,
				timestamp: new_timestamp,
			});

			// Emit ABR state changed event
			Self::deposit_event(Event::AbrStateChanged {
				epoch: new_epoch,
				state: ABRState::State0,
			});

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn get_last_abr_timestamp() -> u64 {
			// Get current state and epoch
			let current_state = AbrState::<T>::get();
			let current_epoch = AbrEpoch::<T>::get();

			if current_state == ABRState::State0 {
				if current_epoch == 0 {
					return EpochToTimestampMap::<T>::get(current_epoch)
				} else {
					return EpochToTimestampMap::<T>::get(current_epoch - 1)
				}
			} else {
				return EpochToTimestampMap::<T>::get(current_epoch)
			}
		}

		fn calculate_no_of_batches(users_per_batch: u128) -> u128 {
			// Get the count of accounts
			let accounts_count = T::TradingAccountPallet::get_accounts_count();

			let q = accounts_count / users_per_batch;
			let r = accounts_count % users_per_batch;

			if r == 0 {
				return q
			} else {
				return q + 1
			}
		}
	}
}
