use frame_support::dispatch::Vec;
use pallet_support::{
	ecdsa_sign,
	helpers::compute_hash_on_elements,
	traits::{FeltSerializedArrayExt, FieldElementExt},
	types::{
		Asset, AssetAddress, AssetRemoved, AssetUpdated, BaseFee, BaseFeeAggregate, Market,
		MarketRemoved, MarketUpdated, QuorumSet, Setting, SettingsAdded, SignerAdded,
		SignerRemoved, SyncSignature, TradingAccountMinimal, UniversalEvent, UserDeposit,
	},
	FieldElement,
};
use primitive_types::U256;
use sp_arithmetic::fixed_point::FixedI128;
use sp_runtime::{bounded_vec, traits::ConstU32, BoundedVec};

pub trait MarketUpdatedTrait {
	fn new(
		event_index: u32,
		id: u128,
		market: Market,
		metadata_url: BoundedVec<u8, ConstU32<256>>,
		block_number: u64,
	) -> MarketUpdated;
}

pub trait AssetUpdatedTrait {
	fn new(
		event_index: u32,
		id: u128,
		asset: Asset,
		asset_address: BoundedVec<AssetAddress, ConstU32<256>>,
		metadata_url: BoundedVec<u8, ConstU32<256>>,
		block_number: u64,
	) -> AssetUpdated;
}

pub trait MarketRemovedTrait {
	fn new(event_index: u32, id: u128, block_number: u64) -> MarketRemoved;
}

pub trait AssetRemovedTrait {
	fn new(event_index: u32, id: u128, block_number: u64) -> AssetRemoved;
}

pub trait UserDepositTrait {
	fn new(
		event_index: u32,
		trading_account: TradingAccountMinimal,
		collateral_id: u128,
		nonce: U256,
		amount: FixedI128,
		block_number: u64,
	) -> UserDeposit;
}

pub trait SignerAddedTrait {
	fn new(event_index: u32, signer: U256, block_number: u64) -> SignerAdded;
}

pub trait SignerRemovedTrait {
	fn new(event_index: u32, signer: U256, block_number: u64) -> SignerRemoved;
}

pub trait QuorumSetTrait {
	fn new(event_index: u32, quorum: u8, block_number: u64) -> QuorumSet;
}

pub trait SettingsAddedTrait {
	fn new(
		event_index: u32,
		settings: BoundedVec<Setting, ConstU32<256>>,
		block_number: u64,
	) -> SettingsAdded;

	fn get_usdc_fees_settings() -> SettingsAdded;
	fn get_usdt_fees_settings() -> SettingsAdded;
	fn get_btc_usdc_fees_settings() -> SettingsAdded;
	fn get_frax_fees_settings() -> SettingsAdded;
	fn get_max_default_settings() -> SettingsAdded;
	fn get_max_btc_usdc_settings() -> SettingsAdded;
	fn get_max_eth_usdc_settings() -> SettingsAdded;
	// fee share
	fn get_usdc_fee_shares_settings() -> SettingsAdded;
}

impl MarketUpdatedTrait for MarketUpdated {
	fn new(
		event_index: u32,
		id: u128,
		market: Market,
		metadata_url: BoundedVec<u8, ConstU32<256>>,
		block_number: u64,
	) -> MarketUpdated {
		MarketUpdated { event_index, id, market, metadata_url, block_number }
	}
}

impl AssetUpdatedTrait for AssetUpdated {
	fn new(
		event_index: u32,
		id: u128,
		asset: Asset,
		asset_addresses: BoundedVec<AssetAddress, ConstU32<256>>,
		metadata_url: BoundedVec<u8, ConstU32<256>>,
		block_number: u64,
	) -> AssetUpdated {
		AssetUpdated { event_index, id, asset, asset_addresses, metadata_url, block_number }
	}
}

impl MarketRemovedTrait for MarketRemoved {
	fn new(event_index: u32, id: u128, block_number: u64) -> MarketRemoved {
		MarketRemoved { event_index, id, block_number }
	}
}

