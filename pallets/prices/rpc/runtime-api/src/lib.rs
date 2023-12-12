#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::dispatch::Vec;
use pallet_support::types::ABRDetails;
// Here we declare the runtime API. It is implemented it the `impl` block in
// runtime file (the `runtime-api/src/lib.rs`)
sp_api::decl_runtime_apis! {
	pub trait PricesApi {
		fn get_remaining_markets() -> Vec<u128>;
		fn get_no_of_batches_for_current_epoch() -> u128;
		fn get_last_abr_timestamp() -> u64;
		fn get_remaining_pay_abr_calls() -> u128;
		fn get_next_abr_timestamp() -> u64;
		fn get_previous_abr_values(starting_epoch: u64, market_id: u128, n: u64) -> Vec<ABRDetails>;
	}
}
