use crate::mock::*;
use frame_support::assert_ok;
use primitive_types::U256;
use zkx_support::test_helpers::accounts_helper::{
	alice, bob, charlie, create_withdrawal_request, dave, eduard, get_private_key,
	get_trading_account_id,
};
use zkx_support::test_helpers::asset_helper::{btc, eth, usdc, usdt};
use zkx_support::types::BalanceUpdate;

fn setup() -> sp_io::TestExternalities {
	// Create a new test environment
	let mut env = new_test_ext();

	// Set the block number in the environment
	env.execute_with(|| {
		// Set the block number
		System::set_block_number(1);

		// Set the assets in the system
		assert_ok!(Assets::replace_all_assets(
			RuntimeOrigin::signed(1),
			vec![eth(), usdc(), usdt()]
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
		let alice_balance = TradingAccountModule::balances(alice_account_id, usdc().id);
		assert!(alice_balance == 10000.into());

		// Get the trading account of Bob
		let bob_account_id = get_trading_account_id(bob());
		let bob_fetched_account = TradingAccountModule::accounts(bob_account_id)
			.unwrap()
			.to_trading_account_minimal();
		assert_eq!(bob_fetched_account, bob());

		// Check the balance of Bob
		let bob_balance = TradingAccountModule::balances(bob_account_id, usdc().id);
		assert!(bob_balance == 10000.into());

		// Get the trading account of Charlie
		let charlie_account_id = get_trading_account_id(charlie());
		let charlie_fetched_account = TradingAccountModule::accounts(charlie_account_id)
			.unwrap()
			.to_trading_account_minimal();
		assert_eq!(charlie_fetched_account, charlie());

		// Check the balance of Charlie
		let charlie_balance = TradingAccountModule::balances(charlie_account_id, usdc().id);
		assert!(charlie_balance == 10000.into());

		// Get the trading account of Dave
		let dave_account_id = get_trading_account_id(dave());
		let dave_fetched_account = TradingAccountModule::accounts(dave_account_id)
			.unwrap()
			.to_trading_account_minimal();
		assert_eq!(dave_fetched_account, dave());

		// Check the balance of Dave
		let dave_balance = TradingAccountModule::balances(dave_account_id, usdc().id);
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
			vec![BalanceUpdate { asset_id: btc().id, balance_value: 1000.into() }]
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
			vec![BalanceUpdate { asset_id: eth().id, balance_value: 1000.into() }],
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
			BalanceUpdate { asset_id: usdc().id, balance_value: 1000.into() },
			BalanceUpdate { asset_id: usdt().id, balance_value: 500.into() },
		];

		// Dispatch a signed extrinsic.
		assert_ok!(TradingAccountModule::set_balances(
			RuntimeOrigin::signed(1),
			trading_account_id,
			balances_array
		));

		// Check the state
		assert_eq!(TradingAccountModule::balances(trading_account_id, usdc().id), 1000.into());
		assert_eq!(TradingAccountModule::balances(trading_account_id, usdt().id), 500.into());
		assert_eq!(
			TradingAccountModule::account_collaterals(trading_account_id),
			vec![usdc().id, usdt().id]
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
			usdc().id,
			1000.into(),
		));

		// Check the state
		assert_eq!(TradingAccountModule::balances(trading_account_id, usdc().id), 11000.into());
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
			usdc().id,
			1000.into(),
			get_private_key(alice().pub_key),
		)
		.unwrap();

		// Dispatch a signed extrinsic.
		assert_ok!(TradingAccountModule::withdraw(RuntimeOrigin::signed(1), withdrawal_request));

		assert_eq!(TradingAccountModule::balances(trading_account_id, usdc().id), 9000.into());
		let event_record: frame_system::EventRecord<_, _> = System::events().pop().unwrap();
		println!("Events: {:?}", event_record);
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
			usdc().id,
			1000.into(),
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
			usdc().id,
			1000.into(),
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
			usdc().id,
			11000.into(),
			get_private_key(alice().pub_key),
		)
		.unwrap();

		// Dispatch a signed extrinsic.
		assert_ok!(TradingAccountModule::withdraw(RuntimeOrigin::signed(1), withdrawal_request));
	});
}
