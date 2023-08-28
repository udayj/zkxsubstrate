use codec::{Decode, Encode};
use frame_support::pallet_prelude::*;
use frame_support::inherent::Vec;
use primitive_types::U256;
use scale_info::TypeInfo;
use sp_arithmetic::FixedPointNumber;
use sp_arithmetic::fixed_point::FixedI128;
use sp_runtime::RuntimeDebug;
use starknet_ff::FieldElement;
use starknet_crypto::poseidon_hash_many;

use super::traits::Hashable;
use super::{FixedI128_to_U256, pedersen_hash_multiple};

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

impl From<Direction> for u8 {

	fn from(value: Direction) -> u8 {

		match value {
			Direction::Long => 0_u8,
			Direction::Short => 1_u8
		}
	}
}

#[derive(Clone, Copy, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub enum Side {
	#[default]
	Buy,
	Sell,
}

impl From<Side> for u8 {

	fn from(value: Side) -> u8 {

		match value {
			Side::Buy => 0_u8,
			Side::Sell => 1_u8
		}
	}
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

impl From<TimeInForce> for u8 {

	fn from(value: TimeInForce) -> u8 {

		match value {
			TimeInForce::GTC => 0_u8,
			TimeInForce::IOC => 1_u8,
			TimeInForce::FOK => 2_u8
		}
	}
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

#[derive(Clone, Decode, Default, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub enum HashType {
	#[default]
	Pedersen,
	Poseidon
}

impl Hashable for Order {

	fn hash(&self, hash_type: HashType) -> FieldElement{

		let mut elements: Vec<FieldElement> = Vec::new();
		let mut buffer:[u8;32] = [0;32];
		self.user.to_big_endian(&mut buffer);
		elements.push(FieldElement::from_byte_slice_be(&buffer).unwrap());

		elements.push(FieldElement::from(self.order_id));

		self.market_id.to_big_endian(&mut buffer);
		elements.push(FieldElement::from_byte_slice_be(&buffer).unwrap());

		elements.push(FieldElement::from(u8::from(self.order_type)));
		elements.push(FieldElement::from(u8::from(self.direction)));
		elements.push(FieldElement::from(u8::from(self.side)));


		let u256_representation = FixedI128_to_U256(self.price);
		u256_representation.to_big_endian(&mut buffer);
		elements.push(FieldElement::from_byte_slice_be(&buffer).unwrap()); // try using from_byte_slice_be

		let u256_representation = FixedI128_to_U256(self.size);
		u256_representation.to_big_endian(&mut buffer);
		elements.push(FieldElement::from_byte_slice_be(&buffer).unwrap());

		let u256_representation = FixedI128_to_U256(self.leverage);
		u256_representation.to_big_endian(&mut buffer);
		elements.push(FieldElement::from_byte_slice_be(&buffer).unwrap());

		let u256_representation = FixedI128_to_U256(self.slippage);
		u256_representation.to_big_endian(&mut buffer);
		elements.push(FieldElement::from_byte_slice_be(&buffer).unwrap());

		match self.post_only {
			true => elements.push(FieldElement::from(1_u8)),
			false => elements.push(FieldElement::from(0_u8))
		}


		elements.push(FieldElement::from(u8::from(self.time_in_force)));
		// try using U256 from sp_core
		//elements.push();
		match hash_type {
			HashType::Pedersen => pedersen_hash_multiple(&elements),
			HashType::Poseidon => poseidon_hash_many(&elements)
		}
		

	}
}