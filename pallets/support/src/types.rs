use codec::{Decode, Encode};
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_arithmetic::fixed_point::FixedI128;
use sp_arithmetic::per_things::Percent;
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
pub struct TradingAccount {
	pub account_id: [u8; 32],
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
pub struct BalanceUpdate {
	pub asset_id: u64,
	pub balance_value: FixedI128,
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
	pub tick_size: FixedI128,
	pub tick_precision: u8,
	pub step_size: FixedI128,
	pub step_precision: u8,
	pub minimum_order_size: FixedI128,
	pub minimum_leverage: FixedI128,
	pub maximum_leverage: FixedI128,
	pub currently_allowed_leverage: FixedI128,
	pub maintenance_margin_fraction: FixedI128,
	pub initial_margin_fraction: FixedI128,
	pub incremental_initial_margin_fraction: FixedI128,
	pub incremental_position_size: FixedI128,
	pub baseline_position_size: FixedI128,
	pub maximum_position_size: FixedI128,
}

#[derive(Clone, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub enum Direction {
	#[default]
	Long,
	Short,
}

#[derive(Clone, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub enum Side {
	#[default]
	Buy,
	Sell,
}

#[derive(Clone, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Order {
	pub user: TradingAccount,
	pub order_id: u128,
	pub market_id: u64,
	pub direction: Direction,
	pub side: Side,
	pub price: FixedI128,
	pub size: FixedI128,
	pub leverage: FixedI128,
	pub slippage: Percent,
}

#[derive(Clone, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Position {
	pub avg_execution_price: FixedI128,
	pub size: FixedI128,
	pub margin_amount: FixedI128,
	pub borrowed_amount: FixedI128,
	pub leverage: FixedI128,
}
