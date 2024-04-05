use crate::{mock::*, Event};
use frame_support::assert_ok;
use pallet_support::{
	test_helpers::{asset_helper::usdc, btc, btc_usdc, eth_usdc, link},
	traits::TradingFeesInterface,
	types::BaseFeeAggregate,
};

// declare test_helper module
pub mod test_helper;
use sp_arithmetic::FixedI128;
use sp_runtime::traits::Zero;
use test_helper::*;

fn setup() {
	// Set the assets in the system
	assert_ok!(Assets::replace_all_assets(
		RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
		vec![usdc(), btc(), link()]
	));
	assert_ok!(Markets::replace_all_markets(
		RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
		vec![btc_usdc()]
	));
}

#[test]
fn test_update_fees() {
	new_test_ext().execute_with(|| {
		setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		let expected_fees = get_usdc_aggregate_fees();

		// Dispatch a signed extrinsic.
		assert_ok!(TradingFeesModule::update_base_fees(
			RuntimeOrigin::root(),
			usdc().asset.id,
			expected_fees.clone()
		));

		// Check the state
		assert_eq!(TradingFeesModule::get_all_fees(0_u128, usdc().asset.id), expected_fees);

		// Assert that the correct event was deposited
		System::assert_last_event(
			Event::BaseFeeAggregateSet { id: usdc().asset.id, base_fee_aggregate: expected_fees }
				.into(),
		);
	});
}

