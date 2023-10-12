use crate::mock::*;
use frame_support::assert_ok;
use zkx_support::test_helpers::accounts_helper::{alice, bob, charlie, dave};
use zkx_support::test_helpers::asset_helper::{eth, usdc, usdt};
use zkx_support::types::{
	Asset, BalanceUpdate, HashType, TradingAccount, TradingAccountMinimal, WithdrawalRequest,
};

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

// #[test]
// #[should_panic(expected = "AssetNotFound")]
// fn test_add_balances_with_unknown_asset() {
// 	new_test_ext().execute_with(|| {
// 		let _assets = create_assets();
// 		let (trading_accounts, _) = setup();
// 		let usdt_id: u128 = 123;
// 		// Go past genesis block so events get deposited
// 		System::set_block_number(1);

// 		let trading_account_id: U256 = get_trading_account_id(trading_accounts, 0);
// 		let trading_account: TradingAccount =
// 			TradingAccountModule::accounts(trading_account_id).unwrap();
// 		let balance: BalanceUpdate =
// 			BalanceUpdate { asset_id: usdt_id, balance_value: 1000.into() };
// 		let mut collateral_balances: Vec<BalanceUpdate> = Vec::new();
// 		collateral_balances.push(balance);

// 		// Dispatch a signed extrinsic.
// 		assert_ok!(TradingAccountModule::set_balances(
// 			RuntimeOrigin::signed(1),
// 			trading_account.account_id,
// 			collateral_balances
// 		));
// 	});
// }

// #[test]
// #[should_panic(expected = "AssetNotCollateral")]
// fn test_add_balances_with_asset_not_marked_as_collateral() {
// 	new_test_ext().execute_with(|| {
// 		let _assets = create_assets();
// 		let (trading_accounts, _) = setup();
// 		let eth_id: u128 = eth().id;
// 		// Go past genesis block so events get deposited
// 		System::set_block_number(1);

// 		let trading_account_id: U256 = get_trading_account_id(trading_accounts, 0);
// 		let trading_account: TradingAccount =
// 			TradingAccountModule::accounts(trading_account_id).unwrap();
// 		let balance: BalanceUpdate = BalanceUpdate { asset_id: eth_id, balance_value: 1000.into() };
// 		let mut collateral_balances: Vec<BalanceUpdate> = Vec::new();
// 		collateral_balances.push(balance);

// 		// Dispatch a signed extrinsic.
// 		assert_ok!(TradingAccountModule::set_balances(
// 			RuntimeOrigin::signed(1),
// 			trading_account.account_id,
// 			collateral_balances,
// 		));
// 	});
// }

// #[test]
// fn test_add_balances() {
// 	new_test_ext().execute_with(|| {
// 		let _assets = create_assets();
// 		let (trading_accounts, _) = setup();
// 		let usdc_id: u128 = usdc().id;
// 		let usdt_id: u128 = usdt().id;
// 		// Go past genesis block so events get deposited
// 		System::set_block_number(1);

// 		let trading_account_id: U256 = get_trading_account_id(trading_accounts, 0);
// 		let trading_account: TradingAccount =
// 			TradingAccountModule::accounts(trading_account_id).unwrap();
// 		let balance: BalanceUpdate =
// 			BalanceUpdate { asset_id: usdc_id, balance_value: 1000.into() };
// 		let balance1: BalanceUpdate =
// 			BalanceUpdate { asset_id: usdt_id, balance_value: 500.into() };
// 		let mut collateral_balances: Vec<BalanceUpdate> = Vec::new();
// 		collateral_balances.push(balance);
// 		collateral_balances.push(balance1);

// 		// Dispatch a signed extrinsic.
// 		assert_ok!(TradingAccountModule::set_balances(
// 			RuntimeOrigin::signed(1),
// 			trading_account.account_id,
// 			collateral_balances
// 		));

// 		assert_eq!(
// 			TradingAccountModule::balances(trading_account.account_id, usdc_id),
// 			1000.into()
// 		);
// 		assert_eq!(TradingAccountModule::balances(trading_account.account_id, usdt_id), 500.into());

// 		let collaterals = vec![usdc_id, usdt_id];
// 		assert_eq!(
// 			TradingAccountModule::account_collaterals(trading_account.account_id),
// 			collaterals
// 		);
// 	});
// }

