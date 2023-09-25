use frame_support::inherent::Vec;
use primitive_types::U256;
use zkx_support::FieldElement;
use zkx_support::helpers::pedersen_hash_multiple;
use zkx_support::traits::FeltSerializedArrayExt;
use sp_runtime::BoundedVec;
use sp_runtime::traits::ConstU32;
use zkx_support::types::{Asset, Market, UniversalEventL2, MarketUpdatedL2, AssetUpdatedL2, MarketRemovedL2, AssetRemovedL2, UserDepositL2, TradingAccountMinimal};

pub trait MarketUpdatedL2Trait {
    fn new(id: u64, market: Market, version: u16, block_number: u64) -> MarketUpdatedL2;
}

pub trait AssetUpdatedL2Trait {
    fn new(id: u64, asset: Asset, version: u16, block_number: u64) -> AssetUpdatedL2;
}

pub trait MarketRemovedL2Trait {
    fn new(id: u64, block_number: u64) -> MarketRemovedL2;
}

pub trait AssetRemovedL2Trait {
    fn new(id: u64, block_number: u64) -> AssetRemovedL2;
}

pub trait UserDepositL2Trait {
    fn new(id: u64, trading_account: TradingAccountMinimal, collateral_id: u64, nonce: U256, amount: U256, balance: U256, block_number: u64) -> UserDepositL2;
}


impl MarketUpdatedL2Trait for MarketUpdatedL2 {
    fn new(id: u64, market: Market, version: u16, block_number: u64) -> MarketUpdatedL2 {
        MarketUpdatedL2 {
            event_hash: U256::from(123),
            event_name: U256::from_str_radix("0x4D61676B6574557064617465644C32", 16).unwrap(),
            id: id,
            market: market,
            metadata_url: BoundedVec::new(),
            icon_url: BoundedVec::new(),
            version: version,
            block_number: block_number
        }
    }
}

impl AssetUpdatedL2Trait for AssetUpdatedL2 {
    fn new(id: u64, asset: Asset, version: u16, block_number: u64) -> AssetUpdatedL2 {
        AssetUpdatedL2 {
            event_hash: U256::from(987),
            event_name: U256::from_str_radix("0x4173736574557064617465644C32", 16).unwrap(),
            id: id,
            asset: asset,
            metadata_url: BoundedVec::new(),
            icon_url: BoundedVec::new(),
            version: version,
            block_number: block_number
        }
    }
}

impl MarketRemovedL2Trait for MarketRemovedL2 {
    fn new(id: u64, block_number: u64) -> MarketRemovedL2 {
        MarketRemovedL2 {
            event_hash: U256::from(1234),
            event_name: U256::from_str_radix("0x4D61726B657452656D6F7665644C32", 16).unwrap(),
            id: id,
            block_number: block_number
        }
    }
}

impl AssetRemovedL2Trait for AssetRemovedL2 {
    fn new(id: u64, block_number: u64) -> AssetRemovedL2 {
        AssetRemovedL2 {
            event_hash: U256::from(9876),
            event_name: U256::from_str_radix("0x417373657452656D6F7665644C32", 16).unwrap(),
            id: id,
            block_number: block_number
        }
    }
}
impl UserDepositL2Trait for UserDepositL2 {
    fn new(id: u64, trading_account: TradingAccountMinimal, collateral_id: u64, nonce: U256, amount: U256, balance: U256, block_number: u64) -> UserDepositL2 {
        UserDepositL2 {
            event_hash: U256::from(9876),
            event_name: U256::from_str_radix("0x557365724465706F7369744C32", 16).unwrap(),
            trading_account: trading_account,
            collateral_id: collateral_id,
            nonce: nonce,
            amount: amount,
            balance: balance,
            block_number: block_number
        }
    }
}

pub trait UniversalEventArray {
    fn new() -> Vec<UniversalEventL2>;
    fn add_market_updated_event(&mut self, market_updated_event: MarketUpdatedL2 );
    fn add_asset_updated_event(&mut self, asset_updated_event: AssetUpdatedL2);
    fn add_market_removed_event(&mut self, market_removed_event: MarketRemovedL2);
    fn add_asset_removed_event(&mut self, asset_removed_event: AssetRemovedL2);
    fn add_user_deposit_event(&mut self, user_deposit_event: UserDepositL2);
    fn compute_hash(&self) -> FieldElement;
}

impl UniversalEventArray for Vec<UniversalEventL2> {
    fn new() -> Vec<UniversalEventL2> {
        Vec::<UniversalEventL2>::new()
    }

    fn add_market_updated_event(&mut self, market_updated_event: MarketUpdatedL2 ){
        self.push(UniversalEventL2::MarketUpdatedL2(market_updated_event));
    }

    fn add_asset_updated_event(&mut self, asset_updated_event: AssetUpdatedL2) {
        self.push(UniversalEventL2::AssetUpdatedL2(asset_updated_event));
    }

    fn add_market_removed_event(&mut self, market_removed_event: MarketRemovedL2){
        self.push(UniversalEventL2::MarketRemovedL2(market_removed_event));
    }

    fn add_asset_removed_event(&mut self, asset_removed_event: AssetRemovedL2){
        self.push(UniversalEventL2::AssetRemovedL2(asset_removed_event));
    }

    fn add_user_deposit_event(&mut self, user_deposit_event: UserDepositL2){
        self.push(UniversalEventL2::UserDepositL2(user_deposit_event));
    }

    fn compute_hash(&self) -> FieldElement {
        let mut flattened_array: Vec<FieldElement> = Vec::new();
        flattened_array.try_append_universal_event_array(&self).unwrap();

        // Compute hash of the array and return
        pedersen_hash_multiple(&flattened_array)
    }
}