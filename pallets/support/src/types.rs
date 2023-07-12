use codec::{Decode, Encode};
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
pub struct TradingAccount {
	pub account_id: [u8; 32],
}

#[derive(Clone, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Asset {
	pub id: u64,
	pub name: BoundedVec<u8, ConstU32<50>>,
	pub is_tradable: bool,
	pub is_collateral: bool,
	pub token_decimal: u8,
}

#[derive(Clone, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Market {
	pub id: u64,
	pub asset: u64,
	pub asset_collateral: u64,
	pub is_tradable: u8,
	pub is_archived: bool,
	pub ttl: u32,
	pub tick_size: u64,
	pub tick_precision: u8,
	pub step_size: u64,
	pub step_precision: u8,
	pub minimum_order_size: u64,
	pub minimum_leverage: u8,
	pub maximum_leverage: u8,
	pub currently_allowed_leverage: u8,
	pub maintenance_margin_fraction: u64,
	pub initial_margin_fraction: u64,
	pub incremental_initial_margin_fraction: u64,
	pub incremental_position_size: u64,
	pub baseline_position_size: u64,
	pub maximum_position_size: u64,
}