impl AssetRemovedTrait for AssetRemoved {
	fn new(event_index: u32, id: u128, block_number: u64) -> AssetRemoved {
		AssetRemoved { event_index, id, block_number }
	}
}

impl SignerAddedTrait for SignerAdded {
	fn new(event_index: u32, signer: U256, block_number: u64) -> SignerAdded {
		SignerAdded { event_index, signer, block_number }
	}
}

impl SignerRemovedTrait for SignerRemoved {
	fn new(event_index: u32, signer: U256, block_number: u64) -> SignerRemoved {
		SignerRemoved { event_index, signer, block_number }
	}
}

impl SettingsAddedTrait for SettingsAdded {
	fn new(
		event_index: u32,
		settings: BoundedVec<Setting, ConstU32<256>>,
		block_number: u64,
	) -> SettingsAdded {
		SettingsAdded { event_index, settings, block_number }
	}

	fn get_usdt_fees_settings() -> SettingsAdded {
		let settings = bounded_vec![
			Setting {
				// F_USDT_M_-
				key: U256::from(332324242820923030069037_i128),
				values: bounded_vec![
					FixedI128::from_u32(0),
					FixedI128::from_u32(1000000),
					FixedI128::from_u32(5000000),
				]
			},
			Setting {
				// F_USDT_T_-
				key: U256::from(332324242820923030527789_i128),
				values: bounded_vec![
					FixedI128::from_u32(0),
					FixedI128::from_u32(1000000),
					FixedI128::from_u32(5000000),
					FixedI128::from_u32(10000000),
					FixedI128::from_u32(50000000),
				]
			},
			Setting {
				// F_USDT_M_O
				key: U256::from(332324242820923030069071_i128),
				values: bounded_vec![
					FixedI128::from_float(0.020),
					FixedI128::from_float(0.015),
					FixedI128::from_float(0.0),
				]
			},
			Setting {
				// F_USDT_M_C
				key: U256::from(332324242820923030069059_i128),
				values: bounded_vec![
					FixedI128::from_float(0.020),
					FixedI128::from_float(0.015),
					FixedI128::from_float(0.0),
				]
			},
			Setting {
				// F_USDT_T_O
				key: U256::from(332324242820923030527823_i128),
				values: bounded_vec![
					FixedI128::from_float(0.050),
					FixedI128::from_float(0.040),
					FixedI128::from_float(0.035),
					FixedI128::from_float(0.030),
					FixedI128::from_float(0.025),
				]
			},
			Setting {
				// F_USDT_T_C
				key: U256::from(332324242820923030527811_i128),
				values: bounded_vec![
					FixedI128::from_float(0.050),
					FixedI128::from_float(0.040),
					FixedI128::from_float(0.035),
					FixedI128::from_float(0.030),
					FixedI128::from_float(0.025),
				]
			}
		];

		SettingsAdded { event_index: 1, settings, block_number: 1337 }
	}

