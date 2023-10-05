use frame_support::inherent::Vec;
use primitive_types::U256;
use sp_arithmetic::fixed_point::FixedI128;
use sp_runtime::traits::ConstU32;
use sp_runtime::BoundedVec;
use zkx_support::helpers::pedersen_hash_multiple;
use zkx_support::traits::FeltSerializedArrayExt;
use zkx_support::types::{
	Asset, AssetRemoved, AssetUpdated, Market, MarketRemoved, MarketUpdated, SignerAdded,
	SignerRemoved, TradingAccountMinimal, UniversalEvent, UserDeposit,
};
use zkx_support::FieldElement;

pub trait MarketUpdatedTrait {
	fn new(
		id: u64,
		market: Market,
		metadata_url: BoundedVec<u8, ConstU32<256>>,
		block_number: u64,
	) -> MarketUpdated;
}

pub trait AssetUpdatedTrait {
	fn new(
		id: u64,
		asset: Asset,
		metadata_url: BoundedVec<u8, ConstU32<256>>,
		icon_url: BoundedVec<u8, ConstU32<256>>,
		block_number: u64,
	) -> AssetUpdated;
}

pub trait MarketRemovedTrait {
	fn new(id: u64, block_number: u64) -> MarketRemoved;
}

pub trait AssetRemovedTrait {
	fn new(id: u64, block_number: u64) -> AssetRemoved;
}

pub trait UserDepositTrait {
	fn new(
		trading_account: TradingAccountMinimal,
		collateral_id: u128,
		nonce: U256,
		amount: FixedI128,
		block_number: u64,
	) -> UserDeposit;
}

pub trait SignerAddedTrait {
	fn new(signer: U256, block_number: u64) -> SignerAdded;
}

pub trait SignerRemovedTrait {
	fn new(signer: U256, block_number: u64) -> SignerRemoved;
}

impl MarketUpdatedTrait for MarketUpdated {
	fn new(
		id: u64,
		market: Market,
		metadata_url: BoundedVec<u8, ConstU32<256>>,
		block_number: u64,
	) -> MarketUpdated {
		MarketUpdated { id, market, metadata_url, block_number }
	}
}

impl AssetUpdatedTrait for AssetUpdated {
	fn new(
		id: u64,
		asset: Asset,
		metadata_url: BoundedVec<u8, ConstU32<256>>,
		icon_url: BoundedVec<u8, ConstU32<256>>,
		block_number: u64,
	) -> AssetUpdated {
		AssetUpdated { id, asset, metadata_url, icon_url, block_number }
	}
}

impl MarketRemovedTrait for MarketRemoved {
	fn new(id: u64, block_number: u64) -> MarketRemoved {
		MarketRemoved { id, block_number }
	}
}

impl AssetRemovedTrait for AssetRemoved {
	fn new(id: u64, block_number: u64) -> AssetRemoved {
		AssetRemoved { id, block_number }
	}
}
impl UserDepositTrait for UserDeposit {
	fn new(
		trading_account: TradingAccountMinimal,
		collateral_id: u128,
		nonce: U256,
		amount: FixedI128,
		block_number: u64,
	) -> UserDeposit {
		UserDeposit { trading_account, collateral_id, nonce, amount, block_number }
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
	fn compute_hash(&self) -> FieldElement;
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

	fn compute_hash(&self) -> FieldElement {
		let mut flattened_array: Vec<FieldElement> = Vec::new();
		flattened_array.try_append_universal_event_array(&self).unwrap();

		// Compute hash of the array and return
		pedersen_hash_multiple(&flattened_array)
	}
}
