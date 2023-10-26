use crate::traits::{FeltSerializedArrayExt, U256Ext};
use crate::types::common::convert_to_u128_pair;
use crate::types::{
	Asset, AssetRemoved, AssetUpdated, Market, MarketRemoved, MarketUpdated, SignerAdded,
	SignerRemoved, TradingAccountMinimal, UniversalEvent, UserDeposit,
};
use frame_support::dispatch::Vec;
use primitive_types::U256;
use sp_arithmetic::fixed_point::FixedI128;
use sp_runtime::traits::ConstU32;
use sp_runtime::BoundedVec;
use starknet_ff::{FieldElement, FromByteSliceError};

use super::QuorumSet;

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

	fn try_append_u256_pair(&mut self, u256_value: U256) -> Result<(), FromByteSliceError> {
		let (low_bytes_felt, high_bytes_felt) = convert_to_u128_pair(u256_value)?;
		self.push(low_bytes_felt);
		self.push(high_bytes_felt);

		Ok(())
	}

	fn try_append_u256(&mut self, u256_value: U256) -> Result<(), FromByteSliceError> {
		let felt_value = u256_value.try_to_felt()?;
		self.push(felt_value);

		Ok(())
	}

	fn try_append_fixedi128(&mut self, fixed_value: FixedI128) -> Result<(), FromByteSliceError> {
		// This works as fixedI128 values in l2 events are positive
		let inner_value: u128 = fixed_value.into_inner().try_into().unwrap();

		self.push(FieldElement::from(inner_value));
		Ok(())
	}

	fn try_append_asset(&mut self, asset: &Asset) -> Result<(), FromByteSliceError> {
		self.push(FieldElement::from(asset.id));
		self.try_append_u256(asset.short_name)?;
		self.append_bool(asset.is_tradable);
		self.append_bool(asset.is_collateral);
		self.push(FieldElement::from(asset.decimals));

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
		self.try_append_u256(trading_account.pub_key)?;
		self.push(FieldElement::from(trading_account.index));

		Ok(())
	}

	fn try_append_market_updated_event(
		&mut self,
		market_updated_event: &MarketUpdated,
	) -> Result<(), FromByteSliceError> {
		// enum prefix
		self.push(FieldElement::ZERO);
		self.push(FieldElement::from(market_updated_event.event_index));
		self.push(FieldElement::from(market_updated_event.id));
		self.try_append_market(&market_updated_event.market)?;
		self.append_bounded_vec(&market_updated_event.metadata_url);
		self.push(FieldElement::from(market_updated_event.block_number));

		Ok(())
	}

	fn try_append_asset_updated_event(
		&mut self,
		asset_updated_event: &AssetUpdated,
	) -> Result<(), FromByteSliceError> {
		// enum prefix
		self.push(FieldElement::ONE);
		self.push(FieldElement::from(asset_updated_event.event_index));
		self.push(FieldElement::from(asset_updated_event.id));
		self.try_append_asset(&asset_updated_event.asset)?;
		self.append_bounded_vec(&asset_updated_event.metadata_url);
		self.push(FieldElement::from(asset_updated_event.block_number));

		Ok(())
	}

	fn try_append_market_removed_event(
		&mut self,
		market_removed_event: &MarketRemoved,
	) -> Result<(), FromByteSliceError> {
		// enum prefix
		self.push(FieldElement::TWO);
		self.push(FieldElement::from(market_removed_event.event_index));
		self.push(FieldElement::from(market_removed_event.id));
		self.push(FieldElement::from(market_removed_event.block_number));

		Ok(())
	}

	fn try_append_asset_removed_event(
		&mut self,
		asset_removed_event: &AssetRemoved,
	) -> Result<(), FromByteSliceError> {
		// enum prefix
		self.push(FieldElement::THREE);
		self.push(FieldElement::from(asset_removed_event.event_index));
		self.push(FieldElement::from(asset_removed_event.id));
		self.push(FieldElement::from(asset_removed_event.block_number));

		Ok(())
	}

	fn try_append_user_deposit_event(
		&mut self,
		user_deposit_event: &UserDeposit,
	) -> Result<(), FromByteSliceError> {
		// enum prefix
		self.push(FieldElement::from(4_u8));
		self.push(FieldElement::from(user_deposit_event.event_index));
		self.try_append_trading_account(&user_deposit_event.trading_account)?;
		self.push(FieldElement::from(user_deposit_event.collateral_id));
		self.try_append_u256(user_deposit_event.nonce)?;
		self.try_append_fixedi128(user_deposit_event.amount)?;
		self.push(FieldElement::from(user_deposit_event.block_number));

		Ok(())
	}

	fn try_append_signer_added_event(
		&mut self,
		signer_added: &SignerAdded,
	) -> Result<(), FromByteSliceError> {
		// enum prefix
		self.push(FieldElement::from(5_u8));
		self.push(FieldElement::from(signer_added.event_index));
		self.try_append_u256(signer_added.signer)?;
		self.push(FieldElement::from(signer_added.block_number));

		Ok(())
	}

	fn try_append_signer_removed_event(
		&mut self,
		signer_removed: &SignerRemoved,
	) -> Result<(), FromByteSliceError> {
		// enum prefix
		self.push(FieldElement::from(6_u8));
		self.push(FieldElement::from(signer_removed.event_index));
		self.try_append_u256(signer_removed.signer)?;
		self.push(FieldElement::from(signer_removed.block_number));

		Ok(())
	}

	fn append_quorum_set_event(&mut self, quorum_set: &QuorumSet) {
		// enum prefix
		self.push(FieldElement::from(7_u8));
		self.push(FieldElement::from(quorum_set.event_index));
		self.push(FieldElement::from(quorum_set.quorum));
		self.push(FieldElement::from(quorum_set.block_number));
	}

	fn try_append_universal_event_array(
		&mut self,
		universal_event_array: &Vec<UniversalEvent>,
	) -> Result<(), FromByteSliceError> {
		for event in universal_event_array.iter() {
			match event {
				UniversalEvent::MarketUpdated(market_updated) => {
					self.try_append_market_updated_event(market_updated)?;
				},
				UniversalEvent::AssetUpdated(asset_updated) => {
					self.try_append_asset_updated_event(asset_updated)?;
				},
				UniversalEvent::MarketRemoved(market_removed) => {
					self.try_append_market_removed_event(market_removed)?;
				},
				UniversalEvent::AssetRemoved(asset_removed) => {
					self.try_append_asset_removed_event(asset_removed)?;
				},
				UniversalEvent::UserDeposit(user_deposit) => {
					self.try_append_user_deposit_event(user_deposit)?;
				},
				UniversalEvent::SignerAdded(signer_added) => {
					self.try_append_signer_added_event(signer_added)?;
				},
				UniversalEvent::SignerRemoved(signer_removed) => {
					self.try_append_signer_removed_event(signer_removed)?;
				},
				UniversalEvent::QuorumSet(quorum_set) => {
					self.append_quorum_set_event(quorum_set);
				},
			}
		}

		Ok(())
	}
}