	fn get_frax_fees_settings() -> SettingsAdded {
		let settings = bounded_vec![
			Setting {
				// F_FRAX_M_-
				key: U256::from(332323161672256129425197_i128),
				values: bounded_vec![
					FixedI128::from_u32(0),
					FixedI128::from_u32(1000000),
					FixedI128::from_u32(5000000),
					FixedI128::from_u32(10000000),
					FixedI128::from_u32(50000000),
				]
			},
			Setting {
				// F_FRAX_T_-
				key: U256::from(332323161672256129883949_i128),
				values: bounded_vec![
					FixedI128::from_u32(0),
					FixedI128::from_u32(1000000),
					FixedI128::from_u32(5000000),
					FixedI128::from_u32(10000000),
					FixedI128::from_u32(50000000),
					FixedI128::from_u32(200000000),
				]
			},
			Setting {
				// F_FRAX_M_O
				key: U256::from(332323161672256129425231_i128),
				values: bounded_vec![
					FixedI128::from_float(0.020),
					FixedI128::from_float(0.015),
					FixedI128::from_float(0.010),
					FixedI128::from_float(0.005),
					FixedI128::from_float(0.0),
				]
			},
			Setting {
				// F_FRAX_M_C
				key: U256::from(332323161672256129425219_i128),
				values: bounded_vec![
					FixedI128::from_float(0.020),
					FixedI128::from_float(0.015),
					FixedI128::from_float(0.010),
					FixedI128::from_float(0.005),
					FixedI128::from_float(0.0),
				]
			},
			Setting {
				// F_FRAX_T_O
				key: U256::from(332323161672256129883983_i128),
				values: bounded_vec![
					FixedI128::from_float(0.050),
					FixedI128::from_float(0.040),
					FixedI128::from_float(0.035),
					FixedI128::from_float(0.030),
					FixedI128::from_float(0.025),
					FixedI128::from_float(0.020),
				]
			},
			Setting {
				// F_FRAX_T_C
				key: U256::from(332323161672256129883971_i128),
				values: bounded_vec![
					FixedI128::from_float(0.050),
					FixedI128::from_float(0.040),
					FixedI128::from_float(0.035),
					FixedI128::from_float(0.030),
					FixedI128::from_float(0.025),
					FixedI128::from_float(0.020),
				]
			}
		];

		SettingsAdded { event_index: 1, settings, block_number: 1337 }
	}

	fn get_usdc_fees_settings() -> SettingsAdded {
		let settings = bounded_vec![
			Setting {
				// F_USDC_M_-
				key: U256::from(332324242820850015625005_i128),
				values: bounded_vec![
					FixedI128::from_u32(0),
					FixedI128::from_u32(1000000),
					FixedI128::from_u32(5000000),
					FixedI128::from_u32(10000000),
					FixedI128::from_u32(50000000),
				]
			},
			Setting {
				// F_USDC_T_-
				key: U256::from(332324242820850016083757_i128),
				values: bounded_vec![
					FixedI128::from_u32(0),
					FixedI128::from_u32(1000000),
					FixedI128::from_u32(5000000),
					FixedI128::from_u32(10000000),
					FixedI128::from_u32(50000000),
					FixedI128::from_u32(200000000),
				]
			},
			Setting {
				// F_USDC_M_O
				key: U256::from(332324242820850015625039_i128),
				values: bounded_vec![
					FixedI128::from_float(0.020),
					FixedI128::from_float(0.015),
					FixedI128::from_float(0.010),
					FixedI128::from_float(0.005),
					FixedI128::from_float(0.0),
				]
			},
			Setting {
				// F_USDC_M_C
				key: U256::from(332324242820850015625027_i128),
				values: bounded_vec![
					FixedI128::from_float(0.020),
					FixedI128::from_float(0.015),
					FixedI128::from_float(0.010),
					FixedI128::from_float(0.005),
					FixedI128::from_float(0.0),
				]
			},
			Setting {
				// F_USDC_T_O
				key: U256::from(332324242820850016083791_i128),
				values: bounded_vec![
					FixedI128::from_float(0.050),
					FixedI128::from_float(0.040),
					FixedI128::from_float(0.035),
					FixedI128::from_float(0.030),
					FixedI128::from_float(0.025),
					FixedI128::from_float(0.020),
				]
			},
			Setting {
				// F_USDC_T_C
				key: U256::from(332324242820850016083779_i128),
				values: bounded_vec![
					FixedI128::from_float(0.050),
					FixedI128::from_float(0.040),
					FixedI128::from_float(0.035),
					FixedI128::from_float(0.030),
					FixedI128::from_float(0.025),
					FixedI128::from_float(0.020),
				]
			}
		];

		SettingsAdded { event_index: 1, settings, block_number: 1337 }
	}

