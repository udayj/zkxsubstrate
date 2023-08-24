use codec::{Decode, Encode};
use frame_support::pallet_prelude::*;
use frame_support::inherent::Vec;
use primitive_types::U256;
use scale_info::TypeInfo;
use sp_arithmetic::FixedPointNumber;
use sp_arithmetic::fixed_point::FixedI128;
use sp_runtime::RuntimeDebug;
use starknet_ff::FieldElement;
use starknet_crypto::pedersen_hash;


#[derive(
	Encode, Decode, Default, Clone, Copy, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug,
)]
pub struct TradingAccount {
	pub account_id: U256,
	pub account_address: U256,
	pub index: u8,
	pub pub_key: U256,
}

#[derive(
	Encode, Decode, Default, Clone, Copy, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug,
)]
pub struct TradingAccountWithoutId {
	pub account_address: U256,
	pub index: u8,
	pub pub_key: U256,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
pub struct BalanceUpdate {
	pub asset_id: U256,
	pub balance_value: FixedI128,
}

#[derive(Clone, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Asset {
	pub id: U256,
	pub name: BoundedVec<u8, ConstU32<50>>,
	pub is_tradable: bool,
	pub is_collateral: bool,
	pub token_decimal: u8,
}

#[derive(Clone, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Market {
	pub id: U256,
	pub asset: U256,
	pub asset_collateral: U256,
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

#[derive(Clone, Copy, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub enum Direction {
	#[default]
	Long,
	Short,
}

#[derive(Clone, Copy, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub enum Side {
	#[default]
	Buy,
	Sell,
}

#[derive(Clone, Copy, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub enum OrderType {
	#[default]
	Limit,
	Market,
}

impl From<OrderType> for u8 {

	fn from(value: OrderType) -> u8 {

		match value {
			OrderType::Limit => 0_u8,
			OrderType::Market => 1_u8
		}
	}
}

#[derive(Clone, Copy, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub enum TimeInForce {
	#[default]
	GTC,
	IOC,
	FOK,
}

#[derive(Clone, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Order {
	pub user: U256,
	pub order_id: u128,
	pub market_id: U256,
	pub order_type: OrderType,
	pub direction: Direction,
	pub side: Side,
	pub price: FixedI128,
	pub size: FixedI128,
	pub leverage: FixedI128,
	pub slippage: FixedI128,
	pub post_only: bool,
	pub time_in_force: TimeInForce,
}

#[derive(Clone, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Position {
	pub avg_execution_price: FixedI128,
	pub size: FixedI128,
	pub margin_amount: FixedI128,
	pub borrowed_amount: FixedI128,
	pub leverage: FixedI128,
	pub realized_pnl: FixedI128,
}

#[derive(Clone, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub struct ErrorEventList {
	pub order_id: u128,
	pub error_code: u16,
}

#[derive(Clone, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub struct OrderEventList {
	pub user: U256,
	pub order_id: u128,
	pub market_id: U256,
	pub size: FixedI128,
	pub direction: Direction,
	pub side: Side,
	pub order_type: OrderType,
	pub execution_price: FixedI128,
	pub pnl: FixedI128,
	pub opening_fee: FixedI128,
}

#[derive(Clone, Copy, Decode, Default, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct BaseFee {
	pub number_of_tokens: U256,
	pub maker_fee: FixedI128,
	pub taker_fee: FixedI128,
}

#[derive(Clone, Copy, Decode, Default, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Discount {
	pub number_of_tokens: U256,
	pub discount: FixedI128,
}

#[derive(Clone, Decode, Default, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub enum OrderSide {
	#[default]
	Maker,
	Taker,
}

impl Order {

	pub fn hash_elements(&self) -> FieldElement{

		let mut elements: Vec<FieldElement> = Vec::new();
		elements.push(FieldElement::from(self.order_id));
		elements.push(FieldElement::from(u8::from(self.order_type)));
		let inner_val:U256;
		let max = U256::from_dec_str("3618502788666131213697322783095070105623107215331596699973092056135872020481").unwrap();
		
		if (self.price.into_inner() > 0 ){
			inner_val = U256::from(self.price.into_inner()); // multiply by 10^20
		}
		else {
			inner_val = max-U256::from(self.price.into_inner()*(-1));
		}
		let mut be_bytes:[u8;32];
		inner_val.to_big_endian(&mut be_bytes);
		elements.push(FieldElement::from_bytes_be(be_bytes).unwrap());

		//elements.push();
		pedersen_hash(&elements[0], &elements[1])

	}
}