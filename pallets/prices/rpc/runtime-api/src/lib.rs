#![cfg_attr(not(feature = "std"), no_std)]

use pallet_support::types::ABRState;

// Here we declare the runtime API. It is implemented it the `impl` block in
// runtime file (the `runtime-api/src/lib.rs`)
sp_api::decl_runtime_apis! {
	pub trait PricesApi {
		fn get_abr_state() -> ABRState;
	}
}
