use crate::{
	helpers::compute_hash_on_elements,
	traits::{FixedI128Ext, Hashable, U256Ext},
	types::common::{convert_to_u128_pair, HashType},
};
use codec::{Decode, Encode};
use frame_support::{dispatch::Vec, pallet_prelude::MaxEncodedLen};
use primitive_types::U256;
use scale_info::TypeInfo;
use sp_arithmetic::fixed_point::FixedI128;
use sp_runtime::RuntimeDebug;
use starknet_crypto::poseidon_hash_many;
use starknet_ff::{FieldElement, FromByteSliceError};

#[derive(
	Encode, Decode, Default, Clone, Copy, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug,
)]
pub enum VolumeType {
	#[default]
	UserVolume,
	MasterVolume,
}

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
pub struct TradingAccountMinimal {
	pub account_address: U256,
	pub pub_key: U256,
	pub index: u8,
}

#[derive(Clone, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub struct MonetaryAccountDetails {
	pub monetary_account: U256,
	pub trading_accounts: Vec<U256>,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
pub struct BalanceUpdate {
	pub asset_id: u128,
	pub balance_value: FixedI128,
}

#[derive(Clone, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub struct WithdrawalRequest {
	pub account_id: U256,
	pub collateral_id: u128,
	pub amount: FixedI128,
	pub timestamp: u64,
	pub sig_r: U256,
	pub sig_s: U256,
	pub hash_type: HashType,
}

#[derive(
	Encode, Decode, Default, Clone, Copy, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug,
)]
pub struct ReferralDetails {
	pub master_account_address: U256,
	pub fee_discount: FixedI128,
}

impl TradingAccountMinimal {
	pub fn new(account_address: U256, pub_key: U256, index: u8) -> TradingAccountMinimal {
		TradingAccountMinimal { account_address, pub_key, index }
	}
}

#[derive(Clone, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub struct FeeSharesInput {
	pub master_account_address: U256,
	pub collateral_id: u128,
	pub amount: FixedI128,
}

impl TradingAccount {
	pub fn to_trading_account_minimal(&self) -> TradingAccountMinimal {
		TradingAccountMinimal {
			account_address: self.account_address,
			pub_key: self.pub_key,
			index: self.index,
		}
	}
}

impl Hashable for WithdrawalRequest {
	// No error apart from error during conversion from U256 to FieldElement should happen
	// Hence associated type is defined to be exactly that error i.e.
	// starknet_ff::FromByteSliceError
	type ConversionError = FromByteSliceError;

	fn hash(&self, hash_type: &HashType) -> Result<FieldElement, Self::ConversionError> {
		let (account_id_low, account_id_high) = convert_to_u128_pair(self.account_id)?;
		let mut elements = Vec::<FieldElement>::new();
		elements.push(account_id_low);
		elements.push(account_id_high);
		elements.push(FieldElement::from(self.collateral_id));
		elements.push(self.amount.to_u256().try_to_felt()?);
		elements.push(FieldElement::from(self.timestamp));

		let result = match hash_type {
			HashType::Pedersen => compute_hash_on_elements(&elements),
			HashType::Poseidon => poseidon_hash_many(&elements),
		};

		Ok(result)
	}
}
