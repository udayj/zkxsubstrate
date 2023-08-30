use crate::types::{Asset, Market, OrderSide, Side, HashType};
use primitive_types::U256;
use sp_arithmetic::fixed_point::FixedI128;
use starknet_ff::FieldElement;
use starknet_ff::FromByteSliceError;

pub trait TradingAccountInterface {
	fn is_registered_user(account: U256) -> bool;
	fn get_balance(account: U256, asset_id: U256) -> FixedI128;
	fn get_locked_margin(account: U256, asset_id: U256) -> FixedI128;
	fn set_locked_margin(account: U256, asset_id: U256, amount: FixedI128);
	fn transfer(account: U256, asset_id: U256, amount: FixedI128);
	fn transfer_from(account: U256, asset_id: U256, amount: FixedI128);
}

pub trait AssetInterface {
	fn get_default_collateral() -> U256;
	fn get_asset(id: U256) -> Option<Asset>;
}

pub trait MarketInterface {
	fn get_market(id: U256) -> Option<Market>;
}

pub trait MarketPricesInterface {
	fn get_market_price(market_id: U256) -> FixedI128;
	fn update_market_price(market_id: U256, price: FixedI128);
}

pub trait FixedI128Ext<T> {
	fn round_to_precision(t: T, precision: u32) -> T;
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
	fn hash(&self, hash_type:HashType) -> Result<FieldElement, Self::ConversionError>;
}