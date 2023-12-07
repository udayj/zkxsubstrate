use crate::mock::*;
use frame_support::assert_ok;
use pallet_support::{
	test_helpers::{
		accounts_helper::{
			alice, bob, charlie, create_withdrawal_request, dave, eduard, get_private_key,
			get_trading_account_id,
		},
		asset_helper::{btc, eth, usdc, usdt, link},
		market_helper::{btc_usdc, link_usdc}
	},
	types::{BalanceUpdate, Order, 
		trading::{Direction, OrderType}}, traits::TradingAccountInterface
};
use primitive_types::U256;

fn setup() -> sp_io::TestExternalities {
	// Create a new test environment
	let mut env = new_test_ext();

	// Set the block number in the environment
	env.execute_with(|| {
		// Set the block number
		System::set_block_number(1);
		assert_ok!(Timestamp::set(None.into(), 1699940367000));
		// Set the assets in the system
		assert_ok!(
			Assets::replace_all_assets(RuntimeOrigin::signed(1), vec![eth(), usdc(), link(), btc(), usdt()])
		);

		assert_ok!(
			Markets::replace_all_markets(RuntimeOrigin::signed(1), vec![btc_usdc(), link_usdc()])
		);

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
			vec![BalanceUpdate { asset_id: btc().asset.id, balance_value: 1000.into() }]
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
		assert_ok!(
			TradingAccountModule::withdraw(RuntimeOrigin::signed(1), withdrawal_request.clone())
		);

		// Send the withdrawal request again
		assert_ok!(
			TradingAccountModule::withdraw(RuntimeOrigin::signed(1), withdrawal_request.clone())
		);
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

// basic first trade for the system - no prior trades exist
#[test]
fn test_volume_update_first_trade() {
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
			Order::new(201_u128, alice_id).sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(202_u128, bob_id)
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch_id
			U256::from(1_u8),
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

		let alice_30day_volume = TradingAccountModule::get_30day_volume(alice_id, market_id).unwrap();
		let bob_30day_volume = TradingAccountModule::get_30day_volume(bob_id, market_id).unwrap();

		assert_eq!(alice_30day_volume, 0.into(), "Error in 30 day volume");
		assert_eq!(bob_30day_volume, 0.into(), "Error in 30 day volume");

		let alice_tx_timestamp = TradingAccountModule::monetary_account_tx_timestamp(alice().account_address, collateral_id).unwrap();
		assert_eq!(alice_tx_timestamp, 1699940367, "Error in timestamp");
	});
}