use crate::types::Asset;

pub trait AssetInterface {
	fn get_default_collateral() -> u64;
	fn get_asset(id: u64) -> Option<Asset>;
}

pub trait FixedI128Ext<T> {
	fn round_to_precision(t: T, precision: u32) -> T;
}
