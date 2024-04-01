use crate::{mock::*, Event};
use frame_support::assert_ok;
use pallet_support::{
	test_helpers::{
		accounts_helper::{
			alice, bob, charlie, create_withdrawal_request, dave, eduard, get_private_key,
			get_trading_account_id,
		},
		asset_helper::{btc, eth, link, usdc, usdt},
		market_helper::{btc_usdc, link_usdc},
	},
	traits::TradingAccountInterface,
	types::{
		trading::{Direction, OrderType},
		BalanceUpdate, FundModifyType, MonetaryAccountDetails, Order,
	},
};
use primitive_types::U256;
use sp_arithmetic::FixedI128;

fn setup() -> sp_io::TestExternalities {
	// Create a new test environment
	let mut env = new_test_ext();

	// Set the block number in the environment
	env.execute_with(|| {
		// Set the block number
		System::set_block_number(1);
		assert_ok!(Timestamp::set(None.into(), 1699940367000));
		// Set the assets in the system
		assert_ok!(Assets::replace_all_assets(
			RuntimeOrigin::signed(1),
			vec![eth(), usdc(), link(), btc(), usdt()]
		));
		assert_ok!(Markets::replace_all_markets(
			RuntimeOrigin::signed(1),
			vec![btc_usdc(), link_usdc()]
		));

		// Add accounts to the system
		assert_ok!(TradingAccountModule::add_accounts(
			RuntimeOrigin::signed(1),
			vec![alice(), bob(), charlie(), dave()]
		));
	});

	env
}

#[test]
fn test_add_accounts() {
	let mut env = setup();

	env.execute_with(|| {
		// Check the state of the env
		// There must be 4 accounts
		assert_eq!(TradingAccountModule::accounts_count(), 4);

		// Get the trading account of Alice
		let alice_account_id = get_trading_account_id(alice());
		let alice_fetched_account = TradingAccountModule::accounts(alice_account_id)
			.unwrap()
			.to_trading_account_minimal();
		assert_eq!(alice_fetched_account, alice());

		// Check the balance of Alice
		let alice_balance = TradingAccountModule::balances(alice_account_id, usdc().asset.id);
		assert!(alice_balance == 10000.into());

		// Get the trading account of Bob
		let bob_account_id = get_trading_account_id(bob());
		let bob_fetched_account = TradingAccountModule::accounts(bob_account_id)
			.unwrap()
			.to_trading_account_minimal();
		assert_eq!(bob_fetched_account, bob());

		// Check the balance of Bob
		let bob_balance = TradingAccountModule::balances(bob_account_id, usdc().asset.id);
		assert!(bob_balance == 10000.into());

		// Get the trading account of Charlie
		let charlie_account_id = get_trading_account_id(charlie());
		let charlie_fetched_account = TradingAccountModule::accounts(charlie_account_id)
			.unwrap()
			.to_trading_account_minimal();
		assert_eq!(charlie_fetched_account, charlie());

		// Check the balance of Charlie
		let charlie_balance = TradingAccountModule::balances(charlie_account_id, usdc().asset.id);
		assert!(charlie_balance == 10000.into());

		// Get the trading account of Dave
		let dave_account_id = get_trading_account_id(dave());
		let dave_fetched_account = TradingAccountModule::accounts(dave_account_id)
			.unwrap()
			.to_trading_account_minimal();
		assert_eq!(dave_fetched_account, dave());

		// Check the balance of Dave
		let dave_balance = TradingAccountModule::balances(dave_account_id, usdc().asset.id);
		assert!(dave_balance == 10000.into());
	});
}

#[test]
fn test_update_monetary_account_to_trading_accounts_map() {
	let mut env = setup();

	env.execute_with(|| {
		// Get the trading account of Alice and bob
		let alice_account_id = get_trading_account_id(alice());
		let bob_account_id = get_trading_account_id(bob());

		// Add alice to the list
		let mut trading_accounts = vec![alice_account_id];

		// update update monetary to trading accounts map
		let account_details = MonetaryAccountDetails {
			monetary_account: U256::from(100_u8),
			trading_accounts: trading_accounts.clone(),
		};
		assert_ok!(TradingAccountModule::update_monetary_to_trading_accounts(
			RuntimeOrigin::root(),
			vec![account_details]
		));
		let trading_accounts_list =
			TradingAccountModule::monetary_to_trading_accounts(U256::from(100_u8));

		assert_eq!(alice_account_id, trading_accounts_list[0]);

		// Add bob to the list
		trading_accounts.push(bob_account_id);

		// update update monetary to trading accounts map
		let account_details = MonetaryAccountDetails {
			monetary_account: U256::from(100_u8),
			trading_accounts: trading_accounts.clone(),
		};
		assert_ok!(TradingAccountModule::update_monetary_to_trading_accounts(
			RuntimeOrigin::root(),
			vec![account_details]
		));
		let trading_accounts_list =
			TradingAccountModule::monetary_to_trading_accounts(U256::from(100_u8));

		assert_eq!(alice_account_id, trading_accounts_list[0]);
		assert_eq!(bob_account_id, trading_accounts_list[1]);

		// Try adding bob_account_id to the list again
		trading_accounts.push(bob_account_id);

		// update update monetary to trading accounts map
		let account_details = MonetaryAccountDetails {
			monetary_account: U256::from(100_u8),
			trading_accounts: trading_accounts.clone(),
		};
		assert_ok!(TradingAccountModule::update_monetary_to_trading_accounts(
			RuntimeOrigin::root(),
			vec![account_details]
		));
		let trading_accounts_list =
			TradingAccountModule::monetary_to_trading_accounts(U256::from(100_u8));

		assert_eq!(alice_account_id, trading_accounts_list[0]);
		assert_eq!(bob_account_id, trading_accounts_list[1]);
		assert_eq!(2, trading_accounts_list.len());
	});
}

#[test]
#[should_panic(expected = "AssetNotFound")]
fn test_add_balances_with_unknown_asset() {
	let mut env = setup();

	env.execute_with(|| {
		// Get trading account of Alice
		let trading_account_id = get_trading_account_id(alice());

		// Dispatch a signed extrinsic.
		assert_ok!(TradingAccountModule::set_balances(
			RuntimeOrigin::signed(1),
			trading_account_id,
			vec![BalanceUpdate { asset_id: 1234567, balance_value: 1000.into() }]
		));
	});
}

#[test]
#[should_panic(expected = "AssetNotCollateral")]
fn test_add_balances_with_asset_not_marked_as_collateral() {
	let mut env = setup();

	env.execute_with(|| {
		// Get the trading id of alice
		let trading_account_id = get_trading_account_id(alice());

		// Dispatch a signed extrinsic.
		assert_ok!(TradingAccountModule::set_balances(
			RuntimeOrigin::signed(1),
			trading_account_id,
			vec![BalanceUpdate { asset_id: eth().asset.id, balance_value: 1000.into() }],
		));
	});
}

#[test]
fn test_add_balances() {
	let mut env = setup();

	env.execute_with(|| {
		// Get the trading account of Alice
		let trading_account_id = get_trading_account_id(alice());
		let balances_array = vec![
			BalanceUpdate { asset_id: usdc().asset.id, balance_value: 1000.into() },
			BalanceUpdate { asset_id: usdt().asset.id, balance_value: 500.into() },
		];

		// Dispatch a signed extrinsic.
		assert_ok!(TradingAccountModule::set_balances(
			RuntimeOrigin::signed(1),
			trading_account_id,
			balances_array
		));

		// Check the state
		assert_eq!(
			TradingAccountModule::balances(trading_account_id, usdc().asset.id),
			1000.into()
		);
		assert_eq!(TradingAccountModule::balances(trading_account_id, usdt().asset.id), 500.into());
		assert_eq!(
			TradingAccountModule::account_collaterals(trading_account_id),
			vec![usdc().asset.id, usdt().asset.id]
		);
	});
}

#[test]
fn test_deposit() {
	let mut env = setup();

	env.execute_with(|| {
		// Get the trading account of Alice
		let trading_account_id = get_trading_account_id(alice());

		// Dispatch a signed extrinsic.
		assert_ok!(TradingAccountModule::deposit(
			RuntimeOrigin::signed(1),
			alice(),
			usdc().asset.id,
			1000.into(),
		));

		// Check the state
		assert_eq!(
			TradingAccountModule::balances(trading_account_id, usdc().asset.id),
			11000.into()
		);
	});
}

