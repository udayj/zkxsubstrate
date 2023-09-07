use crate::traits::{IntoFelt, TryIntoFelt};
use crate::FieldElement;
use codec::{Decode, Encode};
use frame_support::inherent::Vec;
use primitive_types::U256;
use scale_info::TypeInfo;
use sp_arithmetic::fixed_point::FixedI128;
use sp_runtime::RuntimeDebug;
use starknet_ff::FromByteSliceError;

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

impl TryIntoFelt for Market {
	fn try_into_felt(&self, result: &mut Vec<FieldElement>) -> Result<(), FromByteSliceError> {
		self.id.try_into_felt(result)?;
		self.asset.try_into_felt(result)?;
		self.asset_collateral.try_into_felt(result)?;
		self.is_tradable.into_felt(result);
		self.is_archived.into_felt(result);
		result.push(FieldElement::from(self.ttl));
		self.tick_size.try_into_felt(result)?;
		result.push(FieldElement::from(self.tick_precision));
		self.step_size.try_into_felt(result)?;
		result.push(FieldElement::from(self.step_precision));
		self.minimum_order_size.try_into_felt(result)?;
		self.minimum_leverage.try_into_felt(result)?;
		self.maximum_leverage.try_into_felt(result)?;
		self.currently_allowed_leverage.try_into_felt(result)?;
		self.maintenance_margin_fraction.try_into_felt(result)?;
		self.initial_margin_fraction.try_into_felt(result)?;
		self.incremental_initial_margin_fraction.try_into_felt(result)?;
		self.incremental_position_size.try_into_felt(result)?;
		self.baseline_position_size.try_into_felt(result)?;
		self.maximum_position_size.try_into_felt(result)?;

		Ok(())
	}
}
