use crate::{
	traits::{FeltSerializedArrayExt, U256Ext},
	types::{
		common::convert_to_u128_pair, Asset, AssetRemoved, AssetUpdated, Market, MarketRemoved,
		MarketUpdated, MarketUpdatedV2, ReferralDetailsAdded, Setting, SettingsAdded, SignerAdded,
		SignerRemoved, TradingAccountMinimal, UniversalEvent, UserDeposit,
	},
};
use frame_support::dispatch::Vec;
use primitive_types::U256;
use sp_arithmetic::fixed_point::FixedI128;
use sp_runtime::{traits::ConstU32, BoundedVec};
use starknet_ff::{FieldElement, FromByteSliceError};

use super::{AssetAddress, InsuranceFundDeposited, MasterAccountLevelChanged, QuorumSet};

impl FeltSerializedArrayExt for Vec<FieldElement> {
	fn append_bounded_vec_u8(&mut self, vec: &BoundedVec<u8, ConstU32<256>>) {
		self.extend(vec.iter().map(|&value| FieldElement::from(value)));
	}

	fn try_append_bounded_vec_fixed_i128(
		&mut self,
		vec: &BoundedVec<FixedI128, ConstU32<256>>,
	) -> Result<(), FromByteSliceError> {
		vec.iter().try_for_each(|value| {
			self.try_append_fixedi128(*value)?;

			Ok(())
		})
	}

	fn try_append_asset_addresses(
		&mut self,
		vec: &BoundedVec<AssetAddress, ConstU32<256>>,
	) -> Result<(), FromByteSliceError> {
		vec.iter().try_for_each(|asset_address| {
			self.push(FieldElement::from(asset_address.chain));
			self.try_append_u256_pair(asset_address.address)?;

			Ok(())
		})
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
		let inner_value: u128 = fixed_value
			.into_inner()
			.try_into()
			.map_err(|_| FromByteSliceError::OutOfRange)?;

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

	fn try_append_settings(
		&mut self,
		settings: &BoundedVec<Setting, ConstU32<256>>,
	) -> Result<(), FromByteSliceError> {
		settings.iter().try_for_each(|setting| {
			self.try_append_u256(setting.key)?;
			self.try_append_bounded_vec_fixed_i128(&setting.values)?;

			Ok(())
		})
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
		self.append_bounded_vec_u8(&market_updated_event.metadata_url);
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
		self.try_append_asset_addresses(&asset_updated_event.asset_addresses)?;
		self.try_append_asset(&asset_updated_event.asset)?;
		self.append_bounded_vec_u8(&asset_updated_event.metadata_url);
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

	fn try_append_settings_added_event(
		&mut self,
		settings_added: &SettingsAdded,
	) -> Result<(), FromByteSliceError> {
		// enum prefix
		self.push(FieldElement::from(8_u8));
		self.push(FieldElement::from(settings_added.event_index));
		self.try_append_settings(&settings_added.settings)?;
		self.push(FieldElement::from(settings_added.block_number));

		Ok(())
	}

	fn append_quorum_set_event(&mut self, quorum_set: &QuorumSet) {
		// enum prefix
		self.push(FieldElement::from(7_u8));
		self.push(FieldElement::from(quorum_set.event_index));
		self.push(FieldElement::from(quorum_set.quorum));
		self.push(FieldElement::from(quorum_set.block_number));
	}

	fn try_append_referral_added_event(
		&mut self,
		referral_added_event: &ReferralDetailsAdded,
	) -> Result<(), FromByteSliceError> {
		// enum prefix
		self.push(FieldElement::from(9_u8));
		self.push(FieldElement::from(referral_added_event.event_index));
		self.try_append_u256(referral_added_event.master_account_address)?;
		self.try_append_u256(referral_added_event.referral_account_address)?;
		self.push(FieldElement::from(referral_added_event.level));
		self.try_append_u256(referral_added_event.referral_code)?;
		self.try_append_fixedi128(referral_added_event.fee_discount)?;
		self.push(FieldElement::from(referral_added_event.block_number));

		Ok(())
	}

	fn try_append_account_level_updated_event(
		&mut self,
		account_level_updated_event: &MasterAccountLevelChanged,
	) -> Result<(), FromByteSliceError> {
		// enum prefix
		self.push(FieldElement::from(10_u8));
		self.push(FieldElement::from(account_level_updated_event.event_index));
		self.try_append_u256(account_level_updated_event.master_account_address)?;
		self.push(FieldElement::from(account_level_updated_event.level));
		self.push(FieldElement::from(account_level_updated_event.block_number));

		Ok(())
	}

	fn try_append_market_updated_v2_event(
		&mut self,
		market_updated_v2_event: &MarketUpdatedV2,
	) -> Result<(), FromByteSliceError> {
		// enum prefix
		self.push(FieldElement::from(11_u8));
		self.push(FieldElement::from(market_updated_v2_event.event_index));
		self.push(FieldElement::from(market_updated_v2_event.id));
		self.try_append_market(&market_updated_v2_event.market)?;
		self.append_bounded_vec_u8(&market_updated_v2_event.metadata_url);
		self.try_append_u256(market_updated_v2_event.fee_split_details.0)?;
		self.try_append_fixedi128(market_updated_v2_event.fee_split_details.1)?;
		self.push(FieldElement::from(market_updated_v2_event.block_number));

		Ok(())
	}

	fn try_append_insurance_fund_deposited(
		&mut self,
		insurance_fund_deposited: &InsuranceFundDeposited,
	) -> Result<(), FromByteSliceError> {
		// enum prefix
		self.push(FieldElement::from(12_u8));
		self.push(FieldElement::from(insurance_fund_deposited.event_index));
		self.try_append_u256(insurance_fund_deposited.insurance_fund)?;
		self.push(FieldElement::from(insurance_fund_deposited.collateral_id));
		self.try_append_fixedi128(insurance_fund_deposited.amount)?;
		self.push(FieldElement::from(insurance_fund_deposited.block_number));

		Ok(())
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
				UniversalEvent::SettingsAdded(settings_added) => {
					self.try_append_settings_added_event(settings_added)?;
				},
				UniversalEvent::ReferralDetailsAdded(referral_added) => {
					self.try_append_referral_added_event(referral_added)?;
				},
				UniversalEvent::MasterAccountLevelChanged(account_level_updated) => {
					self.try_append_account_level_updated_event(account_level_updated)?;
				},
				UniversalEvent::MarketUpdatedV2(market_updated_v2) => {
					self.try_append_market_updated_v2_event(market_updated_v2)?;
				},
				UniversalEvent::InsuranceFundDeposited(insurance_fund_deposited) =>
					self.try_append_insurance_fund_deposited(insurance_fund_deposited)?,
			}
		}

		Ok(())
	}
}
