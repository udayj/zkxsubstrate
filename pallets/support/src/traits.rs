use crate::types::{
	Asset, AssetRemoved, AssetUpdated, Direction, HashType, LiquidatablePosition, Market,
	MarketRemoved, MarketUpdated, Order, OrderSide, Position, PositionDetailsForRiskManagement,
	Side, SignerAdded, SignerRemoved, TradingAccount, TradingAccountWithoutId, UniversalEvent,
	UserDeposit,
};
use frame_support::inherent::Vec;
use primitive_types::U256;
use sp_arithmetic::fixed_point::FixedI128;
use sp_runtime::traits::ConstU32;
use sp_runtime::BoundedVec;
use starknet_ff::{FieldElement, FromByteSliceError};

pub trait TradingAccountInterface {
	fn deposit(trading_account: TradingAccountWithoutId, collateral_id: u128, amount: FixedI128);
	fn is_registered_user(account: U256) -> bool;
	fn get_balance(account: U256, asset_id: u128) -> FixedI128;
	fn get_unused_balance(account: U256, asset_id: u128) -> FixedI128;
	fn get_locked_margin(account: U256, asset_id: u128) -> FixedI128;
	fn set_locked_margin(account: U256, asset_id: u128, amount: FixedI128);
	fn transfer(account: U256, asset_id: u128, amount: FixedI128);
	fn transfer_from(account: U256, asset_id: u128, amount: FixedI128);
	fn get_account(account_id: &U256) -> Option<TradingAccount>;
	fn get_public_key(account: &U256) -> Option<U256>;
	fn get_margin_info(
		account_id: U256,
		collateral_id: u128,
		new_position_maintanence_requirement: FixedI128,
		new_position_margin: FixedI128,
	) -> (
		bool,
		FixedI128,
		FixedI128,
		FixedI128,
		FixedI128,
		FixedI128,
		PositionDetailsForRiskManagement,
		FixedI128,
	);
}

pub trait TradingInterface {
	fn get_markets_of_collateral(account_id: U256, collateral_id: u128) -> Vec<u128>;
	fn get_position(account_id: U256, market_id: u128, direction: Direction) -> Position;
	fn get_positions(account_id: U256, collateral_id: u128) -> Vec<Position>;
	fn liquidate_position(
		account_id: U256,
		collateral_id: u128,
		position: &PositionDetailsForRiskManagement,
		amount_to_be_sold: FixedI128,
	);
	fn get_deleveragable_or_liquidatable_position(
		account_id: U256,
		collateral_id: u128,
	) -> LiquidatablePosition;
}

pub trait AssetInterface {
	fn get_default_collateral() -> u128;
	fn get_asset(id: u128) -> Option<Asset>;
}

pub trait RiskManagementInterface {
	fn check_for_risk(
		order: &Order,
		size: FixedI128,
		execution_price: FixedI128,
		oracle_price: FixedI128,
		margin_amount: FixedI128,
	) -> (FixedI128, bool);
}

pub trait MarketInterface {
	fn get_market(id: u128) -> Option<Market>;
}

pub trait MarketPricesInterface {
	fn get_market_price(market_id: u128) -> FixedI128;
	fn update_market_price(market_id: u128, price: FixedI128);
}

pub trait FixedI128Ext {
	fn round_to_precision(&self, precision: u32) -> Self;
	fn to_u256(&self) -> U256;
}

pub trait StringExt {
	fn to_felt_rep(&self) -> u128;
}

pub trait U256Ext {
	fn try_to_felt(&self) -> Result<FieldElement, FromByteSliceError>;
}

pub trait FieldElementExt {
	fn to_u256(&self) -> U256;
}

pub trait TradingFeesInterface {
	fn get_fee_rate(
		side: Side,
		order_side: OrderSide,
		number_of_tokens: U256,
	) -> (FixedI128, u8, u8);
}

// This trait needs to be implemented by every type that can be hashed (pedersen or poseidon) and returns a FieldElement
pub trait Hashable {
	type ConversionError;
	fn hash(&self, hash_type: &HashType) -> Result<FieldElement, Self::ConversionError>;
}

pub trait FeltSerializedArrayExt {
	fn append_bounded_vec(&mut self, vec: &BoundedVec<u8, ConstU32<256>>);
	fn append_bool(&mut self, boolean_value: bool);
	fn try_append_u256(&mut self, u256_value: U256) -> Result<(), FromByteSliceError>;
	fn try_append_u256_pair(&mut self, u256_value: U256) -> Result<(), FromByteSliceError>;
	fn try_append_fixedi128(&mut self, fixed_value: FixedI128) -> Result<(), FromByteSliceError>;
	fn try_append_asset(&mut self, asset: &Asset) -> Result<(), FromByteSliceError>;
	fn try_append_market(&mut self, market: &Market) -> Result<(), FromByteSliceError>;
	fn try_append_trading_account(
		&mut self,
		trading_account: &TradingAccountWithoutId,
	) -> Result<(), FromByteSliceError>;
	fn try_append_market_updated_event(
		&mut self,
		market_updated_event: &MarketUpdated,
	) -> Result<(), FromByteSliceError>;
	fn try_append_asset_updated_event(
		&mut self,
		asset_updated_event: &AssetUpdated,
	) -> Result<(), FromByteSliceError>;
	fn try_append_market_removed_event(
		&mut self,
		market_removed_event: &MarketRemoved,
	) -> Result<(), FromByteSliceError>;
	fn try_append_asset_removed_event(
		&mut self,
		asset_removed_event: &AssetRemoved,
	) -> Result<(), FromByteSliceError>;
	fn try_append_user_deposit_event(
		&mut self,
		user_deposit_event: &UserDeposit,
	) -> Result<(), FromByteSliceError>;
	fn try_append_signer_added_event(
		&mut self,
		signer_added: &SignerAdded,
	) -> Result<(), FromByteSliceError>;
	fn try_append_signer_removed_event(
		&mut self,
		signer_added: &SignerRemoved,
	) -> Result<(), FromByteSliceError>;
	fn try_append_universal_event_array(
		&mut self,
		universal_event_array: &Vec<UniversalEvent>,
	) -> Result<(), FromByteSliceError>;
}