	fn get_btc_usdc_fees_settings() -> SettingsAdded {
		let settings = bounded_vec![
			Setting {
				// F_BTCUSDC_M_-
				key: U256::from(5575452638956490725563642502957_i128),
				values: bounded_vec![
					FixedI128::from_u32(0),
					FixedI128::from_u32(10000),
					FixedI128::from_u32(1000000),
				]
			},
			Setting {
				// F_BTCUSDC_T_-
				key: U256::from(5575452638956490725563642961709_i128),
				values: bounded_vec![
					FixedI128::from_u32(0),
					FixedI128::from_u32(10000),
					FixedI128::from_u32(1000000),
					FixedI128::from_u32(5000000)
				]
			},
			Setting {
				// F_BTCUSDC_M_O
				key: U256::from(5575452638956490725563642502991_i128),
				values: bounded_vec![
					FixedI128::from_float(0.002),
					FixedI128::from_float(0.001),
					FixedI128::from_float(0.0),
				]
			},
			Setting {
				// F_BTCUSDC_M_C
				key: U256::from(5575452638956490725563642502979_i128),
				values: bounded_vec![
					FixedI128::from_float(0.002),
					FixedI128::from_float(0.001),
					FixedI128::from_float(0.0),
				]
			},
			Setting {
				// F_BTCUSDC_T_O
				key: U256::from(5575452638956490725563642961743_i128),
				values: bounded_vec![
					FixedI128::from_float(0.005),
					FixedI128::from_float(0.0045),
					FixedI128::from_float(0.004),
					FixedI128::from_float(0.002),
				]
			},
			Setting {
				// F_BTCUSDC_T_C
				key: U256::from(5575452638956490725563642961731_i128),
				values: bounded_vec![
					FixedI128::from_float(0.005),
					FixedI128::from_float(0.0045),
					FixedI128::from_float(0.004),
					FixedI128::from_float(0.002),
				]
			}
		];

		SettingsAdded { event_index: 1, settings, block_number: 1337 }
	}

	fn get_max_default_settings() -> SettingsAdded {
		let settings = bounded_vec![Setting {
			// A_-_-_-
			key: U256::from(18400521961168685_i128),
			values: bounded_vec![FixedI128::from_float(0.0012)]
		}];

		SettingsAdded { event_index: 1, settings, block_number: 1337 }
	}

	fn get_max_btc_usdc_settings() -> SettingsAdded {
		let settings = bounded_vec![Setting {
			// A_BTCUSDC_-_-
			key: U256::from(5179311826385169037595920654125_i128),
			values: bounded_vec![FixedI128::from_float(0.01)]
		}];

		SettingsAdded { event_index: 1, settings, block_number: 1337 }
	}

	fn get_max_eth_usdc_settings() -> SettingsAdded {
		let settings = bounded_vec![Setting {
			// A_ETHUSDC_-_-
			key: U256::from(5179315453254861601851992530733_i128),
			values: bounded_vec![FixedI128::from_float(0.05)]
		}];

		SettingsAdded { event_index: 1, settings, block_number: 1337 }
	}

	fn get_usdc_fee_shares_settings() -> SettingsAdded {
		let settings = bounded_vec![
			Setting {
				// R_USDC_V_-
				key: U256::from(388992640615285758779181_i128),
				values: bounded_vec![FixedI128::from_float(0.05)]
			},
			Setting {
				// R_USDC_F_-
				key: U256::from(388992640615285757730605_i128),
				values: bounded_vec![FixedI128::from_float(0.05)]
			}
		];
	}
}

impl UserDepositTrait for UserDeposit {
	fn new(
		event_index: u32,
		trading_account: TradingAccountMinimal,
		collateral_id: u128,
		nonce: U256,
		amount: FixedI128,
		block_number: u64,
	) -> UserDeposit {
		UserDeposit { event_index, trading_account, collateral_id, nonce, amount, block_number }
	}
}

impl QuorumSetTrait for QuorumSet {
	fn new(event_index: u32, quorum: u8, block_number: u64) -> QuorumSet {
		QuorumSet { event_index, quorum, block_number }
	}
}

