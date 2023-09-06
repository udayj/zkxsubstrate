use crate::traits::{IntoFelt, TryIntoFelt};
use crate::types::{Asset, Market, TradingAccountMinimal};
use crate::FieldElement;
use codec::{Decode, Encode};
use frame_support::inherent::Vec;
use primitive_types::U256;
use scale_info::TypeInfo;
use sp_runtime::traits::ConstU32;
use sp_runtime::{BoundedVec, RuntimeDebug};

#[derive(Clone, Decode, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct SyncSignature {
	pub signer_index: u8,
	pub r: U256,
	pub s: U256,
}

#[derive(Clone, Decode, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub enum UniversalEventL2 {
	MarketUpdatedL2(MarketUpdatedL2),
	AssetUpdatedL2(AssetUpdatedL2),
	MarketRemovedL2(MarketRemovedL2),
	AssetRemovedL2(AssetRemovedL2),
	FundsTransferL2(FundsTransferL2),
	UserDepositL2(UserDepositL2),
}

#[derive(Clone, Copy, Decode, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct FundsTransferL2 {
	pub event_hash: U256,
	pub event_name: U256,
	pub from_fund: FundType,
	pub to_fund: FundType,
	pub asset_id: u64,
	pub amount: U256,
	pub block_number: u64,
}

#[derive(Clone, Copy, Decode, Default, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct MarketRemovedL2 {
	pub event_hash: U256,
	pub event_name: U256,
	pub id: u64,
	pub block_number: u64,
}

#[derive(Clone, Copy, Decode, Default, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct AssetRemovedL2 {
	pub event_hash: U256,
	pub event_name: U256,
	pub id: u64,
	pub block_number: u64,
}

#[derive(Clone, Copy, Decode, Default, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct UserDepositL2 {
	pub event_hash: U256,
	pub event_name: U256,
	pub trading_account: TradingAccountMinimal,
	pub collateral_id: u64,
	pub nonce: U256,
	pub amount: U256,
	pub balance: U256,
	pub block_number: u64,
}

#[derive(Clone, Copy, Decode, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub enum FundType {
	Admin,
	InsuranceFund,
	HoldingFund,
	EmergencyFund,
}

#[derive(Clone, Decode, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct MarketUpdatedL2 {
	pub event_hash: U256,
	pub event_name: U256,
	pub id: u64,
	pub market: Market,
	pub metadata_url: BoundedVec<u8, ConstU32<256>>,
	pub icon_url: BoundedVec<u8, ConstU32<256>>,
	pub version: u16,
	pub block_number: u64,
}

#[derive(Clone, Decode, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct AssetUpdatedL2 {
	pub event_hash: U256,
	pub event_name: U256,
	pub id: u64,
	pub asset: Asset,
	pub metadata_url: BoundedVec<u8, ConstU32<256>>,
	pub icon_url: BoundedVec<u8, ConstU32<256>>,
	pub version: u16,
	pub block_number: u64,
}

pub trait ConvertToFelt252 {
	fn serialize_to_felt_array(&self) -> Vec<FieldElement>;
}

impl IntoFelt for AssetUpdatedL2 {
	fn into_felt(&self, result: &mut Vec<FieldElement>) {
		self.event_hash.try_into_felt(result);
		self.event_name.try_into_felt(result);
		result.push(FieldElement::from(self.id));
		self.asset.into_felt(result);
		self.metadata_url.into_felt(result);
		self.icon_url.into_felt(result);
		result.push(FieldElement::from(self.version));
		result.push(FieldElement::from(self.block_number));
	}
}

impl IntoFelt for MarketUpdatedL2 {
	fn into_felt(&self, result: &mut Vec<FieldElement>) {
		self.event_hash.try_into_felt(result);
		self.event_name.try_into_felt(result);
		result.push(FieldElement::from(self.id));
		self.market.into_felt(result);
		self.metadata_url.into_felt(result);
		self.icon_url.into_felt(result);
		result.push(FieldElement::from(self.version));
		result.push(FieldElement::from(self.block_number));
	}
}

impl IntoFelt for MarketRemovedL2 {
	fn into_felt(&self, result: &mut Vec<FieldElement>) {
		self.event_hash.try_into_felt(result);
		self.event_name.try_into_felt(result);
		result.push(FieldElement::from(self.id));
		result.push(FieldElement::from(self.block_number));
	}
}

impl IntoFelt for AssetRemovedL2 {
	fn into_felt(&self, result: &mut Vec<FieldElement>) {
		self.event_hash.try_into_felt(result);
		self.event_name.try_into_felt(result);
		result.push(FieldElement::from(self.id));
		result.push(FieldElement::from(self.block_number));
	}
}

impl IntoFelt for UserDepositL2 {
	fn into_felt(&self, result: &mut Vec<FieldElement>) {
		self.event_hash.try_into_felt(result);
		self.event_name.try_into_felt(result);
		self.trading_account.into_felt(result);
		result.push(FieldElement::from(self.collateral_id));
		self.nonce.try_into_felt(result);
		self.amount.try_into_felt(result);
		self.balance.try_into_felt(result);
		result.push(FieldElement::from(self.block_number));
	}
}

impl IntoFelt for FundType {
	fn into_felt(&self, result: &mut Vec<FieldElement>) {
		match self {
			FundType::Admin => result.push(FieldElement::ZERO),
			FundType::InsuranceFund => result.push(FieldElement::ONE),
			FundType::HoldingFund => result.push(FieldElement::TWO),
			FundType::EmergencyFund => result.push(FieldElement::THREE),
		};
	}
}

impl IntoFelt for FundsTransferL2 {
	fn into_felt(&self, result: &mut Vec<FieldElement>) {
		self.event_hash.try_into_felt(result);
		self.event_name.try_into_felt(result);
		self.from_fund.into_felt(result);
		self.to_fund.into_felt(result);
		result.push(FieldElement::from(self.asset_id));
		self.amount.try_into_felt(result);
		result.push(FieldElement::from(self.block_number));
	}
}

impl ConvertToFelt252 for [UniversalEventL2] {
	fn serialize_to_felt_array(&self) -> Vec<FieldElement> {
		let result: &mut Vec<FieldElement> = &mut Vec::new();
		for event in self.iter() {
			match event {
				UniversalEventL2::MarketUpdatedL2(market_updated) => {
					market_updated.into_felt(result);
				},
				UniversalEventL2::AssetUpdatedL2(asset_updated) => {
					asset_updated.into_felt(result);
				},
				UniversalEventL2::MarketRemovedL2(market_removed) => {
					market_removed.into_felt(result);
				},
				UniversalEventL2::AssetRemovedL2(asset_removed) => {
					asset_removed.into_felt(result);
				},
				UniversalEventL2::FundsTransferL2(funds_transfer) => {
					funds_transfer.into_felt(result);
				},
				UniversalEventL2::UserDepositL2(user_deposit) => {
					user_deposit.into_felt(result);
				},
			}
		}

		result.to_vec()
	}
}