#[test]
fn test_deposit_when_negative() {
	let mut env = setup();

	env.execute_with(|| {
		// Get the trading account of Alice
		let trading_account_id = get_trading_account_id(alice());

		// Dispatch a signed extrinsic.
		assert_ok!(TradingAccountModule::set_balances(
			RuntimeOrigin::signed(1),
			trading_account_id,
			vec![BalanceUpdate { asset_id: usdc().asset.id, balance_value: (-250).into() }],
		));

		// Check the state
		assert_eq!(
			TradingAccountModule::balances(trading_account_id, usdc().asset.id),
			(-250).into()
		);

		// Desposit 100 USDC
		assert_ok!(TradingAccountModule::deposit(
			RuntimeOrigin::signed(1),
			alice(),
			usdc().asset.id,
			100.into(),
		));

		// Check the state
		assert_eq!(
			TradingAccountModule::balances(trading_account_id, usdc().asset.id),
			(-150).into()
		);

		// Check the InsuranceFundChange event
		System::assert_has_event(
			Event::InsuranceFundChange {
				collateral_id: usdc().asset.id,
				amount: 100.into(),
				modify_type: FundModifyType::Increase,
				block_number: 1,
			}
			.into(),
		);

		// Desposit 160 USDC
		assert_ok!(TradingAccountModule::deposit(
			RuntimeOrigin::signed(1),
			alice(),
			usdc().asset.id,
			160.into(),
		));

		// Check the state
		assert_eq!(TradingAccountModule::balances(trading_account_id, usdc().asset.id), 10.into());

		// Check the InsuranceFundChange event
		System::assert_has_event(
			Event::InsuranceFundChange {
				collateral_id: usdc().asset.id,
				amount: 150.into(),
				modify_type: FundModifyType::Increase,
				block_number: 1,
			}
			.into(),
		);
	});
}

#[test]
fn test_withdraw() {
	let mut env = setup();

	env.execute_with(|| {
		// Get the trading account of Alice and create a withdrawal request
		let trading_account_id = get_trading_account_id(alice());
		let withdrawal_request = create_withdrawal_request(
			trading_account_id,
			usdc().asset.id,
			1000.into(),
			1697733033397,
			get_private_key(alice().pub_key),
		)
		.unwrap();

		// Dispatch a signed extrinsic.
		assert_ok!(TradingAccountModule::withdraw(RuntimeOrigin::signed(1), withdrawal_request));

		assert_eq!(
			TradingAccountModule::balances(trading_account_id, usdc().asset.id),
			9000.into()
		);
		let event_record: frame_system::EventRecord<_, _> = System::events().pop().unwrap();
		println!("Events: {:?}", event_record);
	});
}

#[test]
fn test_withdraw_twice() {
	let mut env = setup();

	env.execute_with(|| {
		// Get the trading account of Alice and create a withdrawal request
		let trading_account_id = get_trading_account_id(alice());
		let withdrawal_request = create_withdrawal_request(
			trading_account_id,
			usdc().asset.id,
			1000.into(),
			1697733033397,
			get_private_key(alice().pub_key),
		)
		.unwrap();

		// Send the withdrawal request
		assert_ok!(TradingAccountModule::withdraw(RuntimeOrigin::signed(1), withdrawal_request));

		// Check the state
		assert_eq!(
			TradingAccountModule::balances(trading_account_id, usdc().asset.id),
			9000.into()
		);

		// Create a new withdrawal request
		let withdrawal_request_2 = create_withdrawal_request(
			trading_account_id,
			usdc().asset.id,
			1000.into(),
			1697733033400,
			get_private_key(alice().pub_key),
		)
		.unwrap();

		// Send the new withdrawal request
		assert_ok!(TradingAccountModule::withdraw(RuntimeOrigin::signed(1), withdrawal_request_2));

		// Check the state
		assert_eq!(
			TradingAccountModule::balances(trading_account_id, usdc().asset.id),
			8000.into()
		);
	});
}

#[test]
#[should_panic(expected = "DuplicateWithdrawal")]
fn test_withdraw_duplicate() {
	let mut env = setup();

	env.execute_with(|| {
		// Get the trading account of Alice and create a withdrawal request
		let trading_account_id = get_trading_account_id(alice());
		let withdrawal_request = create_withdrawal_request(
			trading_account_id,
			usdc().asset.id,
			1000.into(),
			1697733033397,
			get_private_key(alice().pub_key),
		)
		.unwrap();

		// Send the withdrawal request
		assert_ok!(TradingAccountModule::withdraw(
			RuntimeOrigin::signed(1),
			withdrawal_request.clone()
		));

		// Send the withdrawal request again
		assert_ok!(TradingAccountModule::withdraw(
			RuntimeOrigin::signed(1),
			withdrawal_request.clone()
		));
	});
}

#[test]
#[should_panic(expected = "AccountDoesNotExist")]
fn test_withdraw_on_not_existing_account() {
	let mut env = setup();

	env.execute_with(|| {
		// Get the trading account of Alice and create a withdrawal request
		let trading_account_id = get_trading_account_id(eduard());

		let withdrawal_request = create_withdrawal_request(
			trading_account_id,
			usdc().asset.id,
			1000.into(),
			1697733073513,
			get_private_key(eduard().pub_key),
		)
		.unwrap();

		// Dispatch a signed extrinsic.
		assert_ok!(TradingAccountModule::withdraw(RuntimeOrigin::signed(1), withdrawal_request));
	});
}

