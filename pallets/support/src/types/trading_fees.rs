use codec::{Decode, Encode};
use primitive_types::U256;
use scale_info::TypeInfo;
use sp_arithmetic::fixed_point::FixedI128;
use sp_runtime::RuntimeDebug;

#[derive(Clone, Copy, Decode, Default, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct BaseFee {
	pub volume: U256,
	pub maker_fee: FixedI128,
	pub taker_fee: FixedI128,
}
