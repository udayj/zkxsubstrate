use crate::traits::{FeltSerializable, TryFeltSerializable};
use crate::FieldElement;
use codec::{Decode, Encode};
use frame_support::inherent::Vec;
use frame_support::pallet_prelude::MaxEncodedLen;
use primitive_types::U256;
use scale_info::TypeInfo;
use sp_arithmetic::fixed_point::FixedI128;
use sp_runtime::RuntimeDebug;
use starknet_ff::FromByteSliceError;

#[derive(
	Encode, Decode, Default, Clone, Copy, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug,
)]
pub struct TradingAccount {
	pub account_id: U256,
	pub account_address: U256,
	pub index: u8,
	pub pub_key: U256,
}

#[derive(
	Encode, Decode, Default, Clone, Copy, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug,
)]
pub struct TradingAccountWithoutId {
	pub account_address: U256,
	pub index: u8,
	pub pub_key: U256,
}

#[derive(
	Encode, Decode, Default, Clone, Copy, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug,
)]
pub struct TradingAccountMinimal {
	pub account_address: U256,
	pub index: u8,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
pub struct BalanceUpdate {
	pub asset_id: U256,
	pub balance_value: FixedI128,
}

impl TryFeltSerializable for TradingAccountMinimal {
	fn try_felt_serialized(
		&self,
		result: &mut Vec<FieldElement>,
	) -> Result<(), FromByteSliceError> {
		self.account_address.try_felt_serialized(result)?;
		result.push(FieldElement::from(self.index));

		Ok(())
	}
}