#[test]
#[should_panic(expected = "InvalidSignature")]
fn test_withdraw_on_invalid_sig() {
	let mut env = setup();

	env.execute_with(|| {
		// Get the trading account of Alice and create a withdrawal request
		let trading_account_id = get_trading_account_id(dave());

		let mut withdrawal_request = create_withdrawal_request(
			trading_account_id,
			usdc().asset.id,
			1000.into(),
			1697733048414,
			get_private_key(dave().pub_key),
		)
		.unwrap();

		withdrawal_request.sig_r = U256::from(123123_u128);

		// Dispatch a signed extrinsic.
		assert_ok!(TradingAccountModule::withdraw(RuntimeOrigin::signed(1), withdrawal_request));
	});
}

#[test]
#[should_panic(expected = "InvalidWithdrawalRequest")]
fn test_withdraw_with_insufficient_balance() {
	let mut env = setup();

	env.execute_with(|| {
		// Get trading account of Alice
		let trading_account_id = get_trading_account_id(alice());

		let withdrawal_request = create_withdrawal_request(
			trading_account_id,
			usdc().asset.id,
			11000.into(),
			1697733054847,
			get_private_key(alice().pub_key),
		)
		.unwrap();

		// Dispatch a signed extrinsic.
		assert_ok!(TradingAccountModule::withdraw(RuntimeOrigin::signed(1), withdrawal_request));
	});
}

// basic first 2 trades - no prior trade
#[test]
fn test_volume_update_two_trades() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());

		// market id
		let market_id = btc_usdc().market.id;
		let collateral_id = usdc().asset.id;
		// Create orders
		let alice_order =
			Order::new(201.into(), alice_id).sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(202.into(), bob_id)
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.sign_order(get_private_key(bob().pub_key));

		let alice_30day_volume =
			TradingAccountModule::get_30day_user_volume(alice_id, market_id).unwrap();
		let bob_30day_volume =
			TradingAccountModule::get_30day_user_volume(bob_id, market_id).unwrap();
		let alice_volume_actual =
			TradingAccountModule::monetary_account_volume(alice().account_address, collateral_id);
		// None type volume is returned in case of no prior trades
		assert_eq!(alice_volume_actual.is_none(), true, "Error in trade volume vector");
		// Initial 30 day volume is 0 with no prior trades
		assert_eq!(alice_30day_volume, 0.into(), "Error in 30 day volume");
		assert_eq!(bob_30day_volume, 0.into(), "Error in 30 day volume");

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch_id
			1.into(),
			// quantity_locked
			1.into(),
			// market_id
			market_id,
			// oracle_price
			100.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			1699940367000,
		));

		// Check 30 day volume
		let alice_30day_volume =
			TradingAccountModule::get_30day_user_volume(alice_id, market_id).unwrap();
		let bob_30day_volume =
			TradingAccountModule::get_30day_user_volume(bob_id, market_id).unwrap();

		assert_eq!(alice_30day_volume, 0.into(), "Error in 30 day volume");
		assert_eq!(bob_30day_volume, 0.into(), "Error in 30 day volume");

		// Check timestamp recorded for last trade
		let alice_tx_timestamp = TradingAccountModule::monetary_account_tx_timestamp(
			alice().account_address,
			collateral_id,
		)
		.unwrap();
		assert_eq!(alice_tx_timestamp, 1699940367, "Error in timestamp 1");

		// Check volume vector stored
		let alice_volume_actual =
			TradingAccountModule::monetary_account_volume(alice().account_address, collateral_id)
				.unwrap();
		let mut alice_volume_expected: Vec<FixedI128> = Vec::from([0.into(); 30]);
		alice_volume_expected.insert(0, 100.into());
		assert_eq!(alice_volume_actual, alice_volume_expected, "Error in volume 1");

		// new trade on same day
		let alice_order =
			Order::new(203.into(), alice_id).sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(204.into(), bob_id)
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch_id
			2.into(),
			// quantity_locked
			1.into(),
			// market_id
			market_id,
			// oracle_price
			100.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			1699940367000,
		));

		// 30 day volume should still be the same
		let alice_30day_volume =
			TradingAccountModule::get_30day_user_volume(alice_id, market_id).unwrap();
		let bob_30day_volume =
			TradingAccountModule::get_30day_user_volume(bob_id, market_id).unwrap();

		assert_eq!(alice_30day_volume, 0.into(), "Error in 30 day volume");
		assert_eq!(bob_30day_volume, 0.into(), "Error in 30 day volume");

		let alice_tx_timestamp = TradingAccountModule::monetary_account_tx_timestamp(
			alice().account_address,
			collateral_id,
		)
		.unwrap();
		assert_eq!(alice_tx_timestamp, 1699940367, "Error in timestamp 2");

		let alice_volume_actual =
			TradingAccountModule::monetary_account_volume(alice().account_address, collateral_id)
				.unwrap();
		let mut alice_volume_expected: Vec<FixedI128> = Vec::from([0.into(); 30]);

		alice_volume_expected.insert(0, 200.into()); // trade volume should get added cumulatively to same day
		assert_eq!(alice_volume_actual, alice_volume_expected, "Error in volume 2");

		let bob_tx_timestamp = TradingAccountModule::monetary_account_tx_timestamp(
			bob().account_address,
			collateral_id,
		)
		.unwrap();
		assert_eq!(bob_tx_timestamp, 1699940367, "Error in timestamp 3");
	});
}

