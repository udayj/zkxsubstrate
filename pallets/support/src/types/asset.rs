use codec::{Decode, Encode};
use primitive_types::U256;
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

#[derive(Clone, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Asset {
	pub id: u128,
	pub version: u16,
	pub short_name: U256,
	pub is_tradable: bool,
	pub is_collateral: bool,
	pub l2_address: U256,
	pub decimals: u8,
}
