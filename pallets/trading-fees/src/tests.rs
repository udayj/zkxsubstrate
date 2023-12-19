use crate::{mock::*, Event};
use frame_support::assert_ok;
use pallet_support::{
	test_helpers::asset_helper::usdc,
	types::{BaseFee, Side},
};
use sp_arithmetic::FixedI128;

fn setup() -> Vec<BaseFee> {
	// Set the assets in the system
	assert_ok!(Assets::replace_all_assets(RuntimeOrigin::signed(1), vec![usdc()]));

	let mut fee_details: Vec<BaseFee> = Vec::new();
	let base_fee1 = BaseFee {
		volume: 0.into(),
		maker_fee: FixedI128::from_inner(200000000000000),
		taker_fee: FixedI128::from_inner(500000000000000),
	};
	let base_fee2 = BaseFee {
		volume: 1000.into(),
		maker_fee: FixedI128::from_inner(150000000000000),
		taker_fee: FixedI128::from_inner(400000000000000),
	};
	let base_fee3 = BaseFee {
		volume: 5000.into(),
		maker_fee: FixedI128::from_inner(100000000000000),
		taker_fee: FixedI128::from_inner(350000000000000),
	};
	fee_details.push(base_fee1);
	fee_details.push(base_fee2);
	fee_details.push(base_fee3);

	fee_details
}

#[test]
fn test_update_fees() {
	new_test_ext().execute_with(|| {
		let fee_details = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		let side: Side = Side::Buy;
		// Dispatch a signed extrinsic.
		assert_ok!(TradingFeesModule::update_base_fees(
			RuntimeOrigin::signed(1),
			usdc().asset.id,
			side,
			fee_details.clone(),
		));

		assert_eq!(TradingFeesModule::max_base_fee_tier(usdc().asset.id, side), 3);
		let base_fee0 = TradingFeesModule::base_fee_tier(usdc().asset.id, (1, Side::Buy));
		assert_eq!(base_fee0, fee_details[0]);
		let base_fee1 = TradingFeesModule::base_fee_tier(usdc().asset.id, (2, Side::Buy));
		assert_eq!(base_fee1, fee_details[1]);
		let base_fee2 = TradingFeesModule::base_fee_tier(usdc().asset.id, (3, Side::Buy));
		assert_eq!(base_fee2, fee_details[2]);

		// Assert that the correct event was deposited
		System::assert_last_event(Event::BaseFeesUpdated { fee_tiers: 3 }.into());
	});
}

#[test]
#[should_panic(expected = "InvalidVolume")]
fn test_update_fees_with_invalid_volume() {
	new_test_ext().execute_with(|| {
		let mut fee_details = setup();

		let base_fee4 = BaseFee {
			volume: 100.into(),
			maker_fee: FixedI128::from_inner(100000000000000),
			taker_fee: FixedI128::from_inner(350000000000000),
		};
		fee_details.push(base_fee4);

		// Go past genesis block so events get deposited
		System::set_block_number(1);

		let side: Side = Side::Buy;
		// Dispatch a signed extrinsic.
		assert_ok!(TradingFeesModule::update_base_fees(
			RuntimeOrigin::signed(1),
			usdc().asset.id,
			side,
			fee_details.clone(),
		));
	});
}

#[test]
#[should_panic(expected = "InvalidFee")]
fn test_update_fees_with_invalid_fee() {
	new_test_ext().execute_with(|| {
		let mut fee_details = setup();

		let base_fee4 = BaseFee {
			volume: 10000.into(),
			maker_fee: FixedI128::from_inner(600000000000000),
			taker_fee: FixedI128::from_inner(750000000000000),
		};
		fee_details.push(base_fee4);

		// Go past genesis block so events get deposited
		System::set_block_number(1);

		let side: Side = Side::Buy;
		// Dispatch a signed extrinsic.
		assert_ok!(TradingFeesModule::update_base_fees(
			RuntimeOrigin::signed(1),
			usdc().asset.id,
			side,
			fee_details.clone(),
		));
	});
}

#[test]
#[should_panic(expected = "ZeroFeeTiers")]
fn test_update_fees_with_zero_fee_tiers() {
	new_test_ext().execute_with(|| {
		let fee_details = setup();

		// Go past genesis block so events get deposited
		System::set_block_number(1);

		let side: Side = Side::Buy;
		// Dispatch a signed extrinsic.
		assert_ok!(TradingFeesModule::update_base_fees(
			RuntimeOrigin::signed(1),
			usdc().asset.id,
			side,
			fee_details.clone(),
		));
		assert_eq!(TradingFeesModule::max_base_fee_tier(usdc().asset.id, side), 3);

		let fee_tiers: Vec<u8> = Vec::new();
		let fee_details: Vec<BaseFee> = Vec::new();

		// Dispatch a signed extrinsic.
		assert_ok!(TradingFeesModule::update_base_fees(
			RuntimeOrigin::signed(1),
			usdc().asset.id,
			side,
			fee_details.clone(),
		));

		assert_eq!(TradingFeesModule::max_base_fee_tier(usdc().asset.id, side), 0);
	});
}