#[test]
fn test_volume_update_multiple_trades_with_day_diff() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());

		let init_timestamp: u64 = 1699940367;
		let one_day: u64 = 24 * 60 * 60;
		// market id
		let market_id = btc_usdc().market.id;
		let collateral_id = usdc().asset.id;
		// Create orders
		let alice_order =
			Order::new(201.into(), alice_id).sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(202.into(), bob_id)
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.sign_order(get_private_key(bob().pub_key));

		let alice_30day_volume =
			TradingAccountModule::get_30day_user_volume(alice_id, market_id).unwrap();
		let bob_30day_volume =
			TradingAccountModule::get_30day_user_volume(bob_id, market_id).unwrap();

		assert_eq!(alice_30day_volume, 0.into(), "Error in 30 day volume alice-1");
		assert_eq!(bob_30day_volume, 0.into(), "Error in 30 day volume bob-1");

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch_id
			1.into(),
			// quantity_locked
			1.into(),
			// market_id
			market_id,
			// oracle_price
			100.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			1699940367000,
		));

		let alice_tx_timestamp = TradingAccountModule::monetary_account_tx_timestamp(
			alice().account_address,
			collateral_id,
		)
		.unwrap();
		assert_eq!(alice_tx_timestamp, 1699940367, "Error in timestamp 1");

		let alice_volume_actual =
			TradingAccountModule::monetary_account_volume(alice().account_address, collateral_id)
				.unwrap();
		let mut alice_volume_expected: Vec<FixedI128> = Vec::from([0.into(); 30]);
		alice_volume_expected.insert(0, 100.into());
		assert_eq!(alice_volume_actual, alice_volume_expected, "Error in volume 1");

		let alice_order =
			Order::new(203.into(), alice_id).sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(204.into(), bob_id)
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.sign_order(get_private_key(bob().pub_key));

		// next trade on next day i.e. day 2
		Timestamp::set_timestamp((init_timestamp + one_day) * 1000);

		// getting 30 day trade volume should now include previous day's trade although no new trade
		// is made
		let alice_30day_volume =
			TradingAccountModule::get_30day_user_volume(alice_id, market_id).unwrap();
		let bob_30day_volume =
			TradingAccountModule::get_30day_user_volume(bob_id, market_id).unwrap();

		// 30 day volume should now include previous day's trade volume
		assert_eq!(alice_30day_volume, 100.into(), "Error in 30 day volume alice-2");
		assert_eq!(bob_30day_volume, 100.into(), "Error in 30 day volume bob-2");

		// volume vector should also be the same since it is only updated when a trade is made
		let alice_volume_actual =
			TradingAccountModule::monetary_account_volume(alice().account_address, collateral_id)
				.unwrap();
		let mut alice_volume_expected: Vec<FixedI128> = Vec::from([0.into(); 30]);
		alice_volume_expected.insert(0, 100.into());
		assert_eq!(alice_volume_actual, alice_volume_expected, "Error in volume 2");

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch_id
			2.into(),
			// quantity_locked
			1.into(),
			// market_id
			market_id,
			// oracle_price
			100.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			(init_timestamp + one_day) * 1000,
		));

		let alice_30day_volume =
			TradingAccountModule::get_30day_user_volume(alice_id, market_id).unwrap();
		let bob_30day_volume =
			TradingAccountModule::get_30day_user_volume(bob_id, market_id).unwrap();

		// 30 day volume should include previous day's trade volume
		assert_eq!(alice_30day_volume, 100.into(), "Error in 30 day volume alice-3");
		assert_eq!(bob_30day_volume, 100.into(), "Error in 30 day volume bob-3");

		let alice_tx_timestamp = TradingAccountModule::monetary_account_tx_timestamp(
			alice().account_address,
			collateral_id,
		)
		.unwrap();
		assert_eq!(alice_tx_timestamp, init_timestamp + one_day, "Error in timestamp 2");

		// Check volume vector
		let alice_volume_actual =
			TradingAccountModule::monetary_account_volume(alice().account_address, collateral_id)
				.unwrap();
		let mut alice_volume_expected: Vec<FixedI128> = Vec::from([0.into(); 29]);

		alice_volume_expected.insert(0, 100.into()); // previous day's trade
		alice_volume_expected.insert(0, 100.into()); // present day trade
		assert_eq!(alice_volume_actual, alice_volume_expected, "Error in volume 3");

		let bob_tx_timestamp = TradingAccountModule::monetary_account_tx_timestamp(
			bob().account_address,
			collateral_id,
		)
		.unwrap();
		assert_eq!(bob_tx_timestamp, init_timestamp + one_day, "Error in timestamp 3");

		// next trade on same 2nd day
		Timestamp::set_timestamp((init_timestamp + one_day + (one_day / 2)) * 1000);

		let alice_order =
			Order::new(205.into(), alice_id).sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(206.into(), bob_id)
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch_id
			3.into(),
			// quantity_locked
			1.into(),
			// market_id
			market_id,
			// oracle_price
			100.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			(init_timestamp + one_day + (one_day / 2)) * 1000,
		));

		let alice_30day_volume =
			TradingAccountModule::get_30day_user_volume(alice_id, market_id).unwrap();
		let bob_30day_volume =
			TradingAccountModule::get_30day_user_volume(bob_id, market_id).unwrap();

		// 30 day volume should remain unchanged since it is still day 2
		assert_eq!(alice_30day_volume, 100.into(), "Error in 30 day volume alice-4");
		assert_eq!(bob_30day_volume, 100.into(), "Error in 30 day volume bob-4");

		let alice_tx_timestamp = TradingAccountModule::monetary_account_tx_timestamp(
			alice().account_address,
			collateral_id,
		)
		.unwrap();
		assert_eq!(
			alice_tx_timestamp,
			init_timestamp + one_day + (one_day / 2),
			"Error in timestamp 4"
		);

		let alice_volume_actual =
			TradingAccountModule::monetary_account_volume(alice().account_address, collateral_id)
				.unwrap();
		let mut alice_volume_expected: Vec<FixedI128> = Vec::from([0.into(); 29]);

		alice_volume_expected.insert(0, 100.into()); // previous day's trade volume
		alice_volume_expected.insert(0, 200.into()); // current day's trade volume
		assert_eq!(alice_volume_actual, alice_volume_expected, "Error in volume 4");

		let bob_tx_timestamp = TradingAccountModule::monetary_account_tx_timestamp(
			bob().account_address,
			collateral_id,
		)
		.unwrap();
		assert_eq!(
			bob_tx_timestamp,
			init_timestamp + one_day + (one_day / 2),
			"Error in timestamp 5"
		);

		// next trade on 3rd day
		Timestamp::set_timestamp((init_timestamp + one_day + (one_day)) * 1000);

		// getting 30 day trade volume should now include previous 2 day's trade
		let alice_30day_volume =
			TradingAccountModule::get_30day_user_volume(alice_id, market_id).unwrap();
		let bob_30day_volume =
			TradingAccountModule::get_30day_user_volume(bob_id, market_id).unwrap();

		// 30 day volume should now include previous 2 day's trade volume
		assert_eq!(alice_30day_volume, 300.into(), "Error in 30 day volume alice-5");
		assert_eq!(bob_30day_volume, 300.into(), "Error in 30 day volume bob-5");

		let alice_order =
			Order::new(207.into(), alice_id).sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(208.into(), bob_id)
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch_id
			4.into(),
			// quantity_locked
			1.into(),
			// market_id
			market_id,
			// oracle_price
			100.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			(init_timestamp + one_day + (one_day)) * 1000,
		));

		let alice_30day_volume =
			TradingAccountModule::get_30day_user_volume(alice_id, market_id).unwrap();
		let bob_30day_volume =
			TradingAccountModule::get_30day_user_volume(bob_id, market_id).unwrap();

		// 30 day volume should now include day 1 and day 2 trade volumes
		assert_eq!(alice_30day_volume, 300.into(), "Error in 30 day volume alice-6");
		assert_eq!(bob_30day_volume, 300.into(), "Error in 30 day volume bob-6");

		let alice_tx_timestamp = TradingAccountModule::monetary_account_tx_timestamp(
			alice().account_address,
			collateral_id,
		)
		.unwrap();
		assert_eq!(
			alice_tx_timestamp,
			init_timestamp + one_day + (one_day),
			"Error in timestamp 6"
		);

		let alice_volume_actual =
			TradingAccountModule::monetary_account_volume(alice().account_address, collateral_id)
				.unwrap();
		let mut alice_volume_expected: Vec<FixedI128> = Vec::from([0.into(); 28]);

		alice_volume_expected.insert(0, 100.into()); // day 1 trade volume
		alice_volume_expected.insert(0, 200.into()); // day 2 trade volume
		alice_volume_expected.insert(0, 100.into()); // present day's trade volume
		assert_eq!(alice_volume_actual, alice_volume_expected, "Error in volume 5");

		let bob_tx_timestamp = TradingAccountModule::monetary_account_tx_timestamp(
			bob().account_address,
			collateral_id,
		)
		.unwrap();
		assert_eq!(bob_tx_timestamp, init_timestamp + one_day + (one_day), "Error in timestamp 7");
	});
}

