#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use frame_support::{dispatch::Vec, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use pallet_support::{
		ecdsa_verify,
		helpers::pedersen_hash_multiple,
		traits::{
			AssetInterface, FeltSerializedArrayExt, FieldElementExt, MarketInterface,
			TradingAccountInterface, U256Ext,
		},
		types::{ExtendedAsset, ExtendedMarket, SyncSignature, UniversalEvent},
		FieldElement, Signature,
	};
	use primitive_types::U256;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type TradingAccountPallet: TradingAccountInterface;
		type AssetPallet: AssetInterface;
		type MarketPallet: MarketInterface;
	}

	#[pallet::storage]
	#[pallet::getter(fn signers)]
	// Array of U256
	pub(super) type Signers<T: Config> = StorageValue<_, Vec<U256>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn is_signer_valid)]
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
	pub(super) type LastProcessed<T: Config> = StorageValue<_, (u64, u32, U256), ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_signers_quorum)]
	// v - No of signers required for quorum
	pub(super) type SignersQuorum<T: Config> = StorageValue<_, u8, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Signer added by the admin successfully
		SignerAdded { signer: U256 },
		/// Signer removed by the admin succesfully
		SignerRemoved { signer: U256 },
		/// New Quorum requirement set by the admin
		QuorumSet { quorum: u8 },
		/// A invalid request to remove non-existent market
		MarketRemovedError { id: u128 },
		/// An invalid request to remove non-existent asset
		AssetRemovedError { id: u128 },
		/// An invalid request to remove non-existent signer
		SignerRemovedError { pub_key: U256 },
		/// An invalid request to set a signer
		QuorumSetError { quorum: u8 },
	}

	#[pallet::error]
	pub enum Error<T> {
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
		/// Old Batch sent
		OldBatch,
		/// Not enough signatures for a sync tx
		InsufficientSignatures,
		/// Invalid FieldElement value
		ConversionError,
	}

	// Pallet callable functions
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// TODO(merkle-groot): To be removed in production
		#[pallet::weight(0)]
		pub fn add_signer(origin: OriginFor<T>, pub_key: U256) -> DispatchResult {
			ensure_signed(origin)?;

			// The pub key cannot be 0
			ensure!(pub_key != U256::zero(), Error::<T>::ZeroSigner);

			// Ensure that the pub_key is not already whitelisted
			ensure!(!IsSignerWhitelisted::<T>::get(pub_key), Error::<T>::DuplicateSigner);

			// Store the new signer
			Self::add_signer_internal(pub_key);

			// Return ok
			Ok(())
		}

		// TODO(merkle-groot): To be removed in production
		#[pallet::weight(0)]
		pub fn set_signers_quorum(origin: OriginFor<T>, new_quorum: u8) -> DispatchResult {
			ensure_signed(origin)?;

			// It cannot be more than existing number of signers
			ensure!(new_quorum <= Signers::<T>::get().len() as u8, Error::<T>::InsufficientSigners);

			// Store the new qourum
			Self::set_signers_quorum_internal(new_quorum);

			// Return ok
			Ok(())
		}

		// TODO(merkle-groot): To be removed in production
		#[pallet::weight(0)]
		pub fn remove_signer(origin: OriginFor<T>, pub_key: U256) -> DispatchResult {
			ensure_signed(origin)?;

			// Check if the signer exists
			ensure!(IsSignerWhitelisted::<T>::get(pub_key), Error::<T>::SignerNotWhitelisted);

			// Read the state of signers
			let signers_array = Signers::<T>::get();
			let signers_count = signers_array.len();
			let signers_quorum = SignersQuorum::<T>::get();

			// Ensure there are enough signers remaining
			ensure!(signers_count - 1 >= signers_quorum as usize, Error::<T>::InsufficientSigners);

			// Update the state
			Self::remove_signer_internal(pub_key);

			// Return ok
			Ok(())
		}

		/// External function to be called by Synchronizer network to sync events from L2
		#[pallet::weight(0)]
		pub fn synchronize_events(
			origin: OriginFor<T>,
			events_batch: Vec<UniversalEvent>,
			signatures: Vec<SyncSignature>,
			// block_number: u64,
		) -> DispatchResult {
			// Make sure the call is signed
			ensure_signed(origin)?;

			// Check if there are events in the batch
			ensure!(events_batch.len() != 0, Error::<T>::EmptyBatch);

			// Fetch the block number of last event in the batch
			// Unwrap will not fail since we are checking for 0 length in previous line
			let (block_number, event_index) =
				Self::get_block_and_event_number(events_batch.last().unwrap());

			// The block number shouldn't be less than previous batch's block number
			let (last_block_number, _, _) = LastProcessed::<T>::get();
			ensure!(block_number >= last_block_number, Error::<T>::OldBatch);

			// Compute the batch hash
			let batch_hash = Self::compute_batch_hash(&events_batch)?;

			// Check if the batch is already processed
			let batch_hash_u256 = batch_hash.to_u256();
			ensure!(
				IsBatchProcessed::<T>::get(batch_hash_u256) == false,
				Error::<T>::DuplicateBatch
			);

			// Check if there are enough sigs
			ensure!(Self::has_quorum(signatures, batch_hash), Error::<T>::InsufficientSignatures);

			// Handle the events
			Self::handle_events(events_batch);

			// Mark the batch hash as being processed
			IsBatchProcessed::<T>::insert(batch_hash_u256, true);

			// Store the block number and the batch hash
			LastProcessed::<T>::put((block_number, event_index, batch_hash_u256));

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn add_signer_internal(pub_key: U256) {
			// Store the new signer
			Signers::<T>::append(pub_key);
			IsSignerWhitelisted::<T>::insert(pub_key, true);

			// Emit the SignerAdded event
			Self::deposit_event(Event::SignerAdded { signer: pub_key });
		}

		fn set_signers_quorum_internal(new_quorum: u8) {
			// Update the state
			SignersQuorum::<T>::put(new_quorum);

			// Emit the QuorumSet event
			Self::deposit_event(Event::QuorumSet { quorum: new_quorum });
		}

		fn remove_signer_internal(pub_key: U256) {
			// Read the state of signers
			let signers_array = Signers::<T>::get();

			// remove the signer from the array
			let updated_array: Vec<U256> =
				signers_array.into_iter().filter(|&signer| signer != pub_key).collect();

			// Update the state
			IsSignerWhitelisted::<T>::insert(pub_key, false);
			Signers::<T>::put(updated_array);

			// Emit the SignerRemoved event
			Self::deposit_event(Event::SignerRemoved { signer: pub_key });
		}

		fn handle_events(events_batch: Vec<UniversalEvent>) {
			for event in events_batch.iter() {
				match event {
					UniversalEvent::MarketUpdated(market_updated) => {
						// Check if the Market already exists
						match T::MarketPallet::get_market(market_updated.id) {
							// If yes, update it
							Some(_) => {
								T::MarketPallet::update_market_internal(ExtendedMarket {
									market: market_updated.market.clone(),
									metadata_url: market_updated.metadata_url.clone(),
								});
							},
							// If not, add a new market
							None => {
								T::MarketPallet::add_market_internal(ExtendedMarket {
									market: market_updated.market.clone(),
									metadata_url: market_updated.metadata_url.clone(),
								});
							},
						}
					},
					UniversalEvent::AssetUpdated(asset_updated) => {
						// Check if the Asset already exists
						match T::AssetPallet::get_asset(asset_updated.id) {
							// If yes, update it
							Some(_) => {
								T::AssetPallet::update_asset_internal(ExtendedAsset {
									asset: asset_updated.asset.clone(),
									metadata_url: asset_updated.metadata_url.clone(),
								});
							},
							// If not, add a new asset
							None => {
								T::AssetPallet::add_asset_internal(ExtendedAsset {
									asset: asset_updated.asset.clone(),
									metadata_url: asset_updated.metadata_url.clone(),
								});
							},
						}
					},
					UniversalEvent::MarketRemoved(market_removed) => {
						// Check if the Market exists
						match T::MarketPallet::get_market(market_removed.id) {
							// If yes, remove it
							Some(_) => {
								T::MarketPallet::remove_market_internal(market_removed.id);
							},
							// If not, emit an error
							None => {
								Self::deposit_event(Event::MarketRemovedError {
									id: market_removed.id,
								});
							},
						};
					},
					UniversalEvent::AssetRemoved(asset_removed) => {
						// Check if the Asset exists
						match T::AssetPallet::get_asset(asset_removed.id) {
							// If yes, remove it
							Some(_) => {
								T::AssetPallet::remove_asset_internal(asset_removed.id);
							},
							// If not, emit an error
							None => {
								Self::deposit_event(Event::AssetRemovedError {
									id: asset_removed.id,
								});
							},
						};
					},
					UniversalEvent::UserDeposit(user_deposit) => {
						T::TradingAccountPallet::deposit_internal(
							user_deposit.trading_account,
							user_deposit.collateral_id,
							user_deposit.amount,
						);
					},
					UniversalEvent::SignerAdded(signer_added) => {
						Self::add_signer_internal(signer_added.signer);
					},
					UniversalEvent::SignerRemoved(signer_removed) => {
						// Check if the signer exists
						match IsSignerWhitelisted::<T>::get(signer_removed.signer) {
							// If yes, remove it
							true => {
								Self::remove_signer_internal(signer_removed.signer);
							},
							// If not, emit an error
							false => {
								Self::deposit_event(Event::SignerRemovedError {
									pub_key: signer_removed.signer,
								});
							},
						};
					},
					UniversalEvent::QuorumSet(quorum_set) => {
						// Check if there are enough signers in the system
						match quorum_set.quorum <= Signers::<T>::get().len() as u8 {
							// If yes, set the new quorum
							true => Self::set_signers_quorum_internal(quorum_set.quorum),
							// If not, emit an error
							false => Self::deposit_event(Event::QuorumSetError {
								quorum: quorum_set.quorum,
							}),
						}
					},
				}
			}
		}

		fn has_quorum(signatures: Vec<SyncSignature>, hash: FieldElement) -> bool {
			// Get the required data
			let quorum = SignersQuorum::<T>::get() as usize;

			// Find the number of valid sigs
			let valid_sigs = signatures
				.iter()
				.filter(|&curr_signature| {
					// Convert the data to felt252
					let pub_key_result = curr_signature.signer_pub_key.try_to_felt();
					let r_value_result = curr_signature.r.try_to_felt();
					let s_value_result = curr_signature.s.try_to_felt();

					match pub_key_result.is_ok() & r_value_result.is_ok() & s_value_result.is_ok() {
						true => {
							let signature_felt252 = Signature {
								r: r_value_result.unwrap(),
								s: s_value_result.unwrap(),
							};

							// Check if the sig is valid
							Self::verify_signature(pub_key_result.unwrap(), hash, signature_felt252)
						},
						false => false,
					}
				})
				.take(quorum)
				.count();

			valid_sigs == quorum
		}

		fn verify_signature(
			public_key: FieldElement,
			hash: FieldElement,
			signature: Signature,
		) -> bool {
			match ecdsa_verify(&public_key, &hash, &signature) {
				Ok(res) => res,
				Err(_) => false,
			}
		}

		fn compute_batch_hash(
			events_batch: &Vec<UniversalEvent>,
		) -> Result<FieldElement, Error<T>> {
			// Convert the array of enums to array of felts
			let mut flattened_array: Vec<FieldElement> = Vec::new();
			flattened_array
				.try_append_universal_event_array(&events_batch)
				.map_err(|_| Error::<T>::ConversionError)?;

			// Compute hash of the array and return
			Ok(pedersen_hash_multiple(&flattened_array))
		}

		fn get_block_and_event_number(event: &UniversalEvent) -> (u64, u32) {
			match event {
				UniversalEvent::MarketUpdated(market_updated) =>
					(market_updated.block_number, market_updated.event_index),
				UniversalEvent::AssetUpdated(user_withdrawal) =>
					(user_withdrawal.block_number, user_withdrawal.event_index),
				UniversalEvent::MarketRemoved(market_removed) =>
					(market_removed.block_number, market_removed.event_index),
				UniversalEvent::AssetRemoved(asset_removed) =>
					(asset_removed.block_number, asset_removed.event_index),
				UniversalEvent::UserDeposit(user_deposit) =>
					(user_deposit.block_number, user_deposit.event_index),
				UniversalEvent::SignerAdded(signer_added) =>
					(signer_added.block_number, signer_added.event_index),
				UniversalEvent::SignerRemoved(signer_removed) =>
					(signer_removed.block_number, signer_removed.event_index),
				UniversalEvent::QuorumSet(quorum_set) =>
					(quorum_set.block_number, quorum_set.event_index),
			}
		}
	}
}
