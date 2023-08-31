use codec::{Decode, Encode};
use frame_support::inherent::Vec;
use frame_support::pallet_prelude::*;
use primitive_types::U256;
use scale_info::TypeInfo;
use sp_arithmetic::fixed_point::FixedI128;
use sp_runtime::RuntimeDebug;
use starknet_crypto::poseidon_hash_many;
use starknet_ff::{FieldElement, FromByteSliceError};

use super::helpers::{fixed_i128_to_u256, pedersen_hash_multiple, u256_to_field_element};
use super::traits::Hashable;

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
	pub is_tradable: bool,
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
pub struct MarketPrice {
	pub asset_id: U256,
	pub collateral_id: U256,
	pub timestamp: u64,
	pub price: FixedI128,
}

#[derive(Clone, Copy, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub struct MultipleMarketPrices {
	pub market_id: U256,
	pub price: FixedI128,
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
			Direction::Short => 1_u8,
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
			Side::Sell => 1_u8,
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
			OrderType::Market => 1_u8,
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
			TimeInForce::FOK => 2_u8,
		}
	}
}

#[derive(Clone, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Order {
	pub account_id: U256,
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
	pub direction: Direction,
	pub side: Side,
	pub avg_execution_price: FixedI128,
	pub size: FixedI128,
	pub margin_amount: FixedI128,
	pub borrowed_amount: FixedI128,
	pub leverage: FixedI128,
	pub realized_pnl: FixedI128,
}

#[derive(Clone, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub struct FailedOrder {
	pub order_id: u128,
	pub error_code: u16,
}

#[derive(Clone, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub struct ExecutedBatch {
	batch_id: U256,
	market_id: U256,
	size: FixedI128,
	execution_price: FixedI128,
	direction: Direction,
	side: Side,
}

#[derive(Clone, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub struct ExecutedOrder {
	pub account_id: U256,
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
	Poseidon,
}

#[derive(Clone, Decode, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct MarketUpdatedL2 {
	pub event_hash: U256,
	pub event_name: U256,
	pub id: u64,
	pub market: Market,
	pub metadata_url: BoundedVec<u8, ConstU32<256>>,
	pub icon_url: BoundedVec<u8, ConstU32<256>>,
	pub version: u16,
	pub block_number: u64,
}

#[derive(Clone, Decode, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct AssetUpdatedL2 {
	pub event_hash: U256,
	pub event_name: U256,
	pub id: u64,
	pub asset: Asset,
	pub metadata_url: BoundedVec<u8, ConstU32<256>>,
	pub icon_url: BoundedVec<u8, ConstU32<256>>,
	pub version: u16,
	pub block_number: u64,
}

#[derive(Clone, Copy, Decode, Default, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct MarketRemovedL2 {
	pub event_hash: U256,
	pub event_name: U256,
	pub id: u64,
	pub block_number: u64,
}

#[derive(Clone, Copy, Decode, Default, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct AssetRemovedL2 {
	pub event_hash: U256,
	pub event_name: U256,
	pub id: u64,
	pub block_number: u64,
}

#[derive(Clone, Copy, Decode, Default, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct UserDepositL2 {
	pub event_hash: U256,
	pub event_name: U256,
	pub trading_account: TradingAccount,
	pub collateral_id: u64,
	pub nonce: U256,
	pub amount: U256,
	pub balance: U256,
	pub block_number: u64,
}

#[derive(Clone, Copy, Decode, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub enum FundType {
	Admin,
	InsuranceFund,
	LiquidityFund,
	HoldingFund,
	EmergencyFund,
}

#[derive(Clone, Copy, Decode, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct FundsTransferL2 {
	pub event_hash: U256,
	pub event_name: U256,
	pub from_fund: FundType,
	pub to_fund: FundType,
	pub asset_id: u64,
	pub amount: U256,
}

#[derive(Clone, Decode, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub enum UniversalEventL2 {
	MarketUpdatedL2(MarketUpdatedL2),
	AssetUpdatedL2(AssetUpdatedL2),
	MarketRemovedL2(MarketRemovedL2),
	AssetRemovedL2(AssetRemovedL2),
	FundsTransferL2(FundsTransferL2),
	UserDepositL2(UserDepositL2),
}

#[derive(Clone, Decode, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct SyncSignature {
	pub signer_index: u8,
	pub r: U256,
	pub s: U256,
}

impl Hashable for Order {
	// No error apart from error during conversion from U256 to FieldElement should happen
	// Hence associated type is defined to be exactly that error i.e. starknet_ff::FromByteSliceError
	type ConversionError = FromByteSliceError;
	fn hash(&self, hash_type: HashType) -> Result<FieldElement, Self::ConversionError> {
		let mut elements: Vec<FieldElement> = Vec::new();

		elements.push(u256_to_field_element(&self.account_id)?);

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
			false => elements.push(FieldElement::from(0_u8)),
		}

		elements.push(FieldElement::from(u8::from(self.time_in_force)));

		match hash_type {
			HashType::Pedersen => Ok(pedersen_hash_multiple(&elements)),
			HashType::Poseidon => Ok(poseidon_hash_many(&elements)),
		}
	}
}
