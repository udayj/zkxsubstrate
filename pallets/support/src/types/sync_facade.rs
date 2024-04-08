use crate::types::{Asset, AssetAddress, Market, TradingAccountMinimal};
use codec::{Decode, Encode};
use primitive_types::U256;
use scale_info::TypeInfo;
use sp_arithmetic::FixedI128;
use sp_runtime::{traits::ConstU32, BoundedVec, RuntimeDebug};

#[derive(Clone, Decode, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct SyncSignature {
	pub signer_pub_key: U256,
	pub r: U256,
	pub s: U256,
}

#[derive(Clone, Decode, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Setting {
	pub key: U256,
	pub values: BoundedVec<FixedI128, ConstU32<256>>,
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
	SettingsAdded(SettingsAdded),
	ReferralDetailsAdded(ReferralDetailsAdded),
	MasterAccountLevelChanged(MasterAccountLevelChanged),
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
	pub asset_addresses: BoundedVec<AssetAddress, ConstU32<256>>,
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

#[derive(Clone, Decode, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct SettingsAdded {
	pub event_index: u32,
	pub settings: BoundedVec<Setting, ConstU32<256>>,
	pub block_number: u64,
}

#[derive(Clone, Decode, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct ReferralDetailsAdded {
	pub event_index: u32,
	pub master_account_address: U256,
	pub referral_account_address: U256,
	pub referral_code: U256,
	pub fee_discount: FixedI128,
	pub block_number: u64,
}

#[derive(Clone, Decode, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct MasterAccountLevelChanged {
	pub event_index: u32,
	pub master_account_address: U256,
	pub level: u8,
	pub block_number: u64,
}

#[derive(Clone, Copy, Debug, Decode, Encode, Eq, Hash, PartialEq, TypeInfo)]
pub enum SettingsType {
	FeeSettings(FeeSettingsType),
	ABRSettings(ABRSettingsType),
	GeneralSettings,
}

#[derive(Clone, Copy, Debug, Decode, Encode, Eq, Hash, PartialEq, TypeInfo)]
pub enum FeeSettingsType {
	MakerVols,
	TakerVols,
	MakerOpen,
	MakerClose,
	TakerOpen,
	TakerClose,
}

#[derive(Clone, Copy, Debug, Decode, Encode, Eq, Hash, PartialEq, TypeInfo)]
pub enum ABRSettingsType {
	MaxDefault,
	MaxPerMarket,
}
