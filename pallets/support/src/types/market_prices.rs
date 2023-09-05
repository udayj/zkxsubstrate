use codec::{Decode, Encode};
use primitive_types::U256;
use scale_info::TypeInfo;
use sp_arithmetic::fixed_point::FixedI128;
use sp_runtime::RuntimeDebug;

#[derive(Clone, Copy, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub struct MarketPrice {
	pub asset_id: U256,
	pub collateral_id: U256,
	pub timestamp: u64,
	pub price: FixedI128,
}

#[derive(Clone, Copy, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub struct MultipleMarketPrices {
	pub market_id: U256,
	pub price: FixedI128,
}
