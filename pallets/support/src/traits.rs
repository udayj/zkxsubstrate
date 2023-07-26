use crate::types::{Asset, Market, TradingAccount};
use sp_arithmetic::fixed_point::FixedI128;

pub trait TradingAccountInterface {
	fn is_registered_user(account: TradingAccount) -> bool;
	fn get_balance(account: TradingAccount, asset_id: u64) -> FixedI128;
	fn get_locked_margin(account: TradingAccount, asset_id: u64) -> FixedI128;
	fn set_locked_margin(account: TradingAccount, asset_id: u64, amount: FixedI128);
	fn transfer(account: TradingAccount, asset_id: u64, amount: FixedI128);
	fn transfer_from(account: TradingAccount, asset_id: u64, amount: FixedI128);
}

pub trait AssetInterface {
	fn get_default_collateral() -> u64;
	fn get_asset(id: u64) -> Option<Asset>;
}

pub trait MarketInterface {
	fn get_market(id: u64) -> Option<Market>;
}

pub trait FixedI128Ext<T> {
	fn round_to_precision(t: T, precision: u32) -> T;
}
