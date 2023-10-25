use crate::helpers::pedersen_hash_multiple;
use crate::traits::{FixedI128Ext, Hashable, U256Ext};
use crate::types::common::{convert_to_u128_pair, HashType};
use codec::{Decode, Encode};
use frame_support::inherent::Vec;
use primitive_types::U256;
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_arithmetic::fixed_point::FixedI128;
use sp_runtime::RuntimeDebug;
use starknet_crypto::poseidon_hash_many;
use starknet_ff::{FieldElement, FromByteSliceError};

// Order related
#[derive(Clone, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Order {
	pub account_id: U256,
	pub order_id: u128,
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
	pub sig_r: U256,
	pub sig_s: U256,
	pub hash_type: HashType,
}

#[derive(Clone, Copy, Decode, Default, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub enum FundModifyType {
	#[default]
	Increase,
	Decrease,
}

#[derive(Clone, Decode, Default, Encode, PartialEq, RuntimeDebug, TypeInfo)]
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
	#[default]
	Fee,
	Deposit,
	Liquidation,
	PnlRealization,
	Withdrawal,
	WithdrawalFee,
}

#[derive(
	Clone, Copy, Encode, Decode, Default, Deserialize, PartialEq, RuntimeDebug, Serialize, TypeInfo,
)]
pub enum ForceClosureFlag {
	#[default]
	Absent,
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
	pub side: Side,
	pub avg_execution_price: FixedI128,
	pub size: FixedI128,
	pub margin_amount: FixedI128,
	pub borrowed_amount: FixedI128,
	pub leverage: FixedI128,
	pub realized_pnl: FixedI128,
}

#[derive(
	Clone, Encode, Decode, Default, Deserialize, PartialEq, RuntimeDebug, Serialize, TypeInfo,
)]
pub struct PositionExtended {
	pub market_id: u128,
	pub direction: Direction,
	pub side: Side,
	pub avg_execution_price: FixedI128,
	pub size: FixedI128,
	pub margin_amount: FixedI128,
	pub borrowed_amount: FixedI128,
	pub leverage: FixedI128,
	pub realized_pnl: FixedI128,
	pub maintenance_margin: FixedI128,
	pub market_price: FixedI128,
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

#[derive(
	Clone, Copy, Decode, Default, Deserialize, Encode, PartialEq, RuntimeDebug, Serialize, TypeInfo,
)]
pub struct DeleveragablePosition {
	pub market_id: u128,
	pub direction: Direction,
	pub amount_to_be_sold: FixedI128,
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
	pub least_collateral_ratio: FixedI128,
	pub least_collateral_ratio_position: PositionDetailsForRiskManagement,
	pub least_collateral_ratio_position_asset_price: FixedI128,
}

#[derive(
	Clone, Encode, Decode, Default, Deserialize, PartialEq, RuntimeDebug, Serialize, TypeInfo,
)]
pub struct AccountInfo {
	pub positions: Vec<PositionExtended>,
	pub available_margin: FixedI128,
	pub total_margin: FixedI128,
	pub collateral_balance: FixedI128,
	pub force_closure_flag: ForceClosureFlag,
	pub deleveragable_position: DeleveragablePosition,
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

impl From<BalanceChangeReason> for u8 {
	fn from(value: BalanceChangeReason) -> u8 {
		match value {
			BalanceChangeReason::Deposit => 0_u8,
			BalanceChangeReason::Fee => 1_u8,
			BalanceChangeReason::Liquidation => 2_u8,
			BalanceChangeReason::PnlRealization => 3_u8,
			BalanceChangeReason::Withdrawal => 4_u8,
			BalanceChangeReason::WithdrawalFee => 5_u8,
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
			ForceClosureFlag::Absent => 0_u8,
			ForceClosureFlag::Deleverage => 1_u8,
			ForceClosureFlag::Liquidate => 2_u8,
		}
	}
}

impl PositionExtended {
	pub fn new(
		position: Position,
		maintenance_margin: FixedI128,
		market_price: FixedI128,
	) -> PositionExtended {
		PositionExtended {
			market_id: position.market_id,
			direction: position.direction,
			side: position.side,
			avg_execution_price: position.avg_execution_price,
			size: position.size,
			margin_amount: position.margin_amount,
			borrowed_amount: position.borrowed_amount,
			leverage: position.leverage,
			realized_pnl: position.realized_pnl,
			maintenance_margin,
			market_price,
		}
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

		elements.push(FieldElement::from(self.market_id));

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
