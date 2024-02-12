#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::dispatch::Vec;
use pallet_support::types::ABRDetails;
use primitive_types::U256;
use sp_arithmetic::fixed_point::FixedI128;

// Here we declare the runtime API. It is implemented it the `impl` block in
// runtime file (the `runtime-api/src/lib.rs`)
sp_api::decl_runtime_apis! {
	pub trait PricesApi {
		fn get_remaining_markets() -> Vec<U256>;
		fn get_no_of_batches_for_current_epoch() -> u64;
		fn get_last_abr_timestamp() -> u64;
		fn get_remaining_pay_abr_calls() -> u64;
		fn get_next_abr_timestamp() -> u64;
		fn get_previous_abr_values(market_id: U256, start_timestamp: u64, end_timestamp: u64) -> Vec<ABRDetails>;
		fn get_intermediary_abr_value(market_id: U256) -> FixedI128;
		fn get_remaining_prices_cleanup_calls() -> u64;
	}
}
