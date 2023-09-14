use crate::types::{Asset, Market, TradingAccountMinimal};
use codec::{Decode, Encode};
use primitive_types::U256;
use scale_info::TypeInfo;
use sp_runtime::traits::ConstU32;
use sp_runtime::{BoundedVec, RuntimeDebug};

#[derive(Clone, Decode, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct SyncSignature {
	pub signer_pub_key: U256,
	pub r: U256,
	pub s: U256,
}

#[derive(Clone, Decode, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub enum UniversalEventL2 {
	MarketUpdatedL2(MarketUpdatedL2),
	AssetUpdatedL2(AssetUpdatedL2),
	MarketRemovedL2(MarketRemovedL2),
	AssetRemovedL2(AssetRemovedL2),
	UserDepositL2(UserDepositL2),
}

#[derive(Clone, Copy, Decode, Default, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct MarketRemovedL2 {
	pub event_hash: U256,
	pub event_name: U256,
	pub id: u64,
	pub block_number: u64,
}

#[derive(Clone, Copy, Decode, Default, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct AssetRemovedL2 {
	pub event_hash: U256,
	pub event_name: U256,
	pub id: u64,
	pub block_number: u64,
}

#[derive(Clone, Copy, Decode, Default, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct UserDepositL2 {
	pub event_hash: U256,
	pub event_name: U256,
	pub trading_account: TradingAccountMinimal,
	pub collateral_id: u64,
	pub nonce: U256,
	pub amount: U256,
	pub balance: U256,
	pub block_number: u64,
}

#[derive(Clone, Copy, Decode, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub enum FundType {
	Admin,
	InsuranceFund,
	HoldingFund,
	EmergencyFund,
}

#[derive(Clone, Decode, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct MarketUpdatedL2 {
	pub event_hash: U256,
	pub event_name: U256,
	pub id: u64,
	pub market: Market,
	pub metadata_url: BoundedVec<u8, ConstU32<256>>,
	pub icon_url: BoundedVec<u8, ConstU32<256>>,
	pub version: u16,
	pub block_number: u64,
}

#[derive(Clone, Decode, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct AssetUpdatedL2 {
	pub event_hash: U256,
	pub event_name: U256,
	pub id: u64,
	pub asset: Asset,
	pub metadata_url: BoundedVec<u8, ConstU32<256>>,
	pub icon_url: BoundedVec<u8, ConstU32<256>>,
	pub version: u16,
	pub block_number: u64,
}
