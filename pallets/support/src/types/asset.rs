use crate::traits::{FeltSerializable, TryFeltSerializable};
use crate::FieldElement;
use codec::{Decode, Encode};
use frame_support::inherent::Vec;
use primitive_types::U256;
use scale_info::TypeInfo;
use sp_runtime::traits::ConstU32;
use sp_runtime::{BoundedVec, RuntimeDebug};
use starknet_ff::FromByteSliceError;

#[derive(Clone, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Asset {
	pub id: U256,
	pub name: BoundedVec<u8, ConstU32<256>>,
	pub is_tradable: bool,
	pub is_collateral: bool,
	pub token_decimal: u8,
}

impl TryFeltSerializable for Asset {
	fn try_felt_serialized(
		&self,
		result: &mut Vec<FieldElement>,
	) -> Result<(), FromByteSliceError> {
		self.id.try_felt_serialized(result)?;
		self.name.felt_serialized(result);
		self.is_tradable.felt_serialized(result);
		self.is_collateral.felt_serialized(result);
		result.push(FieldElement::from(self.token_decimal));

		Ok(())
	}
}
