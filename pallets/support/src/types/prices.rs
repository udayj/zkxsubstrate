use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_arithmetic::fixed_point::FixedI128;
use sp_runtime::RuntimeDebug;

#[derive(Clone, Copy, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub struct CurrentPrice {
	pub timestamp: u64,
	pub index_price: FixedI128,
	pub mark_price: FixedI128,
}

#[derive(Clone, Copy, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub struct HistoricalPrice {
	pub index_price: FixedI128,
	pub mark_price: FixedI128,
}

#[derive(Clone, Copy, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub struct LastTradedPrice {
	pub timestamp: u64,
	pub price: FixedI128,
}

#[derive(Clone, Copy, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub struct MultiplePrices {
	pub market_id: u128,
	pub index_price: FixedI128,
	pub mark_price: FixedI128,
}
