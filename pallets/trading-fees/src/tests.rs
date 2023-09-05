use crate::{mock::*, Event};
use frame_support::assert_ok;
use sp_arithmetic::FixedI128;
use zkx_support::types::{BaseFee, Discount, Side};

fn setup() -> (Vec<u8>, Vec<BaseFee>, Vec<u8>, Vec<Discount>) {
	let fee_tiers: Vec<u8> = vec![1, 2, 3];
	let mut fee_details: Vec<BaseFee> = Vec::new();
	let base_fee1 = BaseFee {
		number_of_tokens: 0.into(),
		maker_fee: FixedI128::from_inner(200000000000000),
		taker_fee: FixedI128::from_inner(500000000000000),
	};
	let base_fee2 = BaseFee {
		number_of_tokens: 1000.into(),
		maker_fee: FixedI128::from_inner(150000000000000),
		taker_fee: FixedI128::from_inner(400000000000000),
	};
	let base_fee3 = BaseFee {
		number_of_tokens: 5000.into(),
		maker_fee: FixedI128::from_inner(100000000000000),
		taker_fee: FixedI128::from_inner(350000000000000),
	};
	fee_details.push(base_fee1);
	fee_details.push(base_fee2);
	fee_details.push(base_fee3);

	let discount_tiers: Vec<u8> = vec![1, 2, 3, 4];
	let mut discount_details: Vec<Discount> = Vec::new();
	let discount1 =
		Discount { number_of_tokens: 0.into(), discount: FixedI128::from_inner(30000000000000000) };
	let discount2 = Discount {
		number_of_tokens: 1000.into(),
		discount: FixedI128::from_inner(50000000000000000),
	};
	let discount3 = Discount {
		number_of_tokens: 4000.into(),
		discount: FixedI128::from_inner(75000000000000000),
	};
	let discount4 = Discount {
		number_of_tokens: 7500.into(),
		discount: FixedI128::from_inner(100000000000000000),
	};
	discount_details.push(discount1);
	discount_details.push(discount2);
	discount_details.push(discount3);
	discount_details.push(discount4);

	(fee_tiers, fee_details, discount_tiers, discount_details)
}

#[test]
fn test_update_fees_and_discounts() {
	new_test_ext().execute_with(|| {
		let (fee_tiers, fee_details, discount_tiers, discount_details) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		let side: Side = Side::Buy;
		// Dispatch a signed extrinsic.
		assert_ok!(TradingFeesModule::update_base_fees_and_discounts(
			RuntimeOrigin::signed(1),
			side,
			fee_tiers,
			fee_details.clone(),
			discount_tiers,
			discount_details.clone()
		));

		assert_eq!(TradingFeesModule::max_base_fee_tier(), 3);
		let base_fee0 = TradingFeesModule::base_fee_tier(1, Side::Buy);
		assert_eq!(base_fee0, fee_details[0]);
		let base_fee1 = TradingFeesModule::base_fee_tier(2, Side::Buy);
		assert_eq!(base_fee1, fee_details[1]);
		let base_fee2 = TradingFeesModule::base_fee_tier(3, Side::Buy);
		assert_eq!(base_fee2, fee_details[2]);

		assert_eq!(TradingFeesModule::max_discount_tier(), 4);
		let discount0 = TradingFeesModule::discount_tier(1, Side::Buy);
		assert_eq!(discount0, discount_details[0]);
		let discount1 = TradingFeesModule::discount_tier(2, Side::Buy);
		assert_eq!(discount1, discount_details[1]);
		let discount2 = TradingFeesModule::discount_tier(3, Side::Buy);
		assert_eq!(discount2, discount_details[2]);
		let discount3 = TradingFeesModule::discount_tier(4, Side::Buy);
		assert_eq!(discount3, discount_details[3]);

		// Assert that the correct event was deposited
		System::assert_last_event(
			Event::BaseFeesAndDiscountsUpdated { fee_tiers: 3, discount_tiers: 4 }.into(),
		);
	});
}

#[test]
#[should_panic(expected = "InvalidNumberOfTokens")]
fn test_update_fees_with_invalid_number_of_tokens() {
	new_test_ext().execute_with(|| {
		let (mut fee_tiers, mut fee_details, discount_tiers, discount_details) = setup();
		fee_tiers.push(4);
		let base_fee4 = BaseFee {
			number_of_tokens: 100.into(),
			maker_fee: FixedI128::from_inner(100000000000000),
			taker_fee: FixedI128::from_inner(350000000000000),
		};
		fee_details.push(base_fee4);

		// Go past genesis block so events get deposited
		System::set_block_number(1);

		let side: Side = Side::Buy;
		// Dispatch a signed extrinsic.
		assert_ok!(TradingFeesModule::update_base_fees_and_discounts(
			RuntimeOrigin::signed(1),
			side,
			fee_tiers,
			fee_details.clone(),
			discount_tiers,
			discount_details.clone()
		));
	});
}

