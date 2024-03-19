#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use frame_support::{
		dispatch::Vec,
		pallet_prelude::{OptionQuery, *},
	};
	use frame_system::pallet_prelude::*;
	use pallet_support::{
		ecdsa_verify,
		helpers::compute_hash_on_elements,
		traits::{
			AssetInterface, FeltSerializedArrayExt, FieldElementExt, MarketInterface,
			PricesInterface, TradingAccountInterface, TradingFeesInterface, U256Ext,
		},
		types::{
			ABRSettingsType, BaseFee, ExtendedAsset, ExtendedMarket, FeeSettingsType, OrderSide,
			Setting, SettingsType, Side, SyncSignature, UniversalEvent,
		},
		FieldElement, Signature,
	};
	use primitive_types::U256;
	use sp_arithmetic::fixed_point::FixedI128;

	#[cfg(not(feature = "dev"))]
	pub const IS_DEV_ENABLED: bool = false;

	#[cfg(feature = "dev")]
	pub const IS_DEV_ENABLED: bool = true;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type TradingAccountPallet: TradingAccountInterface;
		type AssetPallet: AssetInterface;
		type MarketPallet: MarketInterface;
		type TradingFeesPallet: TradingFeesInterface;
		type PricesPallet: PricesInterface;
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
	#[pallet::getter(fn get_temp_fees)]
	// k1 - market_id/asset_id, k2 - FeeSettingType, v - FixedI28[]
	pub(super) type TempFeesMap<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		u128,
		Twox64Concat,
		FeeSettingsType,
		Vec<FixedI128>,
		OptionQuery,
	>;

	// Note: This storage map is used for market_ids as well;
	// Keeping the name unchanged for the upgrade support
	#[pallet::storage]
	#[pallet::getter(fn get_temp_assets)]
	// k1 - market_id/asset_id, v - bool
	pub(super) type TempAssetsMap<T: Config> = StorageMap<_, Blake2_128Concat, u128, bool>;

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
		/// An invalid request to add a duplicate signer
		SignerAddedError { pub_key: U256 },
		/// An invalid request to remove non-existent signer
		SignerRemovedError { pub_key: U256 },
		/// An invalid request to remove signer; leads to insufficient signers
		SignerRemovedQuorumError { quorum: u8 },
		/// An invalid request to set a signer
		QuorumSetError { quorum: u8 },
		/// An invalid request for user deposit
		UserDepositError { collateral_id: u128 },
		/// An invalid key in settings
		SettingsKeyError { key: u128 },
		/// Insufficient data for setting fees
		InsufficientFeeData { id: u128 },
		/// Fee data length mismatch
		FeeDataLengthMismatch { id: u128 },
		/// Token parsing error
		TokenParsingError { key: U256 },
		/// An invalid request to add an asset
		AddAssetError { id: u128 },
		/// An invalid request to update an asset
		UpdateAssetError { id: u128 },
		/// An invalid requet to add a market
		AddMarketError { id: u128 },
		/// An invalid request to update a market
		UpdateMarketError { id: u128 },
		/// An unknown asset/market id passed
		UnknownIdForFees { id: u128 },
		/// An invalid request to set max abr
		InvalidMarket { id: u128 },
		/// A max abr request with empty array
		EmptyValuesError { id: u128 },
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
		/// Invalid Call to dev mode only function
		DevOnlyCall,
	}

	// Constants
	const DELIMITER: u8 = 95;
	const FEE_SETTINGS: u128 = 70;
	const GENERAL_SETTINGS: u128 = 71;
	const MAKER_ENCODING: u128 = 77;
	const TAKER_ENCODING: u128 = 84;
	const OPEN_ENCODING: u128 = 79;
	const CLOSE_ENCODING: u128 = 67;
	const OMISSION_ENCODING: u128 = 45;
	const ABR_ENCODING: u128 = 65;

	// Pallet callable functions
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// TODO(merkle-groot): To be removed in production
		#[pallet::weight(0)]
		pub fn add_signer(origin: OriginFor<T>, pub_key: U256) -> DispatchResult {
			if !IS_DEV_ENABLED {
				return Err(Error::<T>::DevOnlyCall.into());
			}
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
			if !IS_DEV_ENABLED {
				return Err(Error::<T>::DevOnlyCall.into());
			}
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
			if !IS_DEV_ENABLED {
				return Err(Error::<T>::DevOnlyCall.into());
			}
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

		fn resolve_setting(
			settings_type: u128,
			param1: u128,
			param2: u128,
			param3: u128,
		) -> Option<SettingsType> {
			match settings_type {
				FEE_SETTINGS => match param2 {
					MAKER_ENCODING => match param3 {
						OPEN_ENCODING =>
							return Some(SettingsType::FeeSettings(FeeSettingsType::MakerOpen)),
						CLOSE_ENCODING =>
							return Some(SettingsType::FeeSettings(FeeSettingsType::MakerClose)),
						OMISSION_ENCODING =>
							return Some(SettingsType::FeeSettings(FeeSettingsType::MakerVols)),
						_ => {
							Self::deposit_event(Event::SettingsKeyError { key: param3 });
							return None;
						},
					},
					TAKER_ENCODING => match param3 {
						OPEN_ENCODING =>
							return Some(SettingsType::FeeSettings(FeeSettingsType::TakerOpen)),
						CLOSE_ENCODING =>
							return Some(SettingsType::FeeSettings(FeeSettingsType::TakerClose)),
						OMISSION_ENCODING =>
							return Some(SettingsType::FeeSettings(FeeSettingsType::TakerVols)),
						_ => {
							Self::deposit_event(Event::SettingsKeyError { key: param3 });
							return None;
						},
					},
					_ => {
						Self::deposit_event(Event::SettingsKeyError { key: param2 });
						return None;
					},
				},
				ABR_ENCODING => match param1 {
					OMISSION_ENCODING =>
						return Some(SettingsType::ABRSettings(ABRSettingsType::MaxDefault)),
					_ => return Some(SettingsType::ABRSettings(ABRSettingsType::MaxPerMarket)),
				},
				GENERAL_SETTINGS => {
					Self::deposit_event(Event::SettingsKeyError { key: settings_type });
					return None;
				},
				_ => {
					Self::deposit_event(Event::SettingsKeyError { key: settings_type });
					return None;
				},
			}
		}

		fn get_ascii_value(vec: Vec<u8>) -> u128 {
			let mut result: u128 = 0;
			for num in vec {
				result = (result * 256) + num as u128;
			}

			return result;
		}

		fn u256_to_tokens(input: U256) -> Option<(u128, u128, u128, u128)> {
			// Create a vector to store the bytes
			let mut bytes: Vec<u8> =
				(0..32).map(|i| input.byte(i)).take_while(|&byte| byte != 0).collect();

			// Reverse the vec
			bytes.reverse();

			// Split the vec according to the delimiter
			let pieces: Vec<u128> = bytes
				.split(|&e| e == DELIMITER)
				.filter(|v| !v.is_empty())
				.map(|v| Self::get_ascii_value(v.to_vec()))
				.collect();

			if pieces.len() != 4 {
				return None;
			}

			return Some((pieces[0], pieces[1], pieces[2], pieces[3]));
		}

		fn create_base_fee_vec(volumes: &Vec<FixedI128>, fees: &Vec<FixedI128>) -> Vec<BaseFee> {
			volumes
				.into_iter()
				.zip(fees.into_iter())
				.map(|(volume, fee)| BaseFee { volume: *volume, fee: *fee })
				.collect()
		}

		fn set_fees_internal(
			id: u128,
			side: Side,
			order_side: OrderSide,
			volumes: &Vec<FixedI128>,
			fees: &Vec<FixedI128>,
		) -> Result<(), ()> {
			match T::TradingFeesPallet::update_base_fees_internal(
				id,
				side,
				order_side,
				Self::create_base_fee_vec(volumes, fees),
			) {
				Ok(_) => Ok(()),
				Err(_) => {
					// Remove partially set fees in Trading Fees pallet
					T::TradingFeesPallet::remove_base_fees_internal(id);

					// Emit Unknown data event
					Self::deposit_event(Event::UnknownIdForFees { id });
					Self::remove_settings_from_maps(id);
					Err(())
				},
			}
		}

		fn set_trading_fees() {
			let id_list: Vec<u128> = TempAssetsMap::<T>::iter().map(|(key, _)| key).collect();
			for &id in id_list.iter() {
				// get maker and taker volumes
				let maker_volumes_query = TempFeesMap::<T>::get(id, FeeSettingsType::MakerVols);
				let taker_volumes_query = TempFeesMap::<T>::get(id, FeeSettingsType::TakerVols);
				// Get the fee vectors of Maker
				let maker_open_fees_query = TempFeesMap::<T>::get(id, FeeSettingsType::MakerOpen);
				let maker_close_fees_query = TempFeesMap::<T>::get(id, FeeSettingsType::MakerClose);

				// Get the fee vectors of Taker
				let taker_open_fees_query = TempFeesMap::<T>::get(id, FeeSettingsType::TakerOpen);
				let taker_close_fees_query = TempFeesMap::<T>::get(id, FeeSettingsType::TakerClose);

				// Check if all the required data is present for this asset
				if !(maker_volumes_query.is_some() &&
					taker_volumes_query.is_some() &&
					maker_open_fees_query.is_some() &&
					maker_close_fees_query.is_some() &&
					taker_open_fees_query.is_some() &&
					taker_close_fees_query.is_some())
				{
					// Emit Insufficient data event
					Self::deposit_event(Event::InsufficientFeeData { id });
					Self::remove_settings_from_maps(id);
					continue;
				}

				// Unwrap maker data
				let maker_volumes = maker_volumes_query.unwrap();
				let maker_open_fees = maker_open_fees_query.unwrap();
				let maker_close_fees = maker_close_fees_query.unwrap();

				// Unwrap taker data
				let taker_volumes = taker_volumes_query.unwrap();
				let taker_open_fees = taker_open_fees_query.unwrap();
				let taker_close_fees = taker_close_fees_query.unwrap();

				// Maker data must be of the same length
				if !(maker_volumes.len() == maker_open_fees.len() &&
					maker_open_fees.len() == maker_close_fees.len()) ||
					!(taker_volumes.len() == taker_open_fees.len() &&
						taker_open_fees.len() == taker_close_fees.len())
				{
					// Emit Insufficient data event
					Self::deposit_event(Event::FeeDataLengthMismatch { id });
					Self::remove_settings_from_maps(id);
					continue;
				}

				if let Err(_) = Self::set_fees_internal(
					id,
					Side::Buy,
					OrderSide::Maker,
					&maker_volumes,
					&maker_open_fees,
				) {
					continue;
				}

				if let Err(_) = Self::set_fees_internal(
					id,
					Side::Sell,
					OrderSide::Maker,
					&maker_volumes,
					&maker_close_fees,
				) {
					continue;
				}

				if let Err(_) = Self::set_fees_internal(
					id,
					Side::Buy,
					OrderSide::Taker,
					&taker_volumes,
					&taker_open_fees,
				) {
					continue;
				}

				if let Err(_) = Self::set_fees_internal(
					id,
					Side::Sell,
					OrderSide::Taker,
					&taker_volumes,
					&taker_close_fees,
				) {
					continue;
				}

				Self::remove_settings_from_maps(id);
			}
		}

		fn add_settings_to_maps(
			id: u128,
			fee_settings_type: FeeSettingsType,
			values: Vec<FixedI128>,
		) {
			// Add the asset to the map
			TempAssetsMap::<T>::insert(id, true);

			// Insert maker volume vector to the map
			TempFeesMap::<T>::insert(id, fee_settings_type, values);
		}

		fn remove_settings_from_maps(id: u128) {
			// Add the asset to the map
			TempAssetsMap::<T>::remove(id);

			// Insert maker volume vector to the map
			TempFeesMap::<T>::remove(id, FeeSettingsType::MakerVols);
			TempFeesMap::<T>::remove(id, FeeSettingsType::TakerVols);
			TempFeesMap::<T>::remove(id, FeeSettingsType::MakerOpen);
			TempFeesMap::<T>::remove(id, FeeSettingsType::MakerClose);
			TempFeesMap::<T>::remove(id, FeeSettingsType::TakerOpen);
			TempFeesMap::<T>::remove(id, FeeSettingsType::TakerClose);
		}

		fn handle_settings(settings: &BoundedVec<Setting, ConstU32<256>>) {
			for setting in settings {
				// Parse the key of the current setting
				let parsing_result = Self::u256_to_tokens(setting.key);

				if parsing_result == None {
					// exit from the loop
					Self::deposit_event(Event::TokenParsingError { key: setting.key });
					continue;
				}

				// Get the constituents of the key
				let (setting_encoding, param1, param2, param3) = parsing_result.unwrap();

				// Resolve the type of setting
				let setting_type = Self::resolve_setting(setting_encoding, param1, param2, param3);
				if setting_type == None {
					continue;
				}

				// Handle the setting
				match setting_type.unwrap() {
					SettingsType::FeeSettings(fee_settings_type) => match fee_settings_type {
						FeeSettingsType::MakerVols => {
							Self::add_settings_to_maps(
								param1,
								FeeSettingsType::MakerVols,
								setting.values.to_vec(),
							);
						},
						FeeSettingsType::TakerVols => {
							Self::add_settings_to_maps(
								param1,
								FeeSettingsType::TakerVols,
								setting.values.to_vec(),
							);
						},
						FeeSettingsType::MakerOpen => {
							Self::add_settings_to_maps(
								param1,
								FeeSettingsType::MakerOpen,
								setting.values.to_vec(),
							);
						},
						FeeSettingsType::MakerClose => {
							Self::add_settings_to_maps(
								param1,
								FeeSettingsType::MakerClose,
								setting.values.to_vec(),
							);
						},
						FeeSettingsType::TakerOpen => {
							Self::add_settings_to_maps(
								param1,
								FeeSettingsType::TakerOpen,
								setting.values.to_vec(),
							);
						},
						FeeSettingsType::TakerClose => {
							Self::add_settings_to_maps(
								param1,
								FeeSettingsType::TakerClose,
								setting.values.to_vec(),
							);
						},
					},
					SettingsType::ABRSettings(abr_settings_type) =>
						Self::set_abr_max(abr_settings_type, param1, setting.values.to_vec()),
					SettingsType::GeneralSettings => {},
				}
			}

			// Set the trading Fees and remove the temporary storage items
			Self::set_trading_fees();
		}

		fn set_abr_max(
			abr_settings_type: ABRSettingsType,
			market_id: u128,
			values: Vec<FixedI128>,
		) {
			// Check if values is not empty
			if values.is_empty() {
				// Handle the case where values is empty
				Self::deposit_event(Event::EmptyValuesError { id: market_id });
				return;
			}

			match abr_settings_type {
				ABRSettingsType::MaxDefault => {
					T::PricesPallet::set_default_max_abr_internal(values[0]);
				},
				ABRSettingsType::MaxPerMarket => {
					match T::PricesPallet::set_max_abr_internal(market_id, values[0]) {
						Ok(()) => (),
						Err(_) => {
							Self::deposit_event(Event::InvalidMarket { id: market_id });
						},
					}
				},
			}
		}

		fn handle_events(events_batch: Vec<UniversalEvent>) {
			for event in events_batch.iter() {
				match event {
					UniversalEvent::MarketUpdated(market_updated) => {
						// Check if the Market already exists
						match T::MarketPallet::get_market(market_updated.id) {
							// If yes, update it
							Some(_) => {
								match T::MarketPallet::update_market_internal(ExtendedMarket {
									market: market_updated.market.clone(),
									metadata_url: market_updated.metadata_url.clone(),
								}) {
									Ok(_) => (),
									Err(_) => Self::deposit_event(Event::UpdateMarketError {
										id: market_updated.id,
									}),
								}
							},
							// If not, add a new market
							None => {
								match T::MarketPallet::add_market_internal(ExtendedMarket {
									market: market_updated.market.clone(),
									metadata_url: market_updated.metadata_url.clone(),
								}) {
									Ok(_) => (),
									Err(_) => Self::deposit_event(Event::AddMarketError {
										id: market_updated.id,
									}),
								}
							},
						}
					},
					UniversalEvent::AssetUpdated(asset_updated) => {
						// Check if the Asset already exists
						match T::AssetPallet::get_asset(asset_updated.id) {
							// If yes, update it
							Some(_) => {
								match T::AssetPallet::update_asset_internal(ExtendedAsset {
									asset: asset_updated.asset.clone(),
									asset_addresses: asset_updated.asset_addresses.clone(),
									metadata_url: asset_updated.metadata_url.clone(),
								}) {
									Ok(_) => (),
									Err(_) => Self::deposit_event(Event::UpdateAssetError {
										id: asset_updated.id,
									}),
								}
							},
							// If not, add a new asset
							None => {
								match T::AssetPallet::add_asset_internal(ExtendedAsset {
									asset: asset_updated.asset.clone(),
									asset_addresses: asset_updated.asset_addresses.clone(),
									metadata_url: asset_updated.metadata_url.clone(),
								}) {
									Ok(_) => (),
									Err(_) => Self::deposit_event(Event::AddAssetError {
										id: asset_updated.id,
									}),
								}
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
						// Check if the Asset exists and is valid
						if let Some(asset) = T::AssetPallet::get_asset(user_deposit.collateral_id) {
							if asset.is_collateral {
								T::TradingAccountPallet::deposit_internal(
									user_deposit.trading_account,
									user_deposit.collateral_id,
									user_deposit.amount,
								);
							} else {
								Self::deposit_event(Event::UserDepositError {
									collateral_id: user_deposit.collateral_id,
								});
							}
						} else {
							Self::deposit_event(Event::UserDepositError {
								collateral_id: user_deposit.collateral_id,
							});
						}
					},
					UniversalEvent::SignerAdded(signer_added) => {
						// Check if the signer exists
						match IsSignerWhitelisted::<T>::get(signer_added.signer) {
							true => {
								// If yes, emit an error
								// Duplicate signer
								Self::deposit_event(Event::SignerAddedError {
									pub_key: signer_added.signer,
								});
							},
							// If not, whitelist the key
							false => {
								Self::add_signer_internal(signer_added.signer);
							},
						};
					},
					UniversalEvent::SignerRemoved(signer_removed) => {
						// Check if the signer exists
						match IsSignerWhitelisted::<T>::get(signer_removed.signer) {
							// If yes, check if removing the signer leaves us with sufficient
							// signers
							true => {
								let signer_quorum = SignersQuorum::<T>::get();
								match signer_quorum < Signers::<T>::get().len() as u8 {
									// If yes, remove the signer
									true => Self::remove_signer_internal(signer_removed.signer),
									// If not, emit an error
									false => Self::deposit_event(Event::SignerRemovedQuorumError {
										quorum: signer_quorum,
									}),
								};
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
						};
					},
					UniversalEvent::SettingsAdded(settings_added) => {
						Self::handle_settings(&settings_added.settings);
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
			Ok(compute_hash_on_elements(&flattened_array))
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
				UniversalEvent::SettingsAdded(settings_added) =>
					(settings_added.block_number, settings_added.event_index),
			}
		}
	}
}
