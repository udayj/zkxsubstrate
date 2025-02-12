use crate::{
	helpers::compute_hash_on_elements,
	traits::{FixedI128Ext, Hashable, U256Ext},
	types::common::{convert_to_u128_pair, HashType},
};
use codec::{Decode, Encode};
use frame_support::dispatch::Vec;
use primitive_types::U256;
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_arithmetic::fixed_point::FixedI128;
use sp_runtime::RuntimeDebug;
use starknet_crypto::poseidon_hash_many;
use starknet_ff::FieldElement;

#[derive(Clone, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub struct SignatureInfo {
	pub liquidator_pub_key: U256,
	pub hash_type: HashType,
	pub sig_r: U256,
	pub sig_s: U256,
}

// Order related
#[derive(Clone, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Order {
	pub account_id: U256,
	pub order_id: U256,
	pub market_id: u128,
	pub order_type: OrderType,
	pub direction: Direction,
	pub side: Side,
	pub price: FixedI128,
	pub size: FixedI128,
	pub leverage: FixedI128,
	pub slippage: FixedI128,
	pub post_only: bool,
	pub time_in_force: TimeInForce,
	pub signature_info: SignatureInfo,
	pub timestamp: u64,
}

#[derive(Clone, Copy, Decode, Default, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub enum FundModifyType {
	#[default]
	Increase,
	Decrease,
}

#[derive(Clone, Copy, Decode, Default, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub enum OrderSide {
	#[default]
	Maker,
	Taker,
}

#[derive(
	Clone, Copy, Encode, Decode, Default, Deserialize, PartialEq, RuntimeDebug, Serialize, TypeInfo,
)]
pub enum Side {
	#[default]
	Buy,
	Sell,
}

#[derive(
	Clone, Copy, Encode, Decode, Default, Deserialize, PartialEq, RuntimeDebug, Serialize, TypeInfo,
)]
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
	Forced,
	ADS,
}

#[derive(Clone, Copy, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub enum TimeInForce {
	#[default]
	GTC,
	IOC,
	FOK,
}

#[derive(Clone, Copy, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub enum BalanceChangeReason {
	Deposit,
	#[default]
	Fee,
	Liquidation,
	PnlRealization,
	Withdrawal,
	WithdrawalFee,
	ABR,
}

#[derive(
	Clone, Copy, Encode, Decode, Default, Deserialize, PartialEq, RuntimeDebug, Serialize, TypeInfo,
)]
pub enum ForceClosureFlag {
	#[default]
	Deleverage,
	Liquidate,
}

// Position Related
#[derive(
	Clone, Encode, Decode, Default, Deserialize, PartialEq, RuntimeDebug, Serialize, TypeInfo,
)]
pub struct Position {
	pub market_id: u128,
	pub direction: Direction,
	pub avg_execution_price: FixedI128,
	pub size: FixedI128,
	pub margin_amount: FixedI128,
	pub borrowed_amount: FixedI128,
	pub leverage: FixedI128,
	pub created_timestamp: u64,
	pub modified_timestamp: u64,
	pub realized_pnl: FixedI128,
}

#[derive(
	Clone, Encode, Decode, Default, Deserialize, PartialEq, RuntimeDebug, Serialize, TypeInfo,
)]
pub struct PositionExtended {
	// market_id is U256 instead of u128 here, so that ZKXNode
	// can handle it correctly
	pub market_id: U256,
	pub direction: Direction,
	pub avg_execution_price: FixedI128,
	pub size: FixedI128,
	pub margin_amount: FixedI128,
	pub borrowed_amount: FixedI128,
	pub leverage: FixedI128,
	pub created_timestamp: u64,
	pub modified_timestamp: u64,
	pub realized_pnl: FixedI128,
	pub maintenance_margin: FixedI128,
	pub mark_price: FixedI128,
}

#[derive(
	Clone, Encode, Decode, Default, Deserialize, PartialEq, RuntimeDebug, Serialize, TypeInfo,
)]
pub struct PositionDetailsForRiskManagement {
	pub market_id: u128,
	pub direction: Direction,
	pub avg_execution_price: FixedI128,
	pub size: FixedI128,
	pub margin_amount: FixedI128,
	pub borrowed_amount: FixedI128,
	pub leverage: FixedI128,
}

// Batch Related
#[derive(Clone, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub struct ExecutedBatch {
	batch_id: U256,
	market_id: u128,
	size: FixedI128,
	execution_price: FixedI128,
	direction: Direction,
	side: Side,
}

#[derive(
	Clone, Encode, Decode, Default, Deserialize, PartialEq, RuntimeDebug, Serialize, TypeInfo,
)]
pub struct MarginInfo {
	pub is_liquidation: bool,
	pub total_margin: FixedI128,
	pub available_margin: FixedI128,
	pub unrealized_pnl_sum: FixedI128,
	pub maintenance_margin_requirement: FixedI128,
}

#[derive(
	Clone, Encode, Decode, Default, Deserialize, PartialEq, RuntimeDebug, Serialize, TypeInfo,
)]
pub struct AccountInfo {
	pub positions: Vec<PositionExtended>,
	pub available_margin: FixedI128,
	pub total_margin: FixedI128,
	pub collateral_balance: FixedI128,
	pub force_closure_flag: Option<ForceClosureFlag>,
	pub unused_balance: FixedI128,
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
			OrderType::Forced => 2_u8,
			OrderType::ADS => 3_u8,
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

impl From<Direction> for &str {
	fn from(value: Direction) -> &'static str {
		match value {
			Direction::Long => "LONG",
			Direction::Short => "SHORT",
		}
	}
}

