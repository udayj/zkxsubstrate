use crate::traits::FeltSerializedArrayExt;
use crate::types::common::convert_to_u128_pair;
use crate::types::{
	Asset, AssetRemovedL2, AssetUpdatedL2, Market, MarketRemovedL2, MarketUpdatedL2,
	TradingAccountMinimal, UniversalEventL2, UserDepositL2,
};
use frame_support::inherent::Vec;
use primitive_types::U256;
use sp_arithmetic::fixed_point::FixedI128;
use sp_runtime::traits::ConstU32;
use sp_runtime::BoundedVec;
use starknet_ff::{FieldElement, FromByteSliceError};

impl FeltSerializedArrayExt for Vec<FieldElement> {
	fn append_bounded_vec(&mut self, vec: &BoundedVec<u8, ConstU32<256>>) {
		self.extend(vec.iter().map(|&value| FieldElement::from(value)));
	}

	fn append_bool(&mut self, boolean_value: bool) {
		match boolean_value {
			true => self.push(FieldElement::ONE),
			false => self.push(FieldElement::ZERO),
		};
	}

	fn try_append_u256(&mut self, u256_value: U256) -> Result<(), FromByteSliceError> {
		let (low_bytes_felt, high_bytes_felt) = convert_to_u128_pair(u256_value)?;
		self.push(low_bytes_felt);
		self.push(high_bytes_felt);

		Ok(())
	}

	fn try_append_fixedi128(&mut self, fixed_value: FixedI128) -> Result<(), FromByteSliceError> {
		let inner_value: U256 = U256::from(fixed_value.into_inner().abs());
		let u256_value = inner_value * 10_u8.pow(8);

		let (low_bytes_felt, high_bytes_felt) = convert_to_u128_pair(u256_value)?;
		self.push(low_bytes_felt);
		self.push(high_bytes_felt);

		Ok(())
	}

	fn try_append_asset(&mut self, asset: &Asset) -> Result<(), FromByteSliceError> {
		self.push(FieldElement::from(asset.id));
		self.append_bounded_vec(&asset.name);
		self.append_bool(asset.is_tradable);
		self.append_bool(asset.is_collateral);
		self.push(FieldElement::from(asset.token_decimal));

		Ok(())
	}

	fn try_append_market(&mut self, market: &Market) -> Result<(), FromByteSliceError> {
		self.push(FieldElement::from(market.id));
		self.push(FieldElement::from(market.asset));
		self.push(FieldElement::from(market.asset_collateral));
		self.append_bool(market.is_tradable);
		self.append_bool(market.is_archived);
		self.push(FieldElement::from(market.ttl));
		self.try_append_fixedi128(market.tick_size)?;
		self.push(FieldElement::from(market.tick_precision));
		self.try_append_fixedi128(market.step_size)?;
		self.push(FieldElement::from(market.step_precision));
		self.try_append_fixedi128(market.minimum_order_size)?;
		self.try_append_fixedi128(market.minimum_leverage)?;
		self.try_append_fixedi128(market.maximum_leverage)?;
		self.try_append_fixedi128(market.currently_allowed_leverage)?;
		self.try_append_fixedi128(market.maintenance_margin_fraction)?;
		self.try_append_fixedi128(market.initial_margin_fraction)?;
		self.try_append_fixedi128(market.incremental_initial_margin_fraction)?;
		self.try_append_fixedi128(market.incremental_position_size)?;
		self.try_append_fixedi128(market.baseline_position_size)?;
		self.try_append_fixedi128(market.maximum_position_size)?;

		Ok(())
	}

	fn try_append_trading_account(
		&mut self,
		trading_account: &TradingAccountMinimal,
	) -> Result<(), FromByteSliceError> {
		self.try_append_u256(trading_account.account_address)?;
		self.push(FieldElement::from(trading_account.index));

		Ok(())
	}

