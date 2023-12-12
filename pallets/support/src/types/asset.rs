use codec::{Decode, Encode};
use primitive_types::U256;
use scale_info::TypeInfo;
use sp_runtime::{traits::ConstU32, BoundedVec, RuntimeDebug};

#[derive(Clone, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Asset {
	pub id: u128,
	pub version: u16,
	pub short_name: U256,
	pub is_tradable: bool,
	pub is_collateral: bool,
	pub decimals: u8,
}

#[derive(Clone, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub struct AssetAddress {
	pub chain: U256,
	pub address: U256,
}

#[derive(Clone, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub struct ExtendedAsset {
	pub asset: Asset,
	pub asset_addresses: BoundedVec<AssetAddress, ConstU32<256>>,
	pub metadata_url: BoundedVec<u8, ConstU32<256>>,
}