#[test]
fn test_update_fee_shares() {
	let usdc_id = usdc().asset.id;
	new_test_ext().execute_with(|| {
		setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		// Before setting the fee share values
		// fetch fee_shares for different levels and volumes of a user
		assert!(
			TradingFeesModule::get_fee_share(0, usdc_id, FixedI128::zero()) == FixedI128::zero()
		);
		assert!(
			TradingFeesModule::get_fee_share(0, usdc_id, FixedI128::from_u32(200001)) ==
				FixedI128::zero()
		);
		assert!(
			TradingFeesModule::get_fee_share(0, usdc_id, FixedI128::from_u32(5000001)) ==
				FixedI128::zero()
		);
		assert!(
			TradingFeesModule::get_fee_share(0, usdc_id, FixedI128::from_u32(10000001)) ==
				FixedI128::zero()
		);
		assert!(
			TradingFeesModule::get_fee_share(0, usdc_id, FixedI128::from_u32(25000001)) ==
				FixedI128::zero()
		);
		assert!(
			TradingFeesModule::get_fee_share(0, usdc_id, FixedI128::from_u32(50000001)) ==
				FixedI128::zero()
		);
		assert!(
			TradingFeesModule::get_fee_share(0, usdc_id, FixedI128::from_u32(49999999)) ==
				FixedI128::zero()
		);
		assert!(
			TradingFeesModule::get_fee_share(0, usdc_id, FixedI128::from_u32(24999999)) ==
				FixedI128::zero()
		);
		assert!(
			TradingFeesModule::get_fee_share(0, usdc_id, FixedI128::from_u32(9999999)) ==
				FixedI128::zero()
		);
		assert!(
			TradingFeesModule::get_fee_share(0, usdc_id, FixedI128::from_u32(4999999)) ==
				FixedI128::zero()
		);
		assert!(
			TradingFeesModule::get_fee_share(0, usdc_id, FixedI128::from_u32(199999)) ==
				FixedI128::zero()
		);

		let expected_fees = get_usdc_fee_shares();

		// Dispatch a signed extrinsic.
		assert_ok!(TradingFeesModule::update_fee_share(
			RuntimeOrigin::root(),
			usdc().asset.id,
			expected_fees.clone()
		));

		// Check the state
		assert_eq!(TradingFeesModule::get_all_fee_shares(usdc().asset.id), expected_fees);

		// Assert that the correct event was deposited
		System::assert_last_event(Event::FeeShareSet { fee_share: expected_fees }.into());

		// fetch fee_shares for different levels and volumes of a user
		assert!(
			TradingFeesModule::get_fee_share(0, usdc_id, FixedI128::zero()) == FixedI128::zero()
		);
		assert!(
			TradingFeesModule::get_fee_share(0, usdc_id, FixedI128::from_u32(200001)) ==
				FixedI128::from_float(0.05)
		);
		assert!(
			TradingFeesModule::get_fee_share(0, usdc_id, FixedI128::from_u32(5000001)) ==
				FixedI128::from_float(0.08)
		);
		assert!(
			TradingFeesModule::get_fee_share(0, usdc_id, FixedI128::from_u32(10000001)) ==
				FixedI128::from_float(0.1)
		);
		assert!(
			TradingFeesModule::get_fee_share(0, usdc_id, FixedI128::from_u32(25000001)) ==
				FixedI128::from_float(0.12)
		);
		assert!(
			TradingFeesModule::get_fee_share(0, usdc_id, FixedI128::from_u32(50000001)) ==
				FixedI128::from_float(0.15)
		);
		assert!(
			TradingFeesModule::get_fee_share(0, usdc_id, FixedI128::from_u32(49999999)) ==
				FixedI128::from_float(0.12)
		);
		assert!(
			TradingFeesModule::get_fee_share(0, usdc_id, FixedI128::from_u32(24999999)) ==
				FixedI128::from_float(0.1)
		);
		assert!(
			TradingFeesModule::get_fee_share(0, usdc_id, FixedI128::from_u32(9999999)) ==
				FixedI128::from_float(0.08)
		);
		assert!(
			TradingFeesModule::get_fee_share(0, usdc_id, FixedI128::from_u32(4999999)) ==
				FixedI128::from_float(0.05)
		);
		assert!(
			TradingFeesModule::get_fee_share(0, usdc_id, FixedI128::from_u32(199999)) ==
				FixedI128::zero()
		);

		// fetch fee_shares for different levels and volumes of a user
		assert!(
			TradingFeesModule::get_fee_share(1, usdc_id, FixedI128::zero()) == FixedI128::zero()
		);
		assert!(
			TradingFeesModule::get_fee_share(1, usdc_id, FixedI128::from_u32(200001)) ==
				FixedI128::from_float(0.5)
		);
		assert!(
			TradingFeesModule::get_fee_share(1, usdc_id, FixedI128::from_u32(199999)) ==
				FixedI128::zero()
		);

		// fetch fees_shares for user level > fee share level
		assert!(
			TradingFeesModule::get_fee_share(2, usdc_id, FixedI128::zero()) == FixedI128::zero()
		);
		assert!(
			TradingFeesModule::get_fee_share(2, usdc_id, FixedI128::from_u32(200001)) ==
				FixedI128::zero()
		);
		assert!(
			TradingFeesModule::get_fee_share(2, usdc_id, FixedI128::from_u32(199999)) ==
				FixedI128::zero()
		);
	});
}

#[test]
fn test_update_market_fees() {
	new_test_ext().execute_with(|| {
		setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		let expected_fees = get_btc_usdc_aggregate_fees();

		// Dispatch a signed extrinsic.
		assert_ok!(TradingFeesModule::update_base_fees(
			RuntimeOrigin::root(),
			btc_usdc().market.id,
			expected_fees.clone(),
		));

		// Check the state
		assert_eq!(
			TradingFeesModule::get_all_fees(btc_usdc().market.id, usdc().asset.id),
			expected_fees
		);

		// Assert that the correct event was deposited
		System::assert_last_event(
			Event::BaseFeeAggregateSet {
				id: btc_usdc().market.id,
				base_fee_aggregate: expected_fees,
			}
			.into(),
		);
	});
}

#[test]
fn test_update_market_fees_0() {
	new_test_ext().execute_with(|| {
		setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		let expected_fees = get_0_aggregate_fees();

		// Dispatch a signed extrinsic.
		assert_ok!(TradingFeesModule::update_base_fees(
			RuntimeOrigin::root(),
			btc_usdc().market.id,
			expected_fees.clone(),
		));

		// Check the state
		assert_eq!(
			TradingFeesModule::get_all_fees(btc_usdc().market.id, usdc().asset.id),
			expected_fees
		);

		// Assert that the correct event was deposited
		System::assert_last_event(
			Event::BaseFeeAggregateSet {
				id: btc_usdc().market.id,
				base_fee_aggregate: expected_fees,
			}
			.into(),
		);
	});
}

