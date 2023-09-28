use crate::types::{Asset, Market, TradingAccountWithoutId};
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
pub enum UniversalEvent {
	MarketUpdated(MarketUpdated),
	AssetUpdated(AssetUpdated),
	MarketRemoved(MarketRemoved),
	AssetRemoved(AssetRemoved),
	UserDeposit(UserDeposit),
	SignerAdded(SignerAdded),
	SignerRemoved(SignerRemoved)
}

#[derive(Clone, Copy, Decode, Default, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct MarketRemoved {
	pub id: u64,
	pub block_number: u64,
}

#[derive(Clone, Copy, Decode, Default, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct AssetRemoved {
	pub id: u64,
	pub block_number: u64,
}

#[derive(Clone, Copy, Decode, Default, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct UserDeposit {
	pub trading_account: TradingAccountWithoutId,
	pub collateral_id: u128,
	pub nonce: U256,
	pub amount: U256,
	pub block_number: u64,
}

#[derive(Clone, Decode, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct MarketUpdated {
	pub id: u64,
	pub market: Market,
	pub metadata_url: BoundedVec<u8, ConstU32<256>>,
	pub block_number: u64,
}

#[derive(Clone, Decode, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct AssetUpdated {
	pub id: u64,
	pub asset: Asset,
	pub metadata_url: BoundedVec<u8, ConstU32<256>>,
	pub icon_url: BoundedVec<u8, ConstU32<256>>,
	pub block_number: u64,
}


#[derive(Clone, Decode, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct SignerAdded {
    pub signer: U256,
    pub block_number: u64
}

#[derive(Clone, Decode, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct SignerRemoved {
    pub signer: U256,
    pub block_number: u64
}