#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use frame_support::inherent::Vec;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use primitive_types::U256;
	use zkx_support::helpers::pedersen_hash_multiple;
	use zkx_support::traits::{ArraySerialized, FieldElementExt, U256Ext};
	use zkx_support::types::{SyncSignature, UniversalEventL2};
	use zkx_support::{ecdsa_verify, FieldElement, Signature};

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::storage]
	#[pallet::getter(fn accounts_count)]
	// Array of U256
	pub(super) type Signers<T: Config> = StorageValue<_, Vec<U256>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_signer)]
	// k1 - U256, v - bool
	pub(super) type IsSignerWhitelisted<T: Config> =
		StorageMap<_, Twox64Concat, U256, bool, ValueQuery>;

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
	#[pallet::getter(fn get_signers_quorum)]
	// v - No of signers required for quorum
	pub(super) type SignersQuorum<T: Config> = StorageValue<_, u8, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Signer added by the admin successfully
		SignerAdded {
			signer: U256,
		},
		SignerRemoved {
			signer: U256,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Unauthorized call
		NotAdmin,
		/// Signer passed is 0
		ZeroSigner,
		/// Duplicate signer
		DuplicateSigner,
		/// No of signers less than required quorum
		InsufficientSigners,
		/// Signer not whitelisted
		SignerNotWhitelisted,
		/// No events provided
		EmptyBatch,
		/// Batch sent again
		DuplicateBatch,
		/// Not enough signatures for a sync tx
		InsufficientSignatures,
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
			ensure!(pub_key != U256::zero(), Error::<T>::ZeroSigner);

			// Ensure that the pub_key is not already whitelisted
			ensure!(!IsSignerWhitelisted::<T>::get(pub_key), Error::<T>::DuplicateSigner);

			// Store the new signer
			Signers::<T>::append(pub_key);
			IsSignerWhitelisted::<T>::insert(pub_key, true);

			// Emit the SignerAdded event
			Self::deposit_event(Event::SignerAdded { signer: pub_key });

			// Return ok
			Ok(())
		}

		/// External function to be called by admin to remove a signer
		#[pallet::weight(0)]
		pub fn remove_signer(origin: OriginFor<T>, pub_key: U256) -> DispatchResult {
			// Make sure the caller is an admin
			ensure_root(origin).map_err(|_| Error::<T>::NotAdmin)?;

			// Check if the signer exists
			ensure!(IsSignerWhitelisted::<T>::get(pub_key), Error::<T>::SignerNotWhitelisted);

			// Read the state of signers
			let signers_array = Signers::<T>::get();
			let signers_count = signers_array.len();
			let signers_quorum = SignersQuorum::<T>::get();

			// Ensure there are enough signers remaining
			ensure!(signers_count - 1 >= signers_quorum as usize, Error::<T>::InsufficientSigners);

			// remove the signer from the array
			let updated_array: Vec<U256> =
				signers_array.into_iter().filter(|&signer| signer != pub_key).collect();

			// Update the state
			IsSignerWhitelisted::<T>::insert(pub_key, false);
			Signers::<T>::put(updated_array);

			// Emit the SignerRemoved event
			Self::deposit_event(Event::SignerRemoved { signer: pub_key });

			// Return ok
			Ok(())
		}

		/// External function to be called by Synchronizer network to sync events from L2
		#[pallet::weight(0)]
		pub fn synchronize_events(
			origin: OriginFor<T>,
			events_batch: Vec<UniversalEventL2>,
			signatures: Vec<SyncSignature>,
			block_number: u64,
		) -> DispatchResult {
			// Make sure the caller is an admin
			ensure_root(origin).map_err(|_| Error::<T>::NotAdmin)?;

			// Check if there are events in the batch
			ensure!(events_batch.len() != 0, Error::<T>::EmptyBatch);

			// Compute the batch hash
			let batch_hash = Self::compute_batch_hash(&events_batch);

			// Check if the batch is already processed
			let batch_hash_u256 = batch_hash.to_u256();
			ensure!(
				IsBatchProcessed::<T>::get(batch_hash_u256) == false,
				Error::<T>::DuplicateBatch
			);

			// Check if there are enough sigs
			ensure!(Self::has_quorum(signatures, batch_hash), Error::<T>::InsufficientSignatures);

			// Mark the batch hash as being processed
			IsBatchProcessed::<T>::insert(batch_hash_u256, true);

			// Store the block number and the batch hash
			LastProcessed::<T>::put((block_number, batch_hash_u256));

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn has_quorum(signatures: Vec<SyncSignature>, hash: FieldElement) -> bool {
			// Get the required data
			let total_len = signatures.len();
			let quorum: u8 = SignersQuorum::<T>::get();

			let mut index = 0;
			let mut valid_sigs = 0;

			loop {
				if index == total_len || valid_sigs == quorum {
					break;
				}

				// Get the corresponding signer pub key
				let curr_signature: &SyncSignature = &signatures[usize::from(index)];

				// Convert the data to felt252
				let pub_key_felt252 = curr_signature.signer_pub_key.try_to_felt().unwrap();
				let signature_felt252 = Signature {
					r: curr_signature.r.try_to_felt().unwrap(),
					s: curr_signature.s.try_to_felt().unwrap(),
				};

				// Check if the sig is valid, if yes increment valid_sigs
				let result = Self::verify_signature(pub_key_felt252, hash, signature_felt252);
				if result {
					valid_sigs += 1;
				}

				index += 1;
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

		fn compute_batch_hash(events_batch: &Vec<UniversalEventL2>) -> FieldElement {
			// Convert the array of enums to array of felts
			let flattened_felt252_array = events_batch.serialize_to_felt_array().unwrap();

			// Compute hash of the array and return
			pedersen_hash_multiple(&flattened_felt252_array)
		}
	}
}