#[test]
#[should_panic(expected = "InvalidFee")]
fn test_update_fees_with_invalid_fee() {
	new_test_ext().execute_with(|| {
		let (mut fee_tiers, mut fee_details, discount_tiers, discount_details) = setup();
		fee_tiers.push(4);
		let base_fee4 = BaseFee {
			number_of_tokens: 10000.into(),
			maker_fee: FixedI128::from_inner(600000000000000),
			taker_fee: FixedI128::from_inner(750000000000000),
		};
		fee_details.push(base_fee4);

		// Go past genesis block so events get deposited
		System::set_block_number(1);

		let side: Side = Side::Buy;
		// Dispatch a signed extrinsic.
		assert_ok!(TradingFeesModule::update_base_fees_and_discounts(
			RuntimeOrigin::signed(1),
			side,
			fee_tiers,
			fee_details.clone(),
			discount_tiers,
			discount_details.clone()
		));
	});
}

#[test]
#[should_panic(expected = "InvalidTier")]
fn test_update_fees_with_invalid_tier() {
	new_test_ext().execute_with(|| {
		let (mut fee_tiers, mut fee_details, discount_tiers, discount_details) = setup();
		fee_tiers.push(5);
		let base_fee4 = BaseFee {
			number_of_tokens: 10000.into(),
			maker_fee: FixedI128::from_inner(600000000000000),
			taker_fee: FixedI128::from_inner(750000000000000),
		};
		fee_details.push(base_fee4);

		// Go past genesis block so events get deposited
		System::set_block_number(1);

		let side: Side = Side::Buy;
		// Dispatch a signed extrinsic.
		assert_ok!(TradingFeesModule::update_base_fees_and_discounts(
			RuntimeOrigin::signed(1),
			side,
			fee_tiers,
			fee_details.clone(),
			discount_tiers,
			discount_details.clone()
		));
	});
}

#[test]
#[should_panic(expected = "InvalidNumberOfTokens")]
fn test_update_discount_with_invalid_number_of_tokens() {
	new_test_ext().execute_with(|| {
		let (fee_tiers, fee_details, mut discount_tiers, mut discount_details) = setup();
		discount_tiers.push(5);
		let discount5 = Discount {
			number_of_tokens: 700.into(),
			discount: FixedI128::from_inner(100000000000000000),
		};
		discount_details.push(discount5);

		// Go past genesis block so events get deposited
		System::set_block_number(1);

		let side: Side = Side::Buy;
		// Dispatch a signed extrinsic.
		assert_ok!(TradingFeesModule::update_base_fees_and_discounts(
			RuntimeOrigin::signed(1),
			side,
			fee_tiers,
			fee_details.clone(),
			discount_tiers,
			discount_details.clone()
		));
	});
}

#[test]
#[should_panic(expected = "InvalidDiscount")]
fn test_update_discount_with_invalid_discount() {
	new_test_ext().execute_with(|| {
		let (fee_tiers, fee_details, mut discount_tiers, mut discount_details) = setup();
		discount_tiers.push(5);
		let discount5 = Discount {
			number_of_tokens: 10000.into(),
			discount: FixedI128::from_inner(10000000000000000),
		};
		discount_details.push(discount5);

		// Go past genesis block so events get deposited
		System::set_block_number(1);

		let side: Side = Side::Buy;
		// Dispatch a signed extrinsic.
		assert_ok!(TradingFeesModule::update_base_fees_and_discounts(
			RuntimeOrigin::signed(1),
			side,
			fee_tiers,
			fee_details.clone(),
			discount_tiers,
			discount_details.clone()
		));
	});
}

#[test]
#[should_panic(expected = "FeeTiersLengthMismatch")]
fn test_update_fees_with_tiers_length_mismatch() {
	new_test_ext().execute_with(|| {
		let (fee_tiers, mut fee_details, discount_tiers, discount_details) = setup();
		let base_fee4 = BaseFee {
			number_of_tokens: 10000.into(),
			maker_fee: FixedI128::from_inner(600000000000000),
			taker_fee: FixedI128::from_inner(750000000000000),
		};
		fee_details.push(base_fee4);

		// Go past genesis block so events get deposited
		System::set_block_number(1);

		let side: Side = Side::Buy;
		// Dispatch a signed extrinsic.
		assert_ok!(TradingFeesModule::update_base_fees_and_discounts(
			RuntimeOrigin::signed(1),
			side,
			fee_tiers,
			fee_details.clone(),
			discount_tiers,
			discount_details.clone()
		));
	});
}