pub trait UniversalEventArray {
	fn new() -> Vec<UniversalEvent>;
	fn add_market_updated_event(&mut self, market_updated_event: MarketUpdated);
	fn add_asset_updated_event(&mut self, asset_updated_event: AssetUpdated);
	fn add_market_removed_event(&mut self, market_removed_event: MarketRemoved);
	fn add_asset_removed_event(&mut self, asset_removed_event: AssetRemoved);
	fn add_user_deposit_event(&mut self, user_deposit_event: UserDeposit);
	fn add_signer_added_event(&mut self, signer_added_event: SignerAdded);
	fn add_signer_removed_event(&mut self, signer_removed_event: SignerRemoved);
	fn add_quorum_set_event(&mut self, quorum_set_event: QuorumSet);
	fn add_settings_event(&mut self, settings_added_event: SettingsAdded);
	fn compute_hash(&self) -> FieldElement;
}

pub trait SyncSignatureArray {
	fn new() -> Vec<SyncSignature>;
	fn add_new_signature(
		&mut self,
		message_hash: FieldElement,
		public_key: U256,
		private_key: FieldElement,
	);
}

impl SyncSignatureArray for Vec<SyncSignature> {
	fn new() -> Vec<SyncSignature> {
		Vec::<SyncSignature>::new()
	}

	fn add_new_signature(
		&mut self,
		message_hash: FieldElement,
		public_key: U256,
		private_key: FieldElement,
	) {
		let signature = ecdsa_sign(&private_key, &message_hash).unwrap();
		self.push(SyncSignature {
			signer_pub_key: public_key,
			r: signature.r.to_u256(),
			s: signature.s.to_u256(),
		});
	}
}

impl UniversalEventArray for Vec<UniversalEvent> {
	fn new() -> Vec<UniversalEvent> {
		Vec::<UniversalEvent>::new()
	}

	fn add_market_updated_event(&mut self, market_updated_event: MarketUpdated) {
		self.push(UniversalEvent::MarketUpdated(market_updated_event));
	}

	fn add_asset_updated_event(&mut self, asset_updated_event: AssetUpdated) {
		self.push(UniversalEvent::AssetUpdated(asset_updated_event));
	}

	fn add_market_removed_event(&mut self, market_removed_event: MarketRemoved) {
		self.push(UniversalEvent::MarketRemoved(market_removed_event));
	}

	fn add_asset_removed_event(&mut self, asset_removed_event: AssetRemoved) {
		self.push(UniversalEvent::AssetRemoved(asset_removed_event));
	}

	fn add_user_deposit_event(&mut self, user_deposit_event: UserDeposit) {
		self.push(UniversalEvent::UserDeposit(user_deposit_event));
	}

	fn add_signer_added_event(&mut self, signer_added_event: SignerAdded) {
		self.push(UniversalEvent::SignerAdded(signer_added_event));
	}

	fn add_signer_removed_event(&mut self, signer_removed_event: SignerRemoved) {
		self.push(UniversalEvent::SignerRemoved(signer_removed_event));
	}

	fn add_quorum_set_event(&mut self, quorum_set_event: QuorumSet) {
		self.push(UniversalEvent::QuorumSet(quorum_set_event));
	}

	fn add_settings_event(&mut self, settings_added_event: SettingsAdded) {
		self.push(UniversalEvent::SettingsAdded(settings_added_event));
	}

	fn compute_hash(&self) -> FieldElement {
		let mut flattened_array: Vec<FieldElement> = Vec::new();
		flattened_array.try_append_universal_event_array(&self).unwrap();

		// Compute hash of the array and return
		compute_hash_on_elements(&flattened_array)
	}
}

pub fn get_usdc_aggregate_fees() -> BaseFeeAggregate {
	BaseFeeAggregate {
		maker_buy: get_usdc_maker_open_fees(),
		maker_sell: get_usdc_maker_close_fees(),
		taker_buy: get_usdc_taker_open_fees(),
		taker_sell: get_usdc_taker_close_fees(),
	}
}

