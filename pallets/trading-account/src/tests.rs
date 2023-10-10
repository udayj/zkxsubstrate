use crate::mock::*;
use frame_support::assert_ok;
use primitive_types::U256;
use sp_arithmetic::FixedI128;
use sp_io::hashing::blake2_256;
use zkx_support::test_helpers::asset_helper::{eth, usdc, usdt};
use zkx_support::types::{Asset, BalanceUpdate, TradingAccount, TradingAccountMinimal};

fn setup() -> Vec<TradingAccountMinimal> {
	let mut trading_accounts: Vec<TradingAccountMinimal> = Vec::new();
	let trading_account1 = TradingAccountMinimal {
		account_address: 1000.into(),
		index: 1.into(),
		pub_key: 100.into(),
	};
	let trading_account2 = TradingAccountMinimal {
		account_address: 2000.into(),
		index: 2.into(),
		pub_key: 200.into(),
	};
	let trading_account3 = TradingAccountMinimal {
		account_address: 3000.into(),
		index: 3.into(),
		pub_key: 300.into(),
	};
	trading_accounts.push(trading_account1);
	trading_accounts.push(trading_account2);
	trading_accounts.push(trading_account3);
	trading_accounts
}

fn create_assets() -> Vec<Asset> {
	let assets: Vec<Asset> = vec![eth(), usdc(), usdt()];
	assert_ok!(Assets::replace_all_assets(RuntimeOrigin::signed(1), assets.clone()));
	assets
}

fn get_trading_account_id(trading_accounts: Vec<TradingAccountMinimal>, index: usize) -> U256 {
	let account_address = U256::from(trading_accounts[index].account_address);
	let mut account_array: [u8; 32] = [0; 32];
	account_address.to_little_endian(&mut account_array);

	let mut concatenated_bytes: Vec<u8> = account_array.to_vec();
	concatenated_bytes.push(trading_accounts.get(index).unwrap().index);
	let result: [u8; 33] = concatenated_bytes.try_into().unwrap();

	let trading_account_id: U256 = blake2_256(&result).into();
	trading_account_id
}

#[test]
fn test_add_accounts() {
	new_test_ext().execute_with(|| {
		let trading_accounts = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.
		assert_ok!(TradingAccountModule::add_accounts(
			RuntimeOrigin::signed(1),
			trading_accounts.clone()
		));
		// Read pallet storage and assert an expected result.
		assert_eq!(TradingAccountModule::accounts_count(), 3);

		let mut trading_account_id: U256 = get_trading_account_id(trading_accounts.clone(), 0);
		let mut trading_account: TradingAccount =
			TradingAccountModule::accounts(trading_account_id).unwrap();
		println!("account: {:?}", trading_account);
		assert_eq!(
			trading_accounts.get(0).unwrap().account_address,
			trading_account.account_address
		);
		assert_eq!(trading_accounts.get(0).unwrap().index, trading_account.index);
		assert_eq!(trading_accounts.get(0).unwrap().pub_key, trading_account.pub_key);

		let usdc_id: u128 = usdc().id;
		let expected_balance: FixedI128 = 10000.into();
		let balance: FixedI128 =
			TradingAccountModule::balances(trading_account.account_id, usdc_id);
		assert!(balance == expected_balance);

		trading_account_id = get_trading_account_id(trading_accounts.clone(), 1);
		trading_account = TradingAccountModule::accounts(trading_account_id).unwrap();
		println!("account: {:?}", trading_account);
		assert_eq!(
			trading_accounts.get(1).unwrap().account_address,
			trading_account.account_address
		);
		assert_eq!(trading_accounts.get(1).unwrap().index, trading_account.index);
		assert_eq!(trading_accounts.get(1).unwrap().pub_key, trading_account.pub_key);

		trading_account_id = get_trading_account_id(trading_accounts.clone(), 2);
		trading_account = TradingAccountModule::accounts(trading_account_id).unwrap();
		println!("account: {:?}", trading_account);
		assert_eq!(
			trading_accounts.get(2).unwrap().account_address,
			trading_account.account_address
		);
		assert_eq!(trading_accounts.get(2).unwrap().index, trading_account.index);
		assert_eq!(trading_accounts.get(2).unwrap().pub_key, trading_account.pub_key);
	});
}

