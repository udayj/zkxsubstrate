use crate::types::Asset;

pub trait AssetInterface {
	fn get_default_collateral() -> u64;
	fn get_asset(id: u64) -> Option<Asset>;
}
