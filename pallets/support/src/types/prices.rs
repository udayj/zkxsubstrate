use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_arithmetic::fixed_point::FixedI128;
use sp_runtime::RuntimeDebug;

#[derive(Clone, Copy, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Price {
	pub asset_id: u128,
	pub collateral_id: u128,
	pub timestamp: u64,
	pub price: FixedI128,
}

#[derive(Clone, Copy, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub struct MultiplePrices {
	pub market_id: u128,
	pub price: FixedI128,
}
