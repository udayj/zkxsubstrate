use crate::traits::{FieldElementExt, FixedI128Ext, StringExt, U256Ext};
use crate::FieldElement;
use codec::{Decode, Encode};
use primitive_types::U256;
use scale_info::TypeInfo;
use sp_arithmetic::{fixed_point::FixedI128, traits::CheckedDiv, FixedPointNumber};
use sp_runtime::RuntimeDebug;
use starknet_ff::FromByteSliceError;

pub fn convert_to_u128_pair(
	u256_value: U256,
) -> Result<(FieldElement, FieldElement), FromByteSliceError> {
	let mut buffer: [u8; 32] = [0; 32];
	u256_value.to_big_endian(&mut buffer);

	let low_bytes = &buffer[16..];
	let high_bytes = &buffer[..16];

	let low_bytes_felt: FieldElement = FieldElement::from_byte_slice_be(&low_bytes)?;
	let high_bytes_felt = FieldElement::from_byte_slice_be(&high_bytes)?;

	Ok((low_bytes_felt, high_bytes_felt))
}

#[derive(Clone, Decode, Default, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub enum HashType {
	#[default]
	Pedersen,
	Poseidon,
}

impl FixedI128Ext for FixedI128 {
	fn round_to_precision(&self, precision: u32) -> FixedI128 {
		// Get the inner value (number * 10^18) from FixedI128 representation
		let inner_value: i128 = FixedI128::into_inner(*self);
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

	// Function to convert from fixed point number to U256 inside the PRIME field
	// This function does the appropriate mod arithmetic to ensure the returned value is actually less than PRIME
	fn to_u256(&self) -> U256 {
		let inner_val: U256;
		// Max prime 2^251 + 17*2^192 + 1
		let prime: U256 = U256::from_dec_str(
			"3618502788666131213697322783095070105623107215331596699973092056135872020481",
		)
		.unwrap();
		// If the fixed point number is positive, we directly convert the inner val to U256
		if !self.is_negative() {
			inner_val = U256::from(self.into_inner());
		} else {
			// If the fixed point number is negative then we need to wrap the value
			// i.e. -x is equivalent to PRIME - x (or -x % PRIME) where x is a positive number
			inner_val = prime - (U256::from(-self.into_inner()));
		}
		inner_val
	}
}

impl StringExt for &str {
	fn to_felt_rep(&self) -> u128 {
		let a = FieldElement::from_byte_slice_be(self.as_bytes());
		u128::try_from(a.unwrap()).unwrap()
	}
}

impl U256Ext for U256 {
	fn try_to_felt(&self) -> Result<FieldElement, FromByteSliceError> {
		let mut buffer: [u8; 32] = [0; 32];
		self.to_big_endian(&mut buffer);
		FieldElement::from_byte_slice_be(&buffer)
	}
}

impl FieldElementExt for FieldElement {
	fn to_u256(&self) -> U256 {
		let buffer = self.to_bytes_be();
		U256::from_big_endian(&buffer)
	}
}