pub fn get_usdt_aggregate_fees() -> BaseFeeAggregate {
	BaseFeeAggregate {
		maker_buy: get_usdt_maker_open_fees(),
		maker_sell: get_usdt_maker_close_fees(),
		taker_buy: get_usdt_taker_open_fees(),
		taker_sell: get_usdt_taker_close_fees(),
	}
}

pub fn get_btc_usdc_aggregate_fees() -> BaseFeeAggregate {
	BaseFeeAggregate {
		maker_buy: get_btc_usdc_maker_open_fees(),
		maker_sell: get_btc_usdc_maker_close_fees(),
		taker_buy: get_btc_usdc_taker_open_fees(),
		taker_sell: get_btc_usdc_taker_close_fees(),
	}
}

fn get_usdc_maker_open_fees() -> Vec<BaseFee> {
	vec![
		BaseFee { volume: FixedI128::from_u32(0), fee: FixedI128::from_float(0.02) },
		BaseFee { volume: FixedI128::from_u32(1000000), fee: FixedI128::from_float(0.015) },
		BaseFee { volume: FixedI128::from_u32(5000000), fee: FixedI128::from_float(0.010) },
		BaseFee { volume: FixedI128::from_u32(10000000), fee: FixedI128::from_float(0.005) },
		BaseFee { volume: FixedI128::from_u32(50000000), fee: FixedI128::from_float(0.0) },
	]
}

fn get_btc_usdc_maker_open_fees() -> Vec<BaseFee> {
	vec![
		BaseFee { volume: FixedI128::from_u32(0), fee: FixedI128::from_float(0.002) },
		BaseFee { volume: FixedI128::from_u32(10000), fee: FixedI128::from_float(0.001) },
		BaseFee { volume: FixedI128::from_u32(1000000), fee: FixedI128::from_float(0.0) },
	]
}

fn get_usdt_maker_open_fees() -> Vec<BaseFee> {
	vec![
		BaseFee { volume: FixedI128::from_u32(0), fee: FixedI128::from_float(0.02) },
		BaseFee { volume: FixedI128::from_u32(1000000), fee: FixedI128::from_float(0.015) },
		BaseFee { volume: FixedI128::from_u32(5000000), fee: FixedI128::from_float(0.0) },
	]
}

fn get_usdc_maker_close_fees() -> Vec<BaseFee> {
	vec![
		BaseFee { volume: FixedI128::from_u32(0), fee: FixedI128::from_float(0.02) },
		BaseFee { volume: FixedI128::from_u32(1000000), fee: FixedI128::from_float(0.015) },
		BaseFee { volume: FixedI128::from_u32(5000000), fee: FixedI128::from_float(0.010) },
		BaseFee { volume: FixedI128::from_u32(10000000), fee: FixedI128::from_float(0.005) },
		BaseFee { volume: FixedI128::from_u32(50000000), fee: FixedI128::from_float(0.0) },
	]
}

fn get_btc_usdc_maker_close_fees() -> Vec<BaseFee> {
	vec![
		BaseFee { volume: FixedI128::from_u32(0), fee: FixedI128::from_float(0.002) },
		BaseFee { volume: FixedI128::from_u32(10000), fee: FixedI128::from_float(0.001) },
		BaseFee { volume: FixedI128::from_u32(1000000), fee: FixedI128::from_float(0.0) },
	]
}

fn get_usdt_maker_close_fees() -> Vec<BaseFee> {
	vec![
		BaseFee { volume: FixedI128::from_u32(0), fee: FixedI128::from_float(0.02) },
		BaseFee { volume: FixedI128::from_u32(1000000), fee: FixedI128::from_float(0.015) },
		BaseFee { volume: FixedI128::from_u32(5000000), fee: FixedI128::from_float(0.0) },
	]
}