impl From<Side> for &str {
	fn from(value: Side) -> &'static str {
		match value {
			Side::Buy => "BUY",
			Side::Sell => "SELL",
		}
	}
}

impl From<OrderType> for &str {
	fn from(value: OrderType) -> &'static str {
		match value {
			OrderType::Market => "MARKET",
			OrderType::Limit => "LIMIT",
			OrderType::Forced => "FORCED",
			OrderType::ADS => "ADS",
		}
	}
}

impl From<TimeInForce> for &str {
	fn from(value: TimeInForce) -> &'static str {
		match value {
			TimeInForce::GTC => "GTC",
			TimeInForce::FOK => "FOK",
			TimeInForce::IOC => "IOC",
		}
	}
}
impl From<BalanceChangeReason> for u8 {
	fn from(value: BalanceChangeReason) -> u8 {
		match value {
			BalanceChangeReason::Deposit => 0_u8,
			BalanceChangeReason::Fee => 2_u8,
			BalanceChangeReason::Liquidation => 3_u8,
			BalanceChangeReason::PnlRealization => 4_u8,
			BalanceChangeReason::Withdrawal => 5_u8,
			BalanceChangeReason::WithdrawalFee => 6_u8,
			BalanceChangeReason::ABR => 7_u8,
		}
	}
}

impl From<FundModifyType> for u8 {
	fn from(value: FundModifyType) -> u8 {
		match value {
			FundModifyType::Increase => 0_u8,
			FundModifyType::Decrease => 1_u8,
		}
	}
}

impl From<ForceClosureFlag> for u8 {
	fn from(value: ForceClosureFlag) -> u8 {
		match value {
			ForceClosureFlag::Deleverage => 0_u8,
			ForceClosureFlag::Liquidate => 1_u8,
		}
	}
}

impl PositionExtended {
	pub fn new(
		position: Position,
		maintenance_margin: FixedI128,
		mark_price: FixedI128,
	) -> PositionExtended {
		PositionExtended {
			market_id: position.market_id.into(),
			direction: position.direction,
			avg_execution_price: position.avg_execution_price,
			size: position.size,
			margin_amount: position.margin_amount,
			borrowed_amount: position.borrowed_amount,
			leverage: position.leverage,
			created_timestamp: position.created_timestamp,
			modified_timestamp: position.modified_timestamp,
			realized_pnl: position.realized_pnl,
			maintenance_margin,
			mark_price,
		}
	}
}

mod general_conversion_error {
	#[derive(Debug)]
	pub enum GeneralConversionError {
		U256ToFieldElementError,
		EnumToFieldElementError,
	}
}

pub use general_conversion_error::GeneralConversionError;

impl Hashable for Order {
	type ConversionError = GeneralConversionError;

	fn hash(&self, hash_type: &HashType) -> Result<FieldElement, Self::ConversionError> {
		let mut elements: Vec<FieldElement> = Vec::new();

		let (account_id_low, account_id_high) = convert_to_u128_pair(self.account_id)
			.map_err(|_err| GeneralConversionError::U256ToFieldElementError)?;
		elements.push(account_id_low);
		elements.push(account_id_high);

		let (order_id_low, order_id_high) = convert_to_u128_pair(self.order_id)
			.map_err(|_| GeneralConversionError::U256ToFieldElementError)?;
		elements.push(order_id_low);
		elements.push(order_id_high);

		elements.push(FieldElement::from(self.market_id));

		let order_type: &str = self.order_type.into();
		elements.push(
			FieldElement::from_hex_be(hex::encode(order_type).as_str())
				.map_err(|_err| GeneralConversionError::EnumToFieldElementError)?,
		);

		let direction: &str = self.direction.into();
		elements.push(
			FieldElement::from_hex_be(hex::encode(direction).as_str())
				.map_err(|_err| GeneralConversionError::EnumToFieldElementError)?,
		);

		let side: &str = self.side.into();
		elements.push(
			FieldElement::from_hex_be(hex::encode(side).as_str())
				.map_err(|_err| GeneralConversionError::EnumToFieldElementError)?,
		);

		let u256_representation = &self.price.to_u256();
		elements.push(
			u256_representation
				.try_to_felt()
				.map_err(|_err| GeneralConversionError::U256ToFieldElementError)?,
		);

		let u256_representation = &self.size.to_u256();
		elements.push(
			u256_representation
				.try_to_felt()
				.map_err(|_err| GeneralConversionError::U256ToFieldElementError)?,
		);

		let u256_representation = &self.leverage.to_u256();
		elements.push(
			u256_representation
				.try_to_felt()
				.map_err(|_err| GeneralConversionError::U256ToFieldElementError)?,
		);

		let u256_representation = &self.slippage.to_u256();
		elements.push(
			u256_representation
				.try_to_felt()
				.map_err(|_err| GeneralConversionError::U256ToFieldElementError)?,
		);

		match self.post_only {
			true => elements.push(FieldElement::from(1_u8)),
			false => elements.push(FieldElement::from(0_u8)),
		}

		let time_in_force: &str = self.time_in_force.into();
		elements.push(
			FieldElement::from_hex_be(hex::encode(time_in_force).as_str())
				.map_err(|_err| GeneralConversionError::EnumToFieldElementError)?,
		);

		elements.push(FieldElement::from(self.timestamp));

		match &hash_type {
			HashType::Pedersen => Ok(compute_hash_on_elements(&elements)),
			HashType::Poseidon => Ok(poseidon_hash_many(&elements)),
		}
	}
}
