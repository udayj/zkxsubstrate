use crate::traits::{IntoFelt, TryIntoFelt};
use crate::FieldElement;
use codec::{Decode, Encode};
use frame_support::inherent::Vec;
use primitive_types::U256;
use scale_info::TypeInfo;
use sp_arithmetic::FixedI128;
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
