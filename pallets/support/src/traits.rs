use crate::types::Asset;
use sp_arithmetic::{
	fixed_point::FixedI128,
	traits::{CheckedDiv, CheckedMul},
	FixedPointNumber,
};

pub trait AssetInterface {
	fn get_default_collateral() -> u64;
	fn get_asset(id: u64) -> Option<Asset>;
}

pub fn approximate(a: FixedI128, precision: u32) -> FixedI128 {
	let x: FixedI128 = FixedI128::checked_from_integer(10_u64.pow(precision)).unwrap();
	let temp: FixedI128 = a.checked_mul(&x).unwrap().round();
	temp.checked_div(&x).unwrap()
}
