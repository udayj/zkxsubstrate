use crate::mock::*;
use frame_support::assert_ok;
use zkx_support::types::{TradingAccount, TradingAccountWithoutId};

fn setup() -> Vec<TradingAccountWithoutId> {
	let mut trading_accounts: Vec<TradingAccountWithoutId> = Vec::new();
	let trading_account1 = TradingAccountWithoutId {
		account_address: 1000.into(),
		index: 1.into(),
		pub_key: 100.into(),
	};
	let trading_account2 = TradingAccountWithoutId {
		account_address: 2000.into(),
		index: 2.into(),
		pub_key: 200.into(),
	};
	let trading_account3 = TradingAccountWithoutId {
		account_address: 3000.into(),
		index: 3.into(),
		pub_key: 300.into(),
	};
	trading_accounts.push(trading_account1);
	trading_accounts.push(trading_account2);
	trading_accounts.push(trading_account3);
	trading_accounts
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
		let mut trading_account: TradingAccount = TradingAccountModule::accounts(0).unwrap();
		println!("account{:?}", trading_account);
		assert_eq!(
			trading_accounts.get(0).unwrap().account_address,
			trading_account.account_address
		);
		assert_eq!(trading_accounts.get(0).unwrap().index, trading_account.index);
		assert_eq!(trading_accounts.get(0).unwrap().pub_key, trading_account.pub_key);

		trading_account = TradingAccountModule::accounts(1).unwrap();
		println!("account{:?}", trading_account);
		assert_eq!(
			trading_accounts.get(1).unwrap().account_address,
			trading_account.account_address
		);
		assert_eq!(trading_accounts.get(1).unwrap().index, trading_account.index);
		assert_eq!(trading_accounts.get(1).unwrap().pub_key, trading_account.pub_key);

		trading_account = TradingAccountModule::accounts(2).unwrap();
		println!("account{:?}", trading_account);
		assert_eq!(
			trading_accounts.get(2).unwrap().account_address,
			trading_account.account_address
		);
		assert_eq!(trading_accounts.get(2).unwrap().index, trading_account.index);
		assert_eq!(trading_accounts.get(2).unwrap().pub_key, trading_account.pub_key);
	});
}
