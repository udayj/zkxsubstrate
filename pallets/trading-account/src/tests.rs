use crate::mock::*;
use frame_support::assert_ok;
use primitive_types::U256;
use sp_arithmetic::FixedI128;
use sp_io::hashing::blake2_256;
use starknet_crypto::{sign, FieldElement};
use zkx_support::test_helpers::{eth, usdc, usdt};
use zkx_support::traits::{FieldElementExt, Hashable, U256Ext};
use zkx_support::types::{
	Asset, BalanceUpdate, HashType, TradingAccount, TradingAccountMinimal, WithdrawalRequest,
};

fn setup() -> (Vec<TradingAccountMinimal>, Vec<U256>) {
	assert_ok!(Timestamp::set(None.into(), 100));
	let user_pub_key_1: U256 = U256::from_dec_str(
		"454932787469224290468444410084879070088819078827906347654495047407276534283",
	)
	.unwrap();
	let user_pri_key_1: U256 = U256::from_dec_str(
		"217039137810971208563823259722717297948702641410765313684702872265493782699",
	)
	.unwrap();
	let user_address_1: U256 = U256::from(100_u8);

	let user_pub_key_2: U256 = U256::from_dec_str(
		"2101677845476848141002376837472833021659088026888369432434421980160153750090",
	)
	.unwrap();
	let user_pri_key_2: U256 = U256::from_dec_str(
		"2835524789612495000294332407161775540542356260492319813526822636942276039073",
	)
	.unwrap();
	let user_address_2: U256 = U256::from(101_u8);

	let user_pub_key_3: U256 = U256::from_dec_str(
		"1927799101328918885926814969993421873905724180750168745093131010179897850144",
	)
	.unwrap();
	let user_pri_key_3: U256 = U256::from_dec_str(
		"3388506857955987752046415916181604993164423072000548640801744803879383940670",
	)
	.unwrap();
	let user_address_3: U256 = U256::from(102_u8);

	let user_pub_key_4: U256 = U256::from_dec_str(
		"824120678599933675767871867465569325984720238047137957464936400424120564339",
	)
	.unwrap();
	let user_pri_key_4: U256 = U256::from_dec_str(
		"84035867551811388210596922086133550045728262314839423570645036080104955628",
	)
	.unwrap();
	let user_address_4: U256 = U256::from(103_u8);

	let user_1 = TradingAccountMinimal {
		account_address: user_address_1,
		index: 0,
		pub_key: user_pub_key_1,
	};
	let user_2 = TradingAccountMinimal {
		account_address: user_address_2,
		index: 0,
		pub_key: user_pub_key_2,
	};
	let user_3 = TradingAccountMinimal {
		account_address: user_address_3,
		index: 0,
		pub_key: user_pub_key_3,
	};
	let user_4 = TradingAccountMinimal {
		account_address: user_address_4,
		index: 0,
		pub_key: user_pub_key_4,
	};
	let accounts: Vec<TradingAccountMinimal> = vec![user_1, user_2, user_3, user_4];
	assert_ok!(TradingAccountModule::add_accounts(RuntimeOrigin::signed(1), accounts.clone()));

	let private_keys: Vec<U256> =
		vec![user_pri_key_1, user_pri_key_2, user_pri_key_3, user_pri_key_4];

	(accounts, private_keys)
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

fn sign_withdrawal_request(
	withdrawal_request: WithdrawalRequest,
	private_key: U256,
) -> WithdrawalRequest {
	let withdrawal_request_hash = withdrawal_request.hash(&withdrawal_request.hash_type).unwrap();
	let private_key = private_key.try_to_felt().unwrap();
	let signature = sign(&private_key, &withdrawal_request_hash, &FieldElement::ONE).unwrap();

	let sig_r = signature.r.to_u256();
	let sig_s = signature.s.to_u256();
	WithdrawalRequest { sig_r, sig_s, ..withdrawal_request }
}

#[test]
fn test_add_accounts() {
	new_test_ext().execute_with(|| {
		let (trading_accounts, _) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Read pallet storage and assert an expected result.
		assert_eq!(TradingAccountModule::accounts_count(), 4);

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

		let usdc_id: u128 = 93816115890698;
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
		let (trading_accounts, _) = setup();
		let usdt_id: u128 = 123;
		// Go past genesis block so events get deposited
		System::set_block_number(1);

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
		let (trading_accounts, _) = setup();
		let eth_id: u128 = 1163151370;
		// Go past genesis block so events get deposited
		System::set_block_number(1);

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
		let (trading_accounts, _) = setup();
		let usdc_id: u128 = 93816115890698;
		let usdt_id: u128 = 24016925953231370;
		// Go past genesis block so events get deposited
		System::set_block_number(1);

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

// #[test]
// #[should_panic(expected = "AssetNotFound")]
// fn test_deposit_with_asset_not_marked_as_collateral() {
// 	new_test_ext().execute_with(|| {
// 		let _assets = create_assets();
// 		let (trading_accounts, private_keys) = setup();
// 		let usdt_id: u128 = 123;
// 		// Go past genesis block so events get deposited
// 		System::set_block_number(1);

// 		let trading_account_id: U256 = get_trading_account_id(trading_accounts, 0);
// 		let trading_account: TradingAccount =
// 			TradingAccountModule::accounts(trading_account_id).unwrap();

// 		// Dispatch a signed extrinsic.
// 		assert_ok!(TradingAccountModule::deposit(
// 			RuntimeOrigin::signed(1),
// 			trading_account.account_id,
// 			trading_account.index,
// 			trading_account.pub_key,
// 			usdt_id,
// 			1000.into(),
// 		));
// 	});
// }

// #[test]
// #[should_panic(expected = "AssetNotCollateral")]
// fn test_deposit_with_unknown_asset() {
// 	new_test_ext().execute_with(|| {
// 		let _assets = create_assets();
// 		let (trading_accounts, private_keys) = setup();
// 		let eth_id: u128 = 4543560;
// 		// Go past genesis block so events get deposited
// 		System::set_block_number(1);

// 		let trading_account_id: U256 = get_trading_account_id(trading_accounts, 0);
// 		let trading_account: TradingAccount =
// 			TradingAccountModule::accounts(trading_account_id).unwrap();

// 		// Dispatch a signed extrinsic.
// 		assert_ok!(TradingAccountModule::deposit(
// 			RuntimeOrigin::signed(1),
// 			trading_account.account_id,
// 			trading_account.index,
// 			trading_account.pub_key,
// 			eth_id,
// 			1000.into(),
// 		));
// 	});
// }

// #[test]
// fn test_deposit() {
// 	new_test_ext().execute_with(|| {
// 		let _assets = create_assets();
// 		let (trading_accounts, private_keys) = setup();
// 		let usdc_id: u128 = 1431520323;
// 		// Go past genesis block so events get deposited
// 		System::set_block_number(1);

// 		let trading_account_id: U256 = get_trading_account_id(trading_accounts, 0);
// 		let trading_account: TradingAccount =
// 			TradingAccountModule::accounts(trading_account_id).unwrap();

// 		// Dispatch a signed extrinsic.
// 		assert_ok!(TradingAccountModule::deposit(
// 			RuntimeOrigin::signed(1),
// 			trading_account.account_address,
// 			trading_account.index,
// 			trading_account.pub_key,
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

#[test]
fn test_withdraw() {
	new_test_ext().execute_with(|| {
		let _assets = create_assets();
		let (trading_accounts, private_keys) = setup();
		let usdc_id: u128 = 93816115890698;
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		let trading_account_id: U256 = get_trading_account_id(trading_accounts, 0);
		let trading_account: TradingAccount =
			TradingAccountModule::accounts(trading_account_id).unwrap();

		let withdrawal_request = WithdrawalRequest {
			account_id: trading_account_id,
			collateral_id: usdc_id,
			amount: 1000.into(),
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};

		let withdrawal_request = sign_withdrawal_request(withdrawal_request, private_keys[0]);

		// Dispatch a signed extrinsic.
		assert_ok!(TradingAccountModule::withdraw(RuntimeOrigin::signed(1), withdrawal_request));

		assert_eq!(
			TradingAccountModule::balances(trading_account.account_id, usdc_id),
			9000.into()
		);
		let event_record: frame_system::EventRecord<_, _> = System::events().pop().unwrap();
		println!("Events: {:?}", event_record);
	});
}

#[test]
#[should_panic(expected = "AccountDoesNotExist")]
fn test_withdraw_on_not_existing_account() {
	new_test_ext().execute_with(|| {
		let _assets = create_assets();
		let (_, private_keys) = setup();
		let usdc_id: u128 = 93816115890698;
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		let withdrawal_request = WithdrawalRequest {
			account_id: 1.into(),
			collateral_id: usdc_id,
			amount: 1000.into(),
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};

		let withdrawal_request = sign_withdrawal_request(withdrawal_request, private_keys[0]);

		// Dispatch a signed extrinsic.
		assert_ok!(TradingAccountModule::withdraw(RuntimeOrigin::signed(1), withdrawal_request));
	});
}

#[test]
#[should_panic(
	expected = "AccountManager: This withdrawal will lead to either deleveraging or liquidation"
)]
fn test_withdraw_with_insufficient_balance() {
	new_test_ext().execute_with(|| {
		let _assets = create_assets();
		let (trading_accounts, private_keys) = setup();
		let usdc_id: u128 = 93816115890698;
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		let trading_account_id: U256 = get_trading_account_id(trading_accounts, 0);

		let withdrawal_request = WithdrawalRequest {
			account_id: trading_account_id,
			collateral_id: usdc_id,
			amount: 11000.into(),
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};

		let withdrawal_request = sign_withdrawal_request(withdrawal_request, private_keys[0]);

		// Dispatch a signed extrinsic.
		assert_ok!(TradingAccountModule::withdraw(RuntimeOrigin::signed(1), withdrawal_request));
	});
}
