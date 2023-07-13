#![cfg_attr(not(feature = "std"), no_std)]

use sp_arithmetic::{fixed_point::FixedI128, traits::CheckedDiv, FixedPointNumber};
use starknet_ff::FieldElement;

pub mod types;

pub mod traits;

pub fn str_to_felt(text: &str) -> u64 {
	let a = FieldElement::from_byte_slice_be(text.as_bytes());
	u64::try_from(a.unwrap()).unwrap()
}

pub fn approximate(a: FixedI128, precision: u32) -> FixedI128 {
	let inner_value: i128 = FixedI128::into_inner(a);
	let divisor: i128 = 10_u64.pow(18).into();
	let integer_part: i128 = inner_value / divisor;
	let decimal_part: i128 = inner_value % divisor;
	let ten_power_precision: i128 = 10_u64.pow(precision).into();
	let decimal_part: FixedI128 = FixedI128::from_inner(decimal_part * ten_power_precision);
	let frac_rounded: FixedI128 = decimal_part.round();

	let ten_power_precision: FixedI128 =
		FixedI128::checked_from_integer(ten_power_precision).unwrap();
	let decimal_part: FixedI128 = frac_rounded.checked_div(&ten_power_precision).unwrap();
	let integer_part: FixedI128 = FixedPointNumber::checked_from_integer(integer_part).unwrap();
	let res: FixedI128 = FixedI128::add(integer_part, decimal_part);
	res
}
