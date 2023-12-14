use codec::{Decode, Encode};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_arithmetic::fixed_point::FixedI128;
use sp_runtime::RuntimeDebug;

#[derive(Clone, Copy, Decode, Default, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct BaseFee {
	pub volume: FixedI128,
	pub maker_fee: FixedI128,
	pub taker_fee: FixedI128,
}

#[derive(
	Clone, Copy, Decode, Default, Deserialize, Encode, PartialEq, RuntimeDebug, Serialize, TypeInfo,
)]
pub struct FeeRates {
	pub maker_buy: FixedI128,
	pub maker_sell: FixedI128,
	pub taker_buy: FixedI128,
	pub taker_sell: FixedI128,
}

impl FeeRates {
	pub fn new(
		maker_buy: FixedI128,
		maker_sell: FixedI128,
		taker_buy: FixedI128,
		taker_sell: FixedI128,
	) -> FeeRates {
		FeeRates { maker_buy, maker_sell, taker_buy, taker_sell }
	}
}