#[test]
#[should_panic(expected = "MarketNotFound")]
fn test_update_market_fees_invalid_market() {
	new_test_ext().execute_with(|| {
		setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		// Dispatch a signed extrinsic.
		assert_ok!(TradingFeesModule::update_base_fees(
			RuntimeOrigin::root(),
			eth_usdc().market.id,
			get_btc_usdc_aggregate_fees(),
		));
	});
}

#[test]
#[should_panic(expected = "AssetNotCollateral")]
fn test_update_fees_invalid_collateral() {
	new_test_ext().execute_with(|| {
		setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		// Dispatch a signed extrinsic.
		assert_ok!(TradingFeesModule::update_base_fees(
			RuntimeOrigin::root(),
			link().asset.id,
			get_usdc_aggregate_fees()
		));
	});
}

#[test]
#[should_panic(expected = "InvalidVolume")]
fn test_update_fees_with_invalid_volume() {
	new_test_ext().execute_with(|| {
		setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		// Dispatch a signed extrinsic.
		assert_ok!(TradingFeesModule::update_base_fees(
			RuntimeOrigin::root(),
			usdc().asset.id,
			get_invalid_aggregate_volume()
		));
	});
}

#[test]
#[should_panic(expected = "InvalidFee")]
fn test_update_fees_with_invalid_fee() {
	new_test_ext().execute_with(|| {
		setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		// Dispatch a signed extrinsic.
		assert_ok!(TradingFeesModule::update_base_fees(
			RuntimeOrigin::root(),
			usdc().asset.id,
			get_invalid_aggregate_fee()
		));
	});
}

#[test]
#[should_panic(expected = "ZeroFeeTiers")]
fn test_update_fees_with_zero_fee_tiers() {
	new_test_ext().execute_with(|| {
		setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		// Get usdc fees
		let fees = get_usdc_aggregate_fees();

		// Dispatch a signed extrinsic.
		assert_ok!(TradingFeesModule::update_base_fees(
			RuntimeOrigin::root(),
			usdc().asset.id,
			BaseFeeAggregate {
				maker_buy: fees.maker_buy,
				maker_sell: fees.maker_sell,
				taker_buy: vec![],
				taker_sell: fees.taker_sell
			},
		));
	});
}

#[test]
fn test_update_fees_with_multiple_calls() {
	new_test_ext().execute_with(|| {
		setup();

		// Go past genesis block so events get deposited
		System::set_block_number(1);

		// Expect fees
		let fees_1 = get_btc_usdc_aggregate_fees();
		let fees_2: BaseFeeAggregate = get_0_aggregate_fees();

		// Dispatch a signed extrinsic.
		assert_ok!(TradingFeesModule::update_base_fees(
			RuntimeOrigin::root(),
			usdc().asset.id,
			fees_1.clone(),
		));

		// Check the state
		assert_eq!(TradingFeesModule::get_all_fees(0_u128, usdc().asset.id), fees_1);

		// Assert that the correct event was deposited
		System::assert_last_event(
			Event::BaseFeeAggregateSet { id: usdc().asset.id, base_fee_aggregate: fees_1 }.into(),
		);

		// Dispatch a signed extrinsic to replace the previously set fees
		assert_ok!(TradingFeesModule::update_base_fees(
			RuntimeOrigin::root(),
			usdc().asset.id,
			fees_2.clone(),
		));

		// Check the state
		assert_eq!(TradingFeesModule::get_all_fees(0_u128, usdc().asset.id), fees_2);

		// Assert that the correct event was deposited
		System::assert_last_event(
			Event::BaseFeeAggregateSet { id: usdc().asset.id, base_fee_aggregate: fees_2 }.into(),
		);
	});
}
