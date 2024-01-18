#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::dispatch::Vec;
use pallet_support::types::{AccountInfo, FeeRates, MarginInfo, PositionExtended};
use primitive_types::U256;
use sp_arithmetic::fixed_point::FixedI128;

// Here we declare the runtime API. It is implemented it the `impl` block in
// runtime file (the `runtime-api/src/lib.rs`)
sp_api::decl_runtime_apis! {
	pub trait TradingApi {
		fn positions(account_id: U256, collateral_id: u128) -> Vec<PositionExtended>;
		fn get_margin_info(account_id: U256, collateral_id: u128) -> MarginInfo;
		fn get_account_info(account_id: U256, collateral_id: u128) -> AccountInfo;
		fn get_account_list(start_index: u128, end_index: u128) -> Vec<U256>;
		fn get_fee(account_id: U256, market_id: U256) -> (FeeRates, u64);
		fn get_withdrawable_amount(account_id: U256, collateral_id: u128) -> FixedI128;
	}
}
