use crate::traits::{IntoFelt, TryIntoFelt};
use crate::FieldElement;
use codec::{Decode, Encode};
use frame_support::inherent::Vec;
use primitive_types::U256;
use scale_info::TypeInfo;
use sp_runtime::traits::ConstU32;
use sp_runtime::{BoundedVec, RuntimeDebug};
use starknet_ff::FromByteSliceError;

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
		let mut buffer: [u8; 32] = [0; 32];
		self.to_big_endian(&mut buffer);

		let felt_rep = FieldElement::from_byte_slice_be(&buffer)?;
		result.push(felt_rep);
		Ok(())
	}
}