fn get_usdc_taker_open_fees() -> Vec<BaseFee> {
	vec![
		BaseFee { volume: FixedI128::from_u32(0), fee: FixedI128::from_float(0.050) },
		BaseFee { volume: FixedI128::from_u32(1000000), fee: FixedI128::from_float(0.040) },
		BaseFee { volume: FixedI128::from_u32(5000000), fee: FixedI128::from_float(0.035) },
		BaseFee { volume: FixedI128::from_u32(10000000), fee: FixedI128::from_float(0.030) },
		BaseFee { volume: FixedI128::from_u32(50000000), fee: FixedI128::from_float(0.025) },
		BaseFee { volume: FixedI128::from_u32(200000000), fee: FixedI128::from_float(0.020) },
	]
}

fn get_btc_usdc_taker_open_fees() -> Vec<BaseFee> {
	vec![
		BaseFee { volume: FixedI128::from_u32(0), fee: FixedI128::from_float(0.005) },
		BaseFee { volume: FixedI128::from_u32(10000), fee: FixedI128::from_float(0.0045) },
		BaseFee { volume: FixedI128::from_u32(1000000), fee: FixedI128::from_float(0.004) },
		BaseFee { volume: FixedI128::from_u32(5000000), fee: FixedI128::from_float(0.002) },
	]
}

fn get_usdt_taker_open_fees() -> Vec<BaseFee> {
	vec![
		BaseFee { volume: FixedI128::from_u32(0), fee: FixedI128::from_float(0.050) },
		BaseFee { volume: FixedI128::from_u32(1000000), fee: FixedI128::from_float(0.040) },
		BaseFee { volume: FixedI128::from_u32(5000000), fee: FixedI128::from_float(0.035) },
		BaseFee { volume: FixedI128::from_u32(10000000), fee: FixedI128::from_float(0.030) },
		BaseFee { volume: FixedI128::from_u32(50000000), fee: FixedI128::from_float(0.025) },
	]
}

fn get_usdc_taker_close_fees() -> Vec<BaseFee> {
	vec![
		BaseFee { volume: FixedI128::from_u32(0), fee: FixedI128::from_float(0.050) },
		BaseFee { volume: FixedI128::from_u32(1000000), fee: FixedI128::from_float(0.040) },
		BaseFee { volume: FixedI128::from_u32(5000000), fee: FixedI128::from_float(0.035) },
		BaseFee { volume: FixedI128::from_u32(10000000), fee: FixedI128::from_float(0.030) },
		BaseFee { volume: FixedI128::from_u32(50000000), fee: FixedI128::from_float(0.025) },
		BaseFee { volume: FixedI128::from_u32(200000000), fee: FixedI128::from_float(0.020) },
	]
}

fn get_btc_usdc_taker_close_fees() -> Vec<BaseFee> {
	vec![
		BaseFee { volume: FixedI128::from_u32(0), fee: FixedI128::from_float(0.005) },
		BaseFee { volume: FixedI128::from_u32(10000), fee: FixedI128::from_float(0.0045) },
		BaseFee { volume: FixedI128::from_u32(1000000), fee: FixedI128::from_float(0.004) },
		BaseFee { volume: FixedI128::from_u32(5000000), fee: FixedI128::from_float(0.002) },
	]
}

fn get_usdt_taker_close_fees() -> Vec<BaseFee> {
	vec![
		BaseFee { volume: FixedI128::from_u32(0), fee: FixedI128::from_float(0.050) },
		BaseFee { volume: FixedI128::from_u32(1000000), fee: FixedI128::from_float(0.040) },
		BaseFee { volume: FixedI128::from_u32(5000000), fee: FixedI128::from_float(0.035) },
		BaseFee { volume: FixedI128::from_u32(10000000), fee: FixedI128::from_float(0.030) },
		BaseFee { volume: FixedI128::from_u32(50000000), fee: FixedI128::from_float(0.025) },
	]
}
