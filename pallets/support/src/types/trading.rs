use crate::helpers::pedersen_hash_multiple;
use crate::traits::{FixedI128Ext, Hashable, U256Ext};
use crate::types::common::{HashType,convert_to_u128_pair};
use codec::{Decode, Encode};
use frame_support::inherent::Vec;
use primitive_types::U256;
use scale_info::TypeInfo;
use sp_arithmetic::fixed_point::FixedI128;
use sp_runtime::RuntimeDebug;
use starknet_crypto::poseidon_hash_many;
use starknet_ff::{FieldElement, FromByteSliceError};

// Order related
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
	pub sig_r: U256,
	pub sig_s: U256,
	pub hash_type: HashType,
}

#[derive(Clone, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub struct FailedOrder {
	pub order_id: u128,
	pub error_code: u16,
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
pub enum FundModifyType {
	#[default]
	Increase,
	Decrease,
}

#[derive(Clone, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub struct AbnormalCloseOrder {
	pub modify_type: FundModifyType,
	pub collateral_id: U256,
	pub amount: FixedI128,
}

#[derive(Clone, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub struct UserBalanceChange {
	pub account_id: U256,
	pub collateral_id: U256,
	pub amount: FixedI128,
	pub modify_type: FundModifyType,
	pub reason: u8,
}

#[derive(Clone, Decode, Default, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub enum OrderSide {
	#[default]
	Maker,
	Taker,
}

#[derive(Clone, Copy, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub enum Side {
	#[default]
	Buy,
	Sell,
}

#[derive(Clone, Copy, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub enum Direction {
	#[default]
	Long,
	Short,
}

#[derive(Clone, Copy, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub enum OrderType {
	#[default]
	Limit,
	Market,
	Liquidation,
	Deleveraging,
}

#[derive(Clone, Copy, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub enum TimeInForce {
	#[default]
	GTC,
	IOC,
	FOK,
}

// Position Related
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
pub struct PositionDetailsForRiskManagement {
	pub market_id: U256,
	pub direction: Direction,
	pub avg_execution_price: FixedI128,
	pub size: FixedI128,
	pub margin_amount: FixedI128,
	pub borrowed_amount: FixedI128,
	pub leverage: FixedI128,
}

#[derive(Clone, Copy, Decode, Default, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct LiquidatablePosition {
	pub market_id: U256,
	pub direction: Direction,
	pub amount_to_be_sold: FixedI128,
	pub liquidatable: bool,
}

// Batch Related
#[derive(Clone, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub struct ExecutedBatch {
	batch_id: U256,
	market_id: U256,
	size: FixedI128,
	execution_price: FixedI128,
	direction: Direction,
	side: Side,
}

// Impls
impl From<Direction> for u8 {
	fn from(value: Direction) -> u8 {
		match value {
			Direction::Long => 0_u8,
			Direction::Short => 1_u8,
		}
	}
}

impl From<Side> for u8 {
	fn from(value: Side) -> u8 {
		match value {
			Side::Buy => 0_u8,
			Side::Sell => 1_u8,
		}
	}
}

impl From<OrderType> for u8 {
	fn from(value: OrderType) -> u8 {
		match value {
			OrderType::Limit => 0_u8,
			OrderType::Market => 1_u8,
			OrderType::Liquidation => 2_u8,
			OrderType::Deleveraging => 3_u8,
		}
	}
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

impl AbnormalCloseOrder {
	pub fn new(
		modify_type: FundModifyType,
		collateral_id: U256,
		amount: FixedI128,
	) -> AbnormalCloseOrder {
		AbnormalCloseOrder { modify_type, collateral_id, amount }
	}
}

impl UserBalanceChange {
	pub fn new(
		account_id: U256,
		collateral_id: U256,
		amount: FixedI128,
		modify_type: FundModifyType,
		reason: u8,
	) -> UserBalanceChange {
		UserBalanceChange { account_id, collateral_id, amount, modify_type, reason }
	}
}

impl Hashable for Order {
	// No error apart from error during conversion from U256 to FieldElement should happen
	// Hence associated type is defined to be exactly that error i.e. starknet_ff::FromByteSliceError
	type ConversionError = FromByteSliceError;

	fn hash(&self, hash_type: &HashType) -> Result<FieldElement, Self::ConversionError> {
		let mut elements: Vec<FieldElement> = Vec::new();

		let (account_id_low, account_id_high) = convert_to_u128_pair(self.account_id)?;
		elements.push(account_id_low);
		elements.push(account_id_high);

		elements.push(FieldElement::from(self.order_id));

		elements.push(self.market_id.try_to_felt()?);

		elements.push(FieldElement::from(u8::from(self.order_type)));
		elements.push(FieldElement::from(u8::from(self.direction)));
		elements.push(FieldElement::from(u8::from(self.side)));

		let u256_representation = &self.price.to_u256();
		elements.push(u256_representation.try_to_felt()?);

		let u256_representation = &self.size.to_u256();
		elements.push(u256_representation.try_to_felt()?);

		let u256_representation = &self.leverage.to_u256();
		elements.push(u256_representation.try_to_felt()?);

		let u256_representation = &self.slippage.to_u256();
		elements.push(u256_representation.try_to_felt()?);

		match self.post_only {
			true => elements.push(FieldElement::from(1_u8)),
			false => elements.push(FieldElement::from(0_u8)),
		}

		elements.push(FieldElement::from(u8::from(self.time_in_force)));

		match &hash_type {
			HashType::Pedersen => Ok(pedersen_hash_multiple(&elements)),
			HashType::Poseidon => Ok(poseidon_hash_many(&elements)),
		}
	}
}
