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
	use frame_system::EnsureRoot;
	use primitive_types::U256;
	use zkx_support::helpers;
	use zkx_support::types::SyncSignature;
	use zkx_support::{ecdsa_verify, FieldElement, Signature};

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::storage]
	#[pallet::getter(fn get_signer)]
	// k1 - index, v - signer's pub key
	pub(super) type Signers<T: Config> = StorageMap<_, Twox64Concat, u8, U256, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_batch_status)]
	// k1 - batch hash, v - true/false
	pub(super) type IsBatchProcessed<T: Config> =
		StorageMap<_, Twox64Concat, U256, bool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_sync_state)]
	// v - tuple of block number and block hash
	pub(super) type LastProcessed<T: Config> = StorageValue<_, (u64, U256), ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_signes_count)]
	// v - Length of signers array
	pub(super) type SignersCount<T: Config> = StorageValue<_, u8, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_next_signer_index)]
	// v - Index at which a new signer can be added
	pub(super) type NextSignerIndex<T: Config> = StorageValue<_, u8, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_signers_quorum)]
	// v - No of signers required for quorum
	pub(super) type SignersQuorum<T: Config> = StorageValue<_, u8, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {}

	#[pallet::error]
	pub enum Error<T> {
		/// Unauthorized call
		NotAdmin,
		/// Signer passed is 0
		ZeroSigner,
		/// No of signers less than required quorum
		InsufficientSigners,
	}

	// Pallet callable functions
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// External function to be called by admin to add a signer
		#[pallet::weight(0)]
		pub fn add_signer(origin: OriginFor<T>, pub_key: U256) -> DispatchResult {
			// Make sure the caller is an admin
			ensure_root(origin).map_err(|_| Error::<T>::NotAdmin)?;

			// The pub key cannot be 0
			ensure!(pub_key != 0.into(), Error::<T>::ZeroSigner);

			// Read the state of signers
			let new_index = NextSignerIndex::<T>::get();
			let prev_signers_count = SignersCount::<T>::get();

			// Update the state
			SignersCount::<T>::put(prev_signers_count + 1);
			NextSignerIndex::<T>::put(new_index + 1);
			Signers::<T>::insert(new_index, pub_key);

			// Return ok
			Ok(())
		}

		/// External function to be called by admin to remove a signer
		#[pallet::weight(0)]
		pub fn remove_signer(origin: OriginFor<T>, index: u8) -> DispatchResult {
			// Make sure the caller is an admin
			ensure_root(origin).map_err(|_| Error::<T>::NotAdmin)?;

			// Check if the signer exists
			let signer = Signers::<T>::get(index);
			ensure!(signer != 0.into(), Error::<T>::ZeroSigner);

			// Read the state of signers
			let signers_count = SignersCount::<T>::get();
			let signers_quorum = SignersQuorum::<T>::get();

			// Ensure there are enough signers remaining
			ensure!(signers_count - 1 >= signers_quorum, Error::<T>::InsufficientSigners);

			// Update the state
			SignersCount::<T>::put(signers_count - 1);

			let new_signer: U256 = 0.into();
			Signers::<T>::insert(index, new_signer);

			// Return ok
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn has_quorum(signatures: Vec<SyncSignature>, hash: FieldElement) -> bool {
			// Get the required data
			let total_len = SignersCount::<T>::get();
			let quorum = SignersQuorum::<T>::get();

			let mut iterator = 0;
			let mut valid_sigs = 0;

			loop {
				if iterator == total_len || valid_sigs == quorum {
					break;
				}

				// Get the corresponding signer pub key
				let curr_signature = &signatures[usize::from(iterator)];
				let pub_key = Signers::<T>::try_get(curr_signature.signer_index).unwrap();

				// Convert the data to felt252
				let pub_key_felt252 = helpers::u256_to_field_element(&pub_key).unwrap();
				let signature_felt252 = Signature {
					r: helpers::u256_to_field_element(&curr_signature.r).unwrap(),
					s: helpers::u256_to_field_element(&curr_signature.s).unwrap(),
				};

				// Check if the sig is valid, if yes increment valid_sigs
				let result = Self::verify_signature(pub_key_felt252, hash, signature_felt252);
				if result {
					valid_sigs += 1;
				}

				iterator += 1;
			}

			return valid_sigs == quorum;
		}

		fn verify_signature(
			public_key: FieldElement,
			hash: FieldElement,
			signature: Signature,
		) -> bool {
			match ecdsa_verify(&public_key, &hash, &signature) {
				Ok(_) => true,
				Err(_) => false,
			}
		}
	}
}
