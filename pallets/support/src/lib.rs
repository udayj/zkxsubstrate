#![cfg_attr(not(feature = "std"), no_std)]

use crate::traits::FixedI128Ext;
use sp_arithmetic::{fixed_point::FixedI128, traits::CheckedDiv, FixedPointNumber};
use starknet_ff::FieldElement;

pub mod types;

pub mod traits;

pub fn str_to_felt(text: &str) -> u128 {
	let a = FieldElement::from_byte_slice_be(text.as_bytes());
	u128::try_from(a.unwrap()).unwrap()
}

impl FixedI128Ext<FixedI128> for FixedI128 {
	fn round_to_precision(t: FixedI128, precision: u32) -> FixedI128 {
		// Get the inner value (number * 10^18) from FixedI128 representation
		let inner_value: i128 = FixedI128::into_inner(t);
		// Get the integer part and decimal part separately
		let divisor: i128 = 10_u64.pow(18).into();
		let integer_part: i128 = inner_value / divisor;
		let decimal_part: i128 = inner_value % divisor;
		// Multiply decimal part with (10 ^ precision) and round it
		// so that now we have the required precision
		let ten_power_precision: i128 = 10_u64.pow(precision).into();
		let decimal_part: FixedI128 = FixedI128::from_inner(decimal_part * ten_power_precision);
		let decimal_part_rounded: FixedI128 = decimal_part.round();

		// Divide the decimal part with (10 ^ precision)
		// so that we get it to required decimal representaion
		let ten_power_precision: FixedI128 =
			FixedI128::checked_from_integer(ten_power_precision).unwrap();
		let decimal_part: FixedI128 =
			decimal_part_rounded.checked_div(&ten_power_precision).unwrap();
		let integer_part: FixedI128 = FixedPointNumber::checked_from_integer(integer_part).unwrap();
		// Add both the parts together to get the final result
		let res: FixedI128 = FixedI128::add(integer_part, decimal_part);
		res
	}
}