#[test]
#[should_panic(expected = "AssetNotFound")]
fn test_add_balances_with_unknown_asset() {
	new_test_ext().execute_with(|| {
		let _assets = create_assets();
		let trading_accounts = setup();
		let usdt_id: u128 = 123;
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.
		assert_ok!(TradingAccountModule::add_accounts(
			RuntimeOrigin::signed(1),
			trading_accounts.clone()
		));

		let trading_account_id: U256 = get_trading_account_id(trading_accounts, 0);
		let trading_account: TradingAccount =
			TradingAccountModule::accounts(trading_account_id).unwrap();
		let balance: BalanceUpdate =
			BalanceUpdate { asset_id: usdt_id, balance_value: 1000.into() };
		let mut collateral_balances: Vec<BalanceUpdate> = Vec::new();
		collateral_balances.push(balance);

		// Dispatch a signed extrinsic.
		assert_ok!(TradingAccountModule::set_balances(
			RuntimeOrigin::signed(1),
			trading_account.account_id,
			collateral_balances
		));
	});
}

#[test]
#[should_panic(expected = "AssetNotCollateral")]
fn test_add_balances_with_asset_not_marked_as_collateral() {
	new_test_ext().execute_with(|| {
		let _assets = create_assets();
		let trading_accounts = setup();
		let eth_id: u128 = eth().id;
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.
		assert_ok!(TradingAccountModule::add_accounts(
			RuntimeOrigin::signed(1),
			trading_accounts.clone()
		));

		let trading_account_id: U256 = get_trading_account_id(trading_accounts, 0);
		let trading_account: TradingAccount =
			TradingAccountModule::accounts(trading_account_id).unwrap();
		let balance: BalanceUpdate = BalanceUpdate { asset_id: eth_id, balance_value: 1000.into() };
		let mut collateral_balances: Vec<BalanceUpdate> = Vec::new();
		collateral_balances.push(balance);

		// Dispatch a signed extrinsic.
		assert_ok!(TradingAccountModule::set_balances(
			RuntimeOrigin::signed(1),
			trading_account.account_id,
			collateral_balances,
		));
	});
}

#[test]
fn test_add_balances() {
	new_test_ext().execute_with(|| {
		let _assets = create_assets();
		let trading_accounts = setup();
		let usdc_id: u128 = usdc().id;
		let usdt_id: u128 = usdt().id;
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.
		assert_ok!(TradingAccountModule::add_accounts(
			RuntimeOrigin::signed(1),
			trading_accounts.clone()
		));

		let trading_account_id: U256 = get_trading_account_id(trading_accounts, 0);
		let trading_account: TradingAccount =
			TradingAccountModule::accounts(trading_account_id).unwrap();
		let balance: BalanceUpdate =
			BalanceUpdate { asset_id: usdc_id, balance_value: 1000.into() };
		let balance1: BalanceUpdate =
			BalanceUpdate { asset_id: usdt_id, balance_value: 500.into() };
		let mut collateral_balances: Vec<BalanceUpdate> = Vec::new();
		collateral_balances.push(balance);
		collateral_balances.push(balance1);

		// Dispatch a signed extrinsic.
		assert_ok!(TradingAccountModule::set_balances(
			RuntimeOrigin::signed(1),
			trading_account.account_id,
			collateral_balances
		));

		assert_eq!(
			TradingAccountModule::balances(trading_account.account_id, usdc_id),
			1000.into()
		);
		assert_eq!(TradingAccountModule::balances(trading_account.account_id, usdt_id), 500.into());

		let collaterals = vec![usdc_id, usdt_id];
		assert_eq!(
			TradingAccountModule::account_collaterals(trading_account.account_id),
			collaterals
		);
	});
}