	fn try_append_market_updated_event(
		&mut self,
		market_updated_event: &MarketUpdatedL2,
	) -> Result<(), FromByteSliceError> {
		self.try_append_u256(market_updated_event.event_hash)?;
		self.try_append_u256(market_updated_event.event_name)?;
		self.push(FieldElement::from(market_updated_event.id));
		self.try_append_market(&market_updated_event.market)?;
		self.append_bounded_vec(&market_updated_event.metadata_url);
		self.append_bounded_vec(&market_updated_event.icon_url);
		self.push(FieldElement::from(market_updated_event.version));
		self.push(FieldElement::from(market_updated_event.block_number));

		Ok(())
	}

	fn try_append_asset_updated_event(
		&mut self,
		asset_updated_event: &AssetUpdatedL2,
	) -> Result<(), FromByteSliceError> {
		self.try_append_u256(asset_updated_event.event_hash)?;
		self.try_append_u256(asset_updated_event.event_name)?;
		self.push(FieldElement::from(asset_updated_event.id));
		self.try_append_asset(&asset_updated_event.asset)?;
		self.append_bounded_vec(&asset_updated_event.metadata_url);
		self.append_bounded_vec(&asset_updated_event.icon_url);
		self.push(FieldElement::from(asset_updated_event.version));
		self.push(FieldElement::from(asset_updated_event.block_number));

		Ok(())
	}

	fn try_append_market_removed_event(
		&mut self,
		market_removed_event: &MarketRemovedL2,
	) -> Result<(), FromByteSliceError> {
		self.try_append_u256(market_removed_event.event_hash)?;
		self.try_append_u256(market_removed_event.event_name)?;
		self.push(FieldElement::from(market_removed_event.id));
		self.push(FieldElement::from(market_removed_event.block_number));

		Ok(())
	}

	fn try_append_asset_removed_event(
		&mut self,
		asset_removed_event: &AssetRemovedL2,
	) -> Result<(), FromByteSliceError> {
		self.try_append_u256(asset_removed_event.event_hash)?;
		self.try_append_u256(asset_removed_event.event_name)?;
		self.push(FieldElement::from(asset_removed_event.id));
		self.push(FieldElement::from(asset_removed_event.block_number));

		Ok(())
	}

	fn try_append_user_deposit_event(
		&mut self,
		user_deposit_event: &UserDepositL2,
	) -> Result<(), FromByteSliceError> {
		self.try_append_u256(user_deposit_event.event_hash)?;
		self.try_append_u256(user_deposit_event.event_name)?;
		self.try_append_trading_account(&user_deposit_event.trading_account)?;
		self.push(FieldElement::from(user_deposit_event.collateral_id));
		self.try_append_u256(user_deposit_event.nonce)?;
		self.try_append_u256(user_deposit_event.amount)?;
		self.try_append_u256(user_deposit_event.balance)?;
		self.push(FieldElement::from(user_deposit_event.block_number));

		Ok(())
	}

	fn try_append_universal_event_array(
		&mut self,
		universal_event_array: &Vec<UniversalEventL2>,
	) -> Result<(), FromByteSliceError> {
		for event in universal_event_array.iter() {
			match event {
				UniversalEventL2::MarketUpdatedL2(market_updated) => {
					self.try_append_market_updated_event(market_updated)?;
				},
				UniversalEventL2::AssetUpdatedL2(asset_updated) => {
					self.try_append_asset_updated_event(asset_updated)?;
				},
				UniversalEventL2::MarketRemovedL2(market_removed) => {
					self.try_append_market_removed_event(market_removed)?;
				},
				UniversalEventL2::AssetRemovedL2(asset_removed) => {
					self.try_append_asset_removed_event(asset_removed)?;
				},
				UniversalEventL2::UserDepositL2(user_deposit) => {
					self.try_append_user_deposit_event(user_deposit)?;
				},
			}
		}

		Ok(())
	}
}