#[test]
fn test_volume_update_30_days_diff() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());

		let init_timestamp: u64 = 1699940367;
		let one_day: u64 = 24 * 60 * 60;
		// market id
		let market_id = btc_usdc().market.id;
		let collateral_id = usdc().asset.id;
		// Create orders
		let alice_order =
			Order::new(201.into(), alice_id).sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(202.into(), bob_id)
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.sign_order(get_private_key(bob().pub_key));

		let alice_30day_volume =
			TradingAccountModule::get_30day_user_volume(alice_id, market_id).unwrap();
		let bob_30day_volume =
			TradingAccountModule::get_30day_user_volume(bob_id, market_id).unwrap();

		assert_eq!(alice_30day_volume, 0.into(), "Error in 30 day volume");
		assert_eq!(bob_30day_volume, 0.into(), "Error in 30 day volume");

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch_id
			1.into(),
			// quantity_locked
			1.into(),
			// market_id
			market_id,
			// oracle_price
			100.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			1699940367000,
		));

		let alice_order =
			Order::new(203.into(), alice_id).sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(204.into(), bob_id)
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.sign_order(get_private_key(bob().pub_key));

		Timestamp::set_timestamp((init_timestamp + 30 * one_day) * 1000);

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch_id
			2.into(),
			// quantity_locked
			1.into(),
			// market_id
			market_id,
			// oracle_price
			100.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			(init_timestamp + 30 * one_day) * 1000,
		));

		let alice_30day_volume =
			TradingAccountModule::get_30day_user_volume(alice_id, market_id).unwrap();
		let bob_30day_volume =
			TradingAccountModule::get_30day_user_volume(bob_id, market_id).unwrap();

		assert_eq!(alice_30day_volume, 100.into(), "Error in 30 day volume");
		assert_eq!(bob_30day_volume, 100.into(), "Error in 30 day volume");

		let alice_tx_timestamp = TradingAccountModule::monetary_account_tx_timestamp(
			alice().account_address,
			collateral_id,
		)
		.unwrap();
		assert_eq!(alice_tx_timestamp, init_timestamp + 30 * one_day, "Error in timestamp 2");

		let alice_volume_actual =
			TradingAccountModule::monetary_account_volume(alice().account_address, collateral_id)
				.unwrap();
		let mut alice_volume_expected: Vec<FixedI128> = Vec::from([0.into(); 29]);

		alice_volume_expected.push(100.into()); // last day's trade (this should now be last element in the volume vector)
		alice_volume_expected.insert(0, 100.into()); // present day trade
		assert_eq!(alice_volume_actual, alice_volume_expected, "Error in volume 2");

		let bob_tx_timestamp = TradingAccountModule::monetary_account_tx_timestamp(
			bob().account_address,
			collateral_id,
		)
		.unwrap();
		assert_eq!(bob_tx_timestamp, init_timestamp + 30 * one_day, "Error in timestamp 3");
	});
}

