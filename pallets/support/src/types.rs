use codec::{Decode, Encode};
use frame_support::pallet_prelude::*;
use frame_support::inherent::Vec;
use primitive_types::U256;
use scale_info::TypeInfo;
use sp_arithmetic::fixed_point::FixedI128;
use sp_runtime::RuntimeDebug;
use starknet_ff::{FieldElement, FromByteSliceError};
use starknet_crypto::poseidon_hash_many;

use super::traits::Hashable;
use super::helpers::{fixed_i128_to_u256, u256_to_field_element, pedersen_hash_multiple};

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

	// No error apart from error during conversion from U256 to FieldElement should happen
	// Hence associated type is defined to be exactly that error i.e. starknet_ff::FromByteSliceError
	type ConversionError = FromByteSliceError;
	fn hash(&self, hash_type: HashType) -> Result<FieldElement, Self::ConversionError>{

		let mut elements: Vec<FieldElement> = Vec::new();
		
		elements.push(u256_to_field_element(&self.user)?);

		elements.push(FieldElement::from(self.order_id));

		elements.push(u256_to_field_element(&self.market_id)?);

		elements.push(FieldElement::from(u8::from(self.order_type)));
		elements.push(FieldElement::from(u8::from(self.direction)));
		elements.push(FieldElement::from(u8::from(self.side)));

		let u256_representation = fixed_i128_to_u256(&self.price);
		elements.push(u256_to_field_element(&u256_representation)?);

		let u256_representation = fixed_i128_to_u256(&self.size);
		elements.push(u256_to_field_element(&u256_representation)?);

		let u256_representation = fixed_i128_to_u256(&self.leverage);
		elements.push(u256_to_field_element(&u256_representation)?);

		let u256_representation = fixed_i128_to_u256(&self.slippage);
		elements.push(u256_to_field_element(&u256_representation)?);

		match self.post_only {
			true => elements.push(FieldElement::from(1_u8)),
			false => elements.push(FieldElement::from(0_u8))
		}

		elements.push(FieldElement::from(u8::from(self.time_in_force)));
		
		match hash_type {
			HashType::Pedersen => Ok(pedersen_hash_multiple(&elements)),
			HashType::Poseidon => Ok(poseidon_hash_many(&elements))
		}		
	}
}