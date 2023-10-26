use crate::types::{Asset, Market, TradingAccountMinimal};
use codec::{Decode, Encode};
use primitive_types::U256;
use scale_info::TypeInfo;
use sp_arithmetic::FixedI128;
use sp_runtime::traits::ConstU32;
use sp_runtime::{BoundedVec, RuntimeDebug};

#[derive(Clone, Decode, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct SyncSignature {
	pub signer_pub_key: U256,
	pub r: U256,
	pub s: U256,
}

#[derive(Clone, Decode, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub enum UniversalEvent {
	MarketUpdated(MarketUpdated),
	AssetUpdated(AssetUpdated),
	MarketRemoved(MarketRemoved),
	AssetRemoved(AssetRemoved),
	UserDeposit(UserDeposit),
	SignerAdded(SignerAdded),
	SignerRemoved(SignerRemoved),
	QuorumSet(QuorumSet),
}

#[derive(Clone, Copy, Decode, Default, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct MarketRemoved {
	pub event_index: u32,
	pub id: u128,
	pub block_number: u64,
}

#[derive(Clone, Copy, Decode, Default, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct AssetRemoved {
	pub event_index: u32,
	pub id: u128,
	pub block_number: u64,
}

#[derive(Clone, Copy, Decode, Default, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct UserDeposit {
	pub event_index: u32,
	pub trading_account: TradingAccountMinimal,
	pub collateral_id: u128,
	pub nonce: U256,
	pub amount: FixedI128,
	pub block_number: u64,
}

#[derive(Clone, Decode, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct MarketUpdated {
	pub event_index: u32,
	pub id: u128,
	pub market: Market,
	pub metadata_url: BoundedVec<u8, ConstU32<256>>,
	pub block_number: u64,
}

#[derive(Clone, Decode, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct AssetUpdated {
	pub event_index: u32,
	pub id: u128,
	pub asset: Asset,
	pub metadata_url: BoundedVec<u8, ConstU32<256>>,
	pub block_number: u64,
}

#[derive(Clone, Decode, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct SignerAdded {
	pub event_index: u32,
	pub signer: U256,
	pub block_number: u64,
}

#[derive(Clone, Decode, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct SignerRemoved {
	pub event_index: u32,
	pub signer: U256,
	pub block_number: u64,
}

#[derive(Clone, Decode, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct QuorumSet {
	pub event_index: u32,
	pub quorum: u8,
	pub block_number: u64,
}