#[test]
fn test_update_fees_with_multiple_calls() {
	new_test_ext().execute_with(|| {
		let fee_details = setup();

		// Go past genesis block so events get deposited
		System::set_block_number(1);

		let side: Side = Side::Buy;
		// Dispatch a signed extrinsic.
		assert_ok!(TradingFeesModule::update_base_fees(
			RuntimeOrigin::signed(1),
			usdc().asset.id,
			side,
			fee_details.clone(),
		));
		assert_eq!(TradingFeesModule::max_base_fee_tier(usdc().asset.id, side), 3);

		let fee_tiers: Vec<u8> = vec![1, 2, 3];
		let mut fee_details: Vec<BaseFee> = Vec::new();
		let base_fee1 = BaseFee {
			volume: 0.into(),
			maker_fee: FixedI128::from_inner(200000000000000),
			taker_fee: FixedI128::from_inner(500000000000000),
		};
		let base_fee2 = BaseFee {
			volume: 1000.into(),
			maker_fee: FixedI128::from_inner(150000000000000),
			taker_fee: FixedI128::from_inner(400000000000000),
		};
		let base_fee3 = BaseFee {
			volume: 3000.into(),
			maker_fee: FixedI128::from_inner(120000000000000),
			taker_fee: FixedI128::from_inner(300000000000000),
		};
		fee_details.push(base_fee1);
		fee_details.push(base_fee2);
		fee_details.push(base_fee3);

		// Dispatch a signed extrinsic.
		assert_ok!(TradingFeesModule::update_base_fees(
			RuntimeOrigin::signed(1),
			usdc().asset.id,
			side,
			fee_details.clone(),
		));

		assert_eq!(TradingFeesModule::max_base_fee_tier(usdc().asset.id, side), 3);
	});
}

#[test]
fn test_update_fees_both_sides() {
	new_test_ext().execute_with(|| {
		let fee_details = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		let side: Side = Side::Buy;
		// Dispatch a signed extrinsic.
		assert_ok!(TradingFeesModule::update_base_fees(
			RuntimeOrigin::signed(1),
			usdc().asset.id,
			side,
			fee_details.clone(),
		));

		// Assert that the correct event was deposited
		System::assert_last_event(Event::BaseFeesUpdated { fee_tiers: 3 }.into());

		let mut fee_details_sell: Vec<BaseFee> = Vec::new();
		let base_fee1 = BaseFee {
			volume: 0.into(),
			maker_fee: FixedI128::from_inner(250000000000000),
			taker_fee: FixedI128::from_inner(500000000000000),
		};
		let base_fee2 = BaseFee {
			volume: 1000.into(),
			maker_fee: FixedI128::from_inner(160000000000000),
			taker_fee: FixedI128::from_inner(410000000000000),
		};
		fee_details_sell.push(base_fee1);
		fee_details_sell.push(base_fee2);

		let side: Side = Side::Sell;
		// Dispatch a signed extrinsic.
		assert_ok!(TradingFeesModule::update_base_fees(
			RuntimeOrigin::signed(1),
			usdc().asset.id,
			side,
			fee_details_sell.clone(),
		));

		assert_eq!(TradingFeesModule::max_base_fee_tier(usdc().asset.id, Side::Buy), 3);
		let base_fee0 = TradingFeesModule::base_fee_tier(usdc().asset.id, (1, Side::Buy));
		assert_eq!(base_fee0, fee_details[0]);
		let base_fee1 = TradingFeesModule::base_fee_tier(usdc().asset.id, (2, Side::Buy));
		assert_eq!(base_fee1, fee_details[1]);
		let base_fee2 = TradingFeesModule::base_fee_tier(usdc().asset.id, (3, Side::Buy));
		assert_eq!(base_fee2, fee_details[2]);

		assert_eq!(TradingFeesModule::max_base_fee_tier(usdc().asset.id, Side::Sell), 2);
		let base_fee0 = TradingFeesModule::base_fee_tier(usdc().asset.id, (1, Side::Sell));
		assert_eq!(base_fee0, fee_details_sell[0]);
		let base_fee1 = TradingFeesModule::base_fee_tier(usdc().asset.id, (2, Side::Sell));
		assert_eq!(base_fee1, fee_details_sell[1]);
	});
}
