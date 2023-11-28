use codec::{Decode, Encode};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_arithmetic::fixed_point::FixedI128;
use sp_runtime::RuntimeDebug;

#[derive(
	Clone, Copy, Encode, Decode, Default, Deserialize, PartialEq, RuntimeDebug, Serialize, TypeInfo,
)]
pub enum ABRState {
	#[default]
	State0,
	State1,
	State2,
}

#[derive(
	Clone, Copy, Encode, Decode, Default, Deserialize, PartialEq, RuntimeDebug, Serialize, TypeInfo,
)]
pub struct ABRDetails {
	pub abr_value: FixedI128,
	pub abr_timestamp: u64,
}
