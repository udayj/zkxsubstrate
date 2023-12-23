use crate::types::{
	ABRDetails, AccountInfo, Asset, AssetAddress, AssetRemoved, AssetUpdated, BalanceChangeReason,
	BaseFee, Direction, ExtendedAsset, ExtendedMarket, FeeRates, ForceClosureFlag, HashType,
	MarginInfo, Market, MarketRemoved, MarketUpdated, Order, OrderSide, Position, PositionExtended,
	QuorumSet, Setting, SettingsAdded, Side, SignerAdded, SignerRemoved, TradingAccount,
	TradingAccountMinimal, UniversalEvent, UserDeposit,
};
use frame_support::dispatch::Vec;
use primitive_types::U256;
use sp_arithmetic::fixed_point::FixedI128;
use sp_runtime::{traits::ConstU32, BoundedVec, DispatchResult};
use starknet_ff::{FieldElement, FromByteSliceError};

pub trait TradingAccountInterface {
	type VolumeError;
	fn deposit_internal(
		trading_account: TradingAccountMinimal,
		collateral_id: u128,
		amount: FixedI128,
	);
	fn is_registered_user(account: U256) -> bool;
	fn get_balance(account: U256, asset_id: u128) -> FixedI128;
	fn get_unused_balance(account: U256, asset_id: u128) -> FixedI128;
	fn get_locked_margin(account: U256, asset_id: u128) -> FixedI128;
	fn get_trading_account_id(trading_account: TradingAccountMinimal) -> U256;
	fn set_locked_margin(account: U256, asset_id: u128, amount: FixedI128);
	fn transfer(
		account_id: U256,
		collateral_id: u128,
		amount: FixedI128,
		reason: BalanceChangeReason,
	);
	fn transfer_from(
		account_id: U256,
		collateral_id: u128,
		amount: FixedI128,
		reason: BalanceChangeReason,
	);
	fn get_account(account_id: &U256) -> Option<TradingAccount>;
	fn get_public_key(account: &U256) -> Option<U256>;
	fn get_margin_info(
		account_id: U256,
		collateral_id: u128,
		new_position_maintanence_requirement: FixedI128,
		new_position_margin: FixedI128,
	) -> (bool, FixedI128, FixedI128, FixedI128, FixedI128);
	fn get_account_list(start_index: u128, end_index: u128) -> Vec<U256>;
	fn add_deferred_balance(account_id: U256, collateral_id: u128) -> DispatchResult;
	fn get_accounts_count() -> u128;
	fn get_collaterals_of_user(account_id: U256) -> Vec<u128>;
	fn update_and_get_cumulative_volume(
		account_id: U256,
		market_id: u128,
		new_volume: FixedI128,
	) -> Result<FixedI128, Self::VolumeError>;
	fn get_30day_volume(account_id: U256, market_id: u128) -> Result<FixedI128, Self::VolumeError>;
}

pub trait TradingInterface {
	fn get_markets_of_collateral(account_id: U256, collateral_id: u128) -> Vec<u128>;
	fn get_position(account_id: U256, market_id: u128, direction: Direction) -> Position;
	fn get_positions(account_id: U256, collateral_id: u128) -> Vec<PositionExtended>;
	fn set_flags_for_force_orders(
		account_id: U256,
		collateral_id: u128,
		force_closure_flag: ForceClosureFlag,
		amount_to_be_sold: FixedI128,
	);
	fn get_deleveragable_amount(account_id: U256, collateral_id: u128) -> FixedI128;
	fn get_account_margin_info(account_id: U256, collateral_id: u128) -> MarginInfo;
	fn get_account_info(account_id: U256, collateral_id: u128) -> AccountInfo;
	fn get_account_list(start_index: u128, end_index: u128) -> Vec<U256>;
	fn get_force_closure_flags(account_id: U256, collateral_id: u128) -> Option<ForceClosureFlag>;
	fn get_fee(account_id: U256, market_id: u128) -> (FeeRates, u64);
}

pub trait AssetInterface {
	fn update_asset_internal(asset: ExtendedAsset);
	fn add_asset_internal(asset: ExtendedAsset);
	fn remove_asset_internal(id: u128);
	fn get_default_collateral() -> u128;
	fn get_asset(id: u128) -> Option<Asset>;
}

pub trait RiskManagementInterface {
	fn check_for_risk(
		order: &Order,
		size: FixedI128,
		execution_price: FixedI128,
		oracle_price: FixedI128,
		margin_amount: FixedI128,
	) -> (FixedI128, bool);
	fn check_for_force_closure(
		account_id: U256,
		collateral_id: u128,
		market_id: u128,
		direction: Direction,
	);
}

pub trait MarketInterface {
	fn get_market(id: u128) -> Option<Market>;
	fn add_market_internal(extended_market: ExtendedMarket);
	fn update_market_internal(extended_market: ExtendedMarket);
	fn remove_market_internal(id: u128);
	fn validate_market_details(market: &Market) -> DispatchResult;
	fn get_all_markets() -> Vec<u128>;
	fn get_all_markets_by_state(is_tradable: bool, is_archived: bool) -> Vec<u128>;
}