// #[test]
// fn test_deposit() {
// 	new_test_ext().execute_with(|| {
// 		let _assets = create_assets();
// 		let (trading_accounts, _private_keys) = setup();
// 		let usdc_id: u128 = usdc().id;
// 		// Go past genesis block so events get deposited
// 		System::set_block_number(1);

// 		let trading_account_id: U256 = get_trading_account_id(trading_accounts, 0);
// 		let trading_account: TradingAccount =
// 			TradingAccountModule::accounts(trading_account_id).unwrap();

// 		// Dispatch a signed extrinsic.
// 		assert_ok!(TradingAccountModule::deposit(
// 			RuntimeOrigin::signed(1),
// 			trading_account.to_trading_account_minimal(),
// 			usdc_id,
// 			1000.into(),
// 		));

// 		assert_eq!(
// 			TradingAccountModule::balances(trading_account.account_id, usdc_id),
// 			11000.into()
// 		);
// 		let event_record: frame_system::EventRecord<_, _> = System::events().pop().unwrap();
// 		println!("Events: {:?}", event_record);
// 	});
// }

// #[test]
// fn test_withdraw() {
// 	new_test_ext().execute_with(|| {
// 		let _assets = create_assets();
// 		let (trading_accounts, private_keys) = setup();
// 		let usdc_id: u128 = usdc().id;
// 		// Go past genesis block so events get deposited
// 		System::set_block_number(1);

// 		let trading_account_id: U256 = get_trading_account_id(trading_accounts, 0);
// 		let trading_account: TradingAccount =
// 			TradingAccountModule::accounts(trading_account_id).unwrap();

// 		let withdrawal_request = WithdrawalRequest {
// 			account_id: trading_account_id,
// 			collateral_id: usdc_id,
// 			amount: 1000.into(),
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};

// 		let withdrawal_request = sign_withdrawal_request(withdrawal_request, private_keys[0]);

// 		// Dispatch a signed extrinsic.
// 		assert_ok!(TradingAccountModule::withdraw(RuntimeOrigin::signed(1), withdrawal_request));

// 		assert_eq!(
// 			TradingAccountModule::balances(trading_account.account_id, usdc_id),
// 			9000.into()
// 		);
// 		let event_record: frame_system::EventRecord<_, _> = System::events().pop().unwrap();
// 		println!("Events: {:?}", event_record);
// 	});
// }

// #[test]
// #[should_panic(expected = "AccountDoesNotExist")]
// fn test_withdraw_on_not_existing_account() {
// 	new_test_ext().execute_with(|| {
// 		let _assets = create_assets();
// 		let (_, private_keys) = setup();
// 		let usdc_id: u128 = 93816115890698;
// 		// Go past genesis block so events get deposited
// 		System::set_block_number(1);

// 		let withdrawal_request = WithdrawalRequest {
// 			account_id: 1.into(),
// 			collateral_id: usdc_id,
// 			amount: 1000.into(),
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};

// 		let withdrawal_request = sign_withdrawal_request(withdrawal_request, private_keys[0]);

// 		// Dispatch a signed extrinsic.
// 		assert_ok!(TradingAccountModule::withdraw(RuntimeOrigin::signed(1), withdrawal_request));
// 	});
// }

// #[test]
// #[should_panic(expected = "InvalidWithdrawalRequest")]
// fn test_withdraw_with_insufficient_balance() {
// 	new_test_ext().execute_with(|| {
// 		let _assets = create_assets();
// 		let (trading_accounts, private_keys) = setup();
// 		let usdc_id: u128 = 93816115890698;
// 		// Go past genesis block so events get deposited
// 		System::set_block_number(1);

// 		let trading_account_id: U256 = get_trading_account_id(trading_accounts, 0);

// 		let withdrawal_request = WithdrawalRequest {
// 			account_id: trading_account_id,
// 			collateral_id: usdc_id,
// 			amount: 11000.into(),
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};

// 		let withdrawal_request = sign_withdrawal_request(withdrawal_request, private_keys[0]);

// 		// Dispatch a signed extrinsic.
// 		assert_ok!(TradingAccountModule::withdraw(RuntimeOrigin::signed(1), withdrawal_request));
// 	});
// }
