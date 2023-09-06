use crate::traits::{IntoFelt, TryIntoFelt};
use crate::FieldElement;
use codec::{Decode, Encode};
use frame_support::inherent::Vec;
use primitive_types::U256;
use scale_info::TypeInfo;
use sp_arithmetic::fixed_point::FixedI128;
use sp_runtime::RuntimeDebug;

#[derive(Clone, Encode, Decode, Default, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Market {
	pub id: U256,
	pub asset: U256,
	pub asset_collateral: U256,
	pub is_tradable: bool,
	pub is_archived: bool,
	pub ttl: u32,
	pub tick_size: FixedI128,
	pub tick_precision: u8,
	pub step_size: FixedI128,
	pub step_precision: u8,
	pub minimum_order_size: FixedI128,
	pub minimum_leverage: FixedI128,
	pub maximum_leverage: FixedI128,
	pub currently_allowed_leverage: FixedI128,
	pub maintenance_margin_fraction: FixedI128,
	pub initial_margin_fraction: FixedI128,
	pub incremental_initial_margin_fraction: FixedI128,
	pub incremental_position_size: FixedI128,
	pub baseline_position_size: FixedI128,
	pub maximum_position_size: FixedI128,
}

impl IntoFelt for Market {
	fn into_felt(&self, result: &mut Vec<FieldElement>) {
		self.id.try_into_felt(result);
		self.asset.try_into_felt(result);
		self.asset_collateral.try_into_felt(result);
		self.is_tradable.into_felt(result);
		self.is_archived.into_felt(result);
		result.push(FieldElement::from(self.ttl));
		// TODO (merkle-groot): Add impl to convert i128 to felt252
		result.push(FieldElement::from(self.tick_precision));
		result.push(FieldElement::from(self.step_precision));
	}
}
