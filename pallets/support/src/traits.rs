use crate::types::{Asset, ExecutedBatch, ExecutedOrder, FailedOrder, Market, OrderSide, Side};
use frame_support::inherent::Vec;
use primitive_types::U256;
use sp_arithmetic::fixed_point::FixedI128;

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

pub trait SyncFacadeInterface {
	fn syncfacade_emit(
		executed_orders: Vec<ExecutedOrder>,
		failed_orders: Vec<FailedOrder>,
		executed_batch: ExecutedBatch,
	);
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