#[test]
#[should_panic(expected = "DiscountTiersLengthMismatch")]
fn test_update_discount_with_tiers_length_mismatch() {
	new_test_ext().execute_with(|| {
		let (fee_tiers, fee_details, discount_tiers, mut discount_details) = setup();
		let discount5 = Discount {
			number_of_tokens: 700.into(),
			discount: FixedI128::from_inner(100000000000000000),
		};
		discount_details.push(discount5);

		// Go past genesis block so events get deposited
		System::set_block_number(1);

		let side: Side = Side::Buy;
		// Dispatch a signed extrinsic.
		assert_ok!(TradingFeesModule::update_base_fees_and_discounts(
			RuntimeOrigin::signed(1),
			side,
			fee_tiers,
			fee_details.clone(),
			discount_tiers,
			discount_details.clone()
		));
	});
}

#[test]
#[should_panic(expected = "ZeroFeeTiers")]
fn test_update_fees_and_discounts_with_zero_fee_tiers() {
	new_test_ext().execute_with(|| {
		let (fee_tiers, fee_details, discount_tiers, discount_details) = setup();

		// Go past genesis block so events get deposited
		System::set_block_number(1);

		let side: Side = Side::Buy;
		// Dispatch a signed extrinsic.
		assert_ok!(TradingFeesModule::update_base_fees_and_discounts(
			RuntimeOrigin::signed(1),
			side,
			fee_tiers,
			fee_details.clone(),
			discount_tiers,
			discount_details.clone()
		));
		assert_eq!(TradingFeesModule::max_base_fee_tier(), 3);
		assert_eq!(TradingFeesModule::max_discount_tier(), 4);

		let fee_tiers: Vec<u8> = Vec::new();
		let fee_details: Vec<BaseFee> = Vec::new();
		let discount_tiers: Vec<u8> = vec![1];
		let mut discount_details: Vec<Discount> = Vec::new();
		let discount = Discount {
			number_of_tokens: 700.into(),
			discount: FixedI128::from_inner(100000000000000000),
		};
		discount_details.push(discount);

		// Dispatch a signed extrinsic.
		assert_ok!(TradingFeesModule::update_base_fees_and_discounts(
			RuntimeOrigin::signed(1),
			side,
			fee_tiers,
			fee_details.clone(),
			discount_tiers,
			discount_details.clone()
		));

		assert_eq!(TradingFeesModule::max_base_fee_tier(), 0);
		assert_eq!(TradingFeesModule::max_discount_tier(), 1);
	});
}

#[test]
fn test_update_fees_and_discounts_with_multiple_calls() {
	new_test_ext().execute_with(|| {
		let (fee_tiers, fee_details, discount_tiers, discount_details) = setup();

		// Go past genesis block so events get deposited
		System::set_block_number(1);

		let side: Side = Side::Buy;
		// Dispatch a signed extrinsic.
		assert_ok!(TradingFeesModule::update_base_fees_and_discounts(
			RuntimeOrigin::signed(1),
			side,
			fee_tiers,
			fee_details.clone(),
			discount_tiers,
			discount_details.clone()
		));
		assert_eq!(TradingFeesModule::max_base_fee_tier(), 3);
		assert_eq!(TradingFeesModule::max_discount_tier(), 4);

		let fee_tiers: Vec<u8> = vec![1, 2];
		let mut fee_details: Vec<BaseFee> = Vec::new();
		let base_fee1 = BaseFee {
			number_of_tokens: 0.into(),
			maker_fee: FixedI128::from_inner(200000000000000),
			taker_fee: FixedI128::from_inner(500000000000000),
		};
		let base_fee2 = BaseFee {
			number_of_tokens: 1000.into(),
			maker_fee: FixedI128::from_inner(150000000000000),
			taker_fee: FixedI128::from_inner(400000000000000),
		};
		fee_details.push(base_fee1);
		fee_details.push(base_fee2);

		let discount_tiers: Vec<u8> = vec![1];
		let mut discount_details: Vec<Discount> = Vec::new();
		let discount = Discount {
			number_of_tokens: 700.into(),
			discount: FixedI128::from_inner(100000000000000000),
		};
		discount_details.push(discount);

		// Dispatch a signed extrinsic.
		assert_ok!(TradingFeesModule::update_base_fees_and_discounts(
			RuntimeOrigin::signed(1),
			side,
			fee_tiers,
			fee_details.clone(),
			discount_tiers,
			discount_details.clone()
		));

		assert_eq!(TradingFeesModule::max_base_fee_tier(), 2);
		assert_eq!(TradingFeesModule::max_discount_tier(), 1);
	});
}
