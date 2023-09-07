use crate::traits::{IntoFelt, TryIntoFelt};
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

impl TryIntoFelt for Asset {
	fn try_into_felt(&self, result: &mut Vec<FieldElement>) -> Result<(), FromByteSliceError> {
		self.id.try_into_felt(result)?;
		self.name.into_felt(result);
		self.is_tradable.into_felt(result);
		self.is_collateral.into_felt(result);
		result.push(FieldElement::from(self.token_decimal));

		Ok(())
	}
}
