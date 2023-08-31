#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::inherent::Vec;
use primitive_types::U256;
use zkx_support::types::Position;

// Here we declare the runtime API. It is implemented it the `impl` block in
// runtime file (the `runtime-api/src/lib.rs`)
sp_api::decl_runtime_apis! {
	pub trait TradingApi {
		fn positions(account_id: U256) -> Vec<Position>;
	}
}