pub trait PricesInterface {
	fn convert_to_seconds(time_in_milli: u64) -> u64;
	fn get_index_price(market_id: u128) -> FixedI128;
	fn get_mark_price(market_id: u128) -> FixedI128;
	fn get_last_traded_price(market_id: u128) -> FixedI128;
	fn update_last_traded_price(market_id: u128, price: FixedI128);
	fn get_remaining_markets() -> Vec<u128>;
	fn get_no_of_batches_for_current_epoch() -> u64;
	fn get_last_abr_timestamp() -> u64;
	fn get_next_abr_timestamp() -> u64;
	fn get_previous_abr_values(starting_epoch: u64, market_id: u128, n: u64) -> Vec<ABRDetails>;
	fn get_remaining_pay_abr_calls() -> u64;
}

pub trait FixedI128Ext {
	fn round_to_precision(&self, precision: u32) -> Self;
	fn to_u256(&self) -> U256;
}

pub trait StringExt {
	fn to_felt_rep(&self) -> u128;
}

pub trait U256Ext {
	fn try_to_felt(&self) -> Result<FieldElement, FromByteSliceError>;
}

pub trait FieldElementExt {
	fn to_u256(&self) -> U256;
}

pub trait TradingFeesInterface {
	fn update_base_fees_internal(
		collateral_id: u128,
		side: Side,
		order_side: OrderSide,
		fee_details: Vec<BaseFee>,
	) -> DispatchResult;
	fn get_fee_rate(
		collateral_id: u128,
		side: Side,
		order_side: OrderSide,
		volume: FixedI128,
	) -> (FixedI128, u8);
	fn get_all_fee_rates(collateral_id: u128, volume: FixedI128) -> FeeRates;
}

// This trait needs to be implemented by every type that can be hashed (pedersen or poseidon) and
// returns a FieldElement
pub trait Hashable {
	type ConversionError;
	fn hash(&self, hash_type: &HashType) -> Result<FieldElement, Self::ConversionError>;
}

pub trait FeltSerializedArrayExt {
	fn append_bounded_vec_u8(&mut self, vec: &BoundedVec<u8, ConstU32<256>>);
	fn append_bool(&mut self, boolean_value: bool);
	fn append_quorum_set_event(&mut self, quorum_set: &QuorumSet);
	fn try_append_bounded_vec_fixed_i128(
		&mut self,
		vec: &BoundedVec<FixedI128, ConstU32<256>>,
	) -> Result<(), FromByteSliceError>;
	fn try_append_asset_addresses(
		&mut self,
		vec: &BoundedVec<AssetAddress, ConstU32<256>>,
	) -> Result<(), FromByteSliceError>;
	fn try_append_u256(&mut self, u256_value: U256) -> Result<(), FromByteSliceError>;
	fn try_append_u256_pair(&mut self, u256_value: U256) -> Result<(), FromByteSliceError>;
	fn try_append_fixedi128(&mut self, fixed_value: FixedI128) -> Result<(), FromByteSliceError>;
	fn try_append_asset(&mut self, asset: &Asset) -> Result<(), FromByteSliceError>;
	fn try_append_market(&mut self, market: &Market) -> Result<(), FromByteSliceError>;
	fn try_append_trading_account(
		&mut self,
		trading_account: &TradingAccountMinimal,
	) -> Result<(), FromByteSliceError>;
	fn try_append_settings(
		&mut self,
		settings: &BoundedVec<Setting, ConstU32<256>>,
	) -> Result<(), FromByteSliceError>;
	fn try_append_market_updated_event(
		&mut self,
		market_updated_event: &MarketUpdated,
	) -> Result<(), FromByteSliceError>;
	fn try_append_asset_updated_event(
		&mut self,
		asset_updated_event: &AssetUpdated,
	) -> Result<(), FromByteSliceError>;
	fn try_append_market_removed_event(
		&mut self,
		market_removed_event: &MarketRemoved,
	) -> Result<(), FromByteSliceError>;
	fn try_append_asset_removed_event(
		&mut self,
		asset_removed_event: &AssetRemoved,
	) -> Result<(), FromByteSliceError>;
	fn try_append_user_deposit_event(
		&mut self,
		user_deposit_event: &UserDeposit,
	) -> Result<(), FromByteSliceError>;
	fn try_append_signer_added_event(
		&mut self,
		signer_added: &SignerAdded,
	) -> Result<(), FromByteSliceError>;
	fn try_append_signer_removed_event(
		&mut self,
		signer_added: &SignerRemoved,
	) -> Result<(), FromByteSliceError>;
	fn try_append_settings_added_event(
		&mut self,
		settings_added: &SettingsAdded,
	) -> Result<(), FromByteSliceError>;
	fn try_append_universal_event_array(
		&mut self,
		universal_event_array: &Vec<UniversalEvent>,
	) -> Result<(), FromByteSliceError>;
}

pub trait ChainConstants {
	fn starknet_chain() -> u128;
	fn zkx_sync_chain() -> u128;
}
