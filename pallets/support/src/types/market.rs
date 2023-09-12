use crate::traits::{FeltSerializable, TryFeltSerializable};
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

impl TryFeltSerializable for Market {
	fn try_felt_serialized(
		&self,
		result: &mut Vec<FieldElement>,
	) -> Result<(), FromByteSliceError> {
		self.id.try_felt_serialized(result)?;
		self.asset.try_felt_serialized(result)?;
		self.asset_collateral.try_felt_serialized(result)?;
		self.is_tradable.felt_serialized(result);
		self.is_archived.felt_serialized(result);
		result.push(FieldElement::from(self.ttl));
		self.tick_size.try_felt_serialized(result)?;
		result.push(FieldElement::from(self.tick_precision));
		self.step_size.try_felt_serialized(result)?;
		result.push(FieldElement::from(self.step_precision));
		self.minimum_order_size.try_felt_serialized(result)?;
		self.minimum_leverage.try_felt_serialized(result)?;
		self.maximum_leverage.try_felt_serialized(result)?;
		self.currently_allowed_leverage.try_felt_serialized(result)?;
		self.maintenance_margin_fraction.try_felt_serialized(result)?;
		self.initial_margin_fraction.try_felt_serialized(result)?;
		self.incremental_initial_margin_fraction.try_felt_serialized(result)?;
		self.incremental_position_size.try_felt_serialized(result)?;
		self.baseline_position_size.try_felt_serialized(result)?;
		self.maximum_position_size.try_felt_serialized(result)?;

		Ok(())
	}
}
