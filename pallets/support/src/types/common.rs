use crate::traits::{FixedI128Ext, IntoFelt, TryIntoFelt};
use crate::FieldElement;
use codec::{Decode, Encode};
use frame_support::inherent::Vec;
use primitive_types::U256;
use scale_info::TypeInfo;
use sp_arithmetic::{fixed_point::FixedI128, traits::CheckedDiv, FixedPointNumber};
use sp_runtime::traits::ConstU32;
use sp_runtime::{BoundedVec, RuntimeDebug};
use starknet_ff::FromByteSliceError;

fn convert_to_u128_pair(
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

impl IntoFelt for BoundedVec<u8, ConstU32<256>> {
	fn into_felt(&self, result: &mut Vec<FieldElement>) {
		result.extend(self.iter().map(|&value| FieldElement::from(value)));
	}
}

impl IntoFelt for bool {
	fn into_felt(&self, result: &mut Vec<FieldElement>) {
		match &self {
			true => result.push(FieldElement::ONE),
			false => result.push(FieldElement::ZERO),
		};
	}
}

impl TryIntoFelt for U256 {
	fn try_into_felt(&self, result: &mut Vec<FieldElement>) -> Result<(), FromByteSliceError> {
		let (low_bytes_felt, high_bytes_felt) = convert_to_u128_pair(*self)?;
		result.push(low_bytes_felt);
		result.push(high_bytes_felt);

		Ok(())
	}
}

impl TryIntoFelt for FixedI128 {
	fn try_into_felt(&self, result: &mut Vec<FieldElement>) -> Result<(), FromByteSliceError> {
		let inner_value: U256 = U256::from(self.into_inner().abs());
		let u256_value = inner_value * 10_u8.pow(8);

		let (low_bytes_felt, high_bytes_felt) = convert_to_u128_pair(u256_value)?;
		result.push(low_bytes_felt);
		result.push(high_bytes_felt);

		Ok(())
	}
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