#[test]
fn test_volume_update_31_days_diff() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());

		let init_timestamp: u64 = 1699940367;
		let one_day: u64 = 24 * 60 * 60;
		// market id
		let market_id = btc_usdc().market.id;
		let collateral_id = usdc().asset.id;
		// Create orders
		let alice_order =
			Order::new(201.into(), alice_id).sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(202.into(), bob_id)
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.sign_order(get_private_key(bob().pub_key));

		let alice_30day_volume =
			TradingAccountModule::get_30day_user_volume(alice_id, market_id).unwrap();
		let bob_30day_volume =
			TradingAccountModule::get_30day_user_volume(bob_id, market_id).unwrap();

		assert_eq!(alice_30day_volume, 0.into(), "Error in 30 day volume");
		assert_eq!(bob_30day_volume, 0.into(), "Error in 30 day volume");
		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch_id
			1.into(),
			// quantity_locked
			1.into(),
			// market_id
			market_id,
			// oracle_price
			100.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			1699940367000,
		));

		let alice_tx_timestamp = TradingAccountModule::monetary_account_tx_timestamp(
			alice().account_address,
			collateral_id,
		)
		.unwrap();
		assert_eq!(alice_tx_timestamp, 1699940367, "Error in timestamp 1");

		let alice_volume_actual =
			TradingAccountModule::monetary_account_volume(alice().account_address, collateral_id)
				.unwrap();
		let mut alice_volume_expected: Vec<FixedI128> = Vec::from([0.into(); 30]);
		alice_volume_expected.insert(0, 100.into());
		assert_eq!(alice_volume_actual, alice_volume_expected, "Error in volume 1");

		let alice_order =
			Order::new(203.into(), alice_id).sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(204.into(), bob_id)
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.sign_order(get_private_key(bob().pub_key));

		// advance timestamp by 31 days
		Timestamp::set_timestamp((init_timestamp + 31 * one_day) * 1000);

		let alice_30day_volume =
			TradingAccountModule::get_30day_user_volume(alice_id, market_id).unwrap();
		let bob_30day_volume =
			TradingAccountModule::get_30day_user_volume(bob_id, market_id).unwrap();

		// last trade should not be included in 30 day trade volume since 31 days have gone by
		assert_eq!(alice_30day_volume, 0.into(), "Error in 30 day volume");
		assert_eq!(bob_30day_volume, 0.into(), "Error in 30 day volume");

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch_id
			2.into(),
			// quantity_locked
			1.into(),
			// market_id
			market_id,
			// oracle_price
			100.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			(init_timestamp + 31 * one_day) * 1000,
		));

		let alice_30day_volume =
			TradingAccountModule::get_30day_user_volume(alice_id, market_id).unwrap();
		let bob_30day_volume =
			TradingAccountModule::get_30day_user_volume(bob_id, market_id).unwrap();

		// last trade should not be included in 30 day trade volume since 31 days have gone by
		assert_eq!(alice_30day_volume, 0.into(), "Error in 30 day volume");
		assert_eq!(bob_30day_volume, 0.into(), "Error in 30 day volume");

		let alice_tx_timestamp = TradingAccountModule::monetary_account_tx_timestamp(
			alice().account_address,
			collateral_id,
		)
		.unwrap();
		assert_eq!(alice_tx_timestamp, init_timestamp + 31 * one_day, "Error in timestamp 2");

		let alice_volume_actual =
			TradingAccountModule::monetary_account_volume(alice().account_address, collateral_id)
				.unwrap();
		let mut alice_volume_expected: Vec<FixedI128> = Vec::from([0.into(); 30]);

		alice_volume_expected.insert(0, 100.into()); // present day trade
		assert_eq!(alice_volume_actual, alice_volume_expected, "Error in volume 2");

		let bob_tx_timestamp = TradingAccountModule::monetary_account_tx_timestamp(
			bob().account_address,
			collateral_id,
		)
		.unwrap();
		assert_eq!(bob_tx_timestamp, init_timestamp + 31 * one_day, "Error in timestamp 3");
	});
}

#[test]
fn test_adjust_balances() {
	let mut env = setup();

	env.execute_with(|| {
		// Get the trading account of Alice
		let trading_account_id = get_trading_account_id(alice());
		let balances_array = vec![BalanceUpdate {
			asset_id: usdc().asset.id,
			balance_value: FixedI128::from_inner(100123456789012345678),
		}];

		// Dispatch a signed extrinsic.
		assert_ok!(TradingAccountModule::set_balances(
			RuntimeOrigin::signed(1),
			trading_account_id,
			balances_array,
		));

		println!("Count: {:?}", TradingAccountModule::accounts_count());

		assert_ok!(TradingAccountModule::adjust_balances(RuntimeOrigin::root(), 0, 3, 6));

		assert_eq!(
			TradingAccountModule::balances(trading_account_id, usdc().asset.id),
			FixedI128::from_inner(100123456000000000000)
		);
		println!(
			"Alice balance: {:?}",
			TradingAccountModule::balances(trading_account_id, usdc().asset.id)
		);
	});
}
