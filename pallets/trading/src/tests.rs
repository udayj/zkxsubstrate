use crate::{mock::*, Event};
use frame_support::assert_ok;
use primitive_types::U256;
use sp_io::hashing::blake2_256;
use sp_runtime::FixedI128;
use zkx_support::traits::Hashable;
use zkx_support::types::{
	Asset, BalanceUpdate, Direction, HashType, Market, Order, OrderType, Position, Side,
	TimeInForce, TradingAccountWithoutId,
};

fn setup() -> (Market, Market, Vec<TradingAccountWithoutId>) {
	let ETH_ID: U256 = 4543560.into();
	let USDC_ID: U256 = 1431520323.into();
	let LINK_ID: U256 = 1279872587.into();
	let BTC_ID: U256 = 4346947.into();
	let name1: Vec<u8> = "ETH".into();
	let asset1: Asset = Asset {
		id: ETH_ID,
		name: name1.try_into().unwrap(),
		is_tradable: true,
		is_collateral: false,
		token_decimal: 18,
	};
	let name2: Vec<u8> = "USDC".into();
	let asset2: Asset = Asset {
		id: USDC_ID,
		name: name2.try_into().unwrap(),
		is_tradable: false,
		is_collateral: true,
		token_decimal: 6,
	};
	let name3: Vec<u8> = "LINK".into();
	let asset3: Asset = Asset {
		id: LINK_ID,
		name: name3.try_into().unwrap(),
		is_tradable: true,
		is_collateral: false,
		token_decimal: 6,
	};
	let name3: Vec<u8> = "BTC".into();
	let asset4: Asset = Asset {
		id: BTC_ID,
		name: name3.try_into().unwrap(),
		is_tradable: true,
		is_collateral: false,
		token_decimal: 6,
	};

	let assets: Vec<Asset> = vec![asset1.clone(), asset2.clone(), asset3.clone()];
	assert_ok!(Assets::replace_all_assets(RuntimeOrigin::signed(1), assets));

	let market1: Market = Market {
		id: 1.into(),
		asset: ETH_ID,
		asset_collateral: USDC_ID,
		is_tradable: true,
		is_archived: false,
		ttl: 3600,
		tick_size: 1.into(),
		tick_precision: 1,
		step_size: 1.into(),
		step_precision: 1,
		minimum_order_size: 1.into(),
		minimum_leverage: 1.into(),
		maximum_leverage: 10.into(),
		currently_allowed_leverage: 8.into(),
		maintenance_margin_fraction: 1.into(),
		initial_margin_fraction: 1.into(),
		incremental_initial_margin_fraction: 1.into(),
		incremental_position_size: 1.into(),
		baseline_position_size: 1.into(),
		maximum_position_size: 1.into(),
	};
	let market2: Market = Market {
		id: 2.into(),
		asset: LINK_ID,
		asset_collateral: USDC_ID,
		is_tradable: false,
		is_archived: false,
		ttl: 360,
		tick_size: 1.into(),
		tick_precision: 1,
		step_size: 1.into(),
		step_precision: 1,
		minimum_order_size: 1.into(),
		minimum_leverage: 1.into(),
		maximum_leverage: 10.into(),
		currently_allowed_leverage: 8.into(),
		maintenance_margin_fraction: 1.into(),
		initial_margin_fraction: 1.into(),
		incremental_initial_margin_fraction: 1.into(),
		incremental_position_size: 1.into(),
		baseline_position_size: 1.into(),
		maximum_position_size: 1.into(),
	};

	let markets: Vec<Market> = vec![market1.clone(), market2.clone()];
	assert_ok!(Markets::replace_all_markets(RuntimeOrigin::signed(1), markets));

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
		"2023717274951296412493017643304581901896527410883689276347754537542684932623",
	)
	.unwrap();
	let user_pri_key_2: U256 = U256::from_dec_str(
		"3178926723073418235635570028666214174158474174899259562390812806614237362579",
	)
	.unwrap();
	let user_address_2: U256 = U256::from(101_u8);
	let user_1 = TradingAccountWithoutId {
		account_address: user_address_1,
		index: 0,
		pub_key: user_pub_key_1,
	};
	let user_2 =
		TradingAccountWithoutId { account_address: user_address_2, index: 0, pub_key: 2.into() };
	let accounts: Vec<TradingAccountWithoutId> = vec![user_1, user_2];
	assert_ok!(TradingAccounts::add_accounts(RuntimeOrigin::signed(1), accounts.clone()));

	(market1, market2, accounts)
}

fn get_trading_account_id(trading_accounts: Vec<TradingAccountWithoutId>, index: usize) -> U256 {
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
// basic open trade without any leverage
fn it_works_for_open_trade_1() {
	new_test_ext().execute_with(|| {
		let order_id_1 = 200_u128;
		let order_id_2 = 201_u128;

		let (market1, _, accounts) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		let account_id_1: U256 = get_trading_account_id(accounts.clone(), 0);
		let account_id_2: U256 = get_trading_account_id(accounts.clone(), 1);

		let order_1 = Order {
			account_id: account_id_1,
			order_id: order_id_1,
			market_id: market1.id,
			order_type: OrderType::Limit,
			direction: Direction::Long,
			side: Side::Buy,
			price: 100.into(),
			size: 1.into(),
			leverage: 1.into(),
			slippage: 10.into(),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};
		let order_2 = Order {
			account_id: account_id_2,
			order_id: order_id_2,
			market_id: market1.id,
			order_type: OrderType::Market,
			direction: Direction::Short,
			side: Side::Buy,
			price: 100.into(),
			size: 1.into(),
			leverage: 1.into(),
			slippage: 10.into(),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};

		let order_hash1 = order_1.hash(&order_1.hash_type);

		// let orders: Vec<Order> = vec![order_1, order_2];

		// assert_ok!(Trading::execute_trade(
		// 	RuntimeOrigin::signed(1),
		// 	U256::from(1_u8),
		// 	1.into(),
		// 	market1.id,
		// 	100.into(),
		// 	orders
		// ));

		// let position1 = Trading::positions(user_id_1, [market1.id, U256::from(1_u8)]);
		// let expected_position: Position = Position {
		// 	avg_execution_price: 100.into(),
		// 	size: 1.into(),
		// 	margin_amount: 100.into(),
		// 	borrowed_amount: 0.into(),
		// 	leverage: 1.into(),
		// 	realized_pnl: 0.into(),
		// };
		// assert_eq!(expected_position, position1);
		// println!("{:?}", position1);

		// let position2 = Trading::positions(user_id_2, [market1.id, U256::from(2_u8)]);
		// let expected_position: Position = Position {
		// 	avg_execution_price: 100.into(),
		// 	size: 1.into(),
		// 	margin_amount: 100.into(),
		// 	borrowed_amount: 0.into(),
		// 	leverage: 1.into(),
		// 	realized_pnl: 0.into(),
		// };
		// assert_eq!(expected_position, position2);
		// println!("{:?}", position2);
	});
}

// #[test]
// // basic open trade with leverage
// fn it_works_for_open_trade_2() {
// 	new_test_ext().execute_with(|| {
// 		let order_id_1 = 200_u128;
// 		let order_id_2 = 201_u128;

// 		let (market1, _) = setup();
// 		// Go past genesis block so events get deposited
// 		System::set_block_number(1);

// 		let user_id_1 = TradingAccounts::accounts(0).unwrap().account_id;
// 		let user_id_2 = TradingAccounts::accounts(0).unwrap().account_id;

// 		let order_1 = Order {
// 			user: user_id_1,
// 			order_id: order_id_1,
// 			market_id: market1.id,
// 			order_type: OrderType::Limit,
// 			direction: Direction::Long,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: 1.into(),
// 			leverage: 5.into(),
// 			slippage: 10.into(),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 		};
// 		let order_2 = Order {
// 			user: user_id_2,
// 			order_id: order_id_2,
// 			market_id: market1.id,
// 			order_type: OrderType::Market,
// 			direction: Direction::Short,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: 1.into(),
// 			leverage: 5.into(),
// 			slippage: 10.into(),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 		};
// 		let orders: Vec<Order> = vec![order_1, order_2];

// 		assert_ok!(Trading::execute_trade(
// 			RuntimeOrigin::signed(1),
// 			U256::from(1_u8),
// 			1.into(),
// 			market1.id,
// 			100.into(),
// 			orders
// 		));

// 		let position1 = Trading::positions(user_id_1, [market1.id, U256::from(1_u8)]);
// 		let expected_position: Position = Position {
// 			avg_execution_price: 100.into(),
// 			size: 1.into(),
// 			margin_amount: 20.into(),
// 			borrowed_amount: 80.into(),
// 			leverage: 5.into(),
// 			realized_pnl: 0.into(),
// 		};
// 		assert_eq!(expected_position, position1);
// 		println!("{:?}", position1);

// 		let position2 = Trading::positions(user_id_2, [market1.id, U256::from(2_u8)]);
// 		let expected_position: Position = Position {
// 			avg_execution_price: 100.into(),
// 			size: 1.into(),
// 			margin_amount: 20.into(),
// 			borrowed_amount: 80.into(),
// 			leverage: 5.into(),
// 			realized_pnl: 0.into(),
// 		};
// 		assert_eq!(expected_position, position2);
// 		println!("{:?}", position2);
// 	});
// }

// #[test]
// // basic open and close trade without any leverage
// fn it_works_for_close_trade_1() {
// 	new_test_ext().execute_with(|| {
// 		let order_id_1 = 200_u128;
// 		let order_id_2 = 201_u128;
// 		let order_id_3 = 202_u128;
// 		let order_id_4 = 203_u128;

// 		let (market1, _) = setup();
// 		// Go past genesis block so events get deposited
// 		System::set_block_number(1);

// 		let user_id_1 = TradingAccounts::accounts(0).unwrap().account_id;
// 		let user_id_2 = TradingAccounts::accounts(1).unwrap().account_id;

// 		let order_1 = Order {
// 			user: user_id_1,
// 			order_id: order_id_1,
// 			market_id: market1.id,
// 			order_type: OrderType::Limit,
// 			direction: Direction::Long,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: 1.into(),
// 			leverage: 1.into(),
// 			slippage: 10.into(),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 		};
// 		let order_2 = Order {
// 			user: user_id_2,
// 			order_id: order_id_2,
// 			market_id: market1.id,
// 			order_type: OrderType::Market,
// 			direction: Direction::Short,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: 1.into(),
// 			leverage: 1.into(),
// 			slippage: 10.into(),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 		};
// 		let orders: Vec<Order> = vec![order_1, order_2];

// 		assert_ok!(Trading::execute_trade(
// 			RuntimeOrigin::signed(1),
// 			U256::from(1_u8),
// 			1.into(),
// 			market1.id,
// 			100.into(),
// 			orders
// 		));

// 		let position1 = Trading::positions(user_id_1, [market1.id, U256::from(1_u8)]);
// 		println!("{:?}", position1);

// 		let position2 = Trading::positions(user_id_2, [market1.id, U256::from(2_u8)]);
// 		println!("{:?}", position2);

// 		// let usdc_id: U256 = 1431520323.into();
// 		// let user_id_1 = TradingAccounts::accounts(0).unwrap().account_id;
// 		// let user_id_2 = TradingAccounts::accounts(1).unwrap().account_id;
// 		// println!("Users {:?} {:?}", user_id_1, user_id_2);
// 		// let balance_1 = TradingAccounts::balances(user_id_1, usdc_id);
// 		// println!("Balance {:?}", (balance_1));
// 		// let balance_2 = TradingAccounts::balances(user_id_2, usdc_id);
// 		// println!("Balance {:?}", (balance_2));
// 		// let locked_1 = TradingAccounts::locked_margin(user_id_1, usdc_id);
// 		// println!("Locked margin {:?}", (locked_1));
// 		// let locked_2 = TradingAccounts::locked_margin(user_id_2, usdc_id);
// 		// println!("Locked margin {:?}", (locked_2));
// 		// let portion = Trading::portion_executed(order_id_3);
// 		// println!("Portion executed {:?}", (portion));
// 		// let portion1 = Trading::portion_executed(order_id_4);
// 		// println!("Portion executed {:?}", (portion1));

// 		// Close orders
// 		let order_3 = Order {
// 			user: user_id_1,
// 			order_id: order_id_3,
// 			market_id: market1.id,
// 			order_type: OrderType::Limit,
// 			direction: Direction::Long,
// 			side: Side::Sell,
// 			price: 105.into(),
// 			size: 1.into(),
// 			leverage: 1.into(),
// 			slippage: 10.into(),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 		};
// 		let order_4 = Order {
// 			user: user_id_2,
// 			order_id: order_id_4,
// 			market_id: market1.id,
// 			order_type: OrderType::Market,
// 			direction: Direction::Short,
// 			side: Side::Sell,
// 			price: 100.into(),
// 			size: 1.into(),
// 			leverage: 1.into(),
// 			slippage: 10.into(),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 		};
// 		let orders: Vec<Order> = vec![order_3, order_4];

// 		assert_ok!(Trading::execute_trade(
// 			RuntimeOrigin::signed(1),
// 			U256::from(2_u8),
// 			1.into(),
// 			market1.id,
// 			105.into(),
// 			orders
// 		));

// 		let usdc_id: U256 = 1431520323.into();
// 		let user_id_1 = TradingAccounts::accounts(0).unwrap().account_id;
// 		let user_id_2 = TradingAccounts::accounts(1).unwrap().account_id;
// 		let balance_1 = TradingAccounts::balances(user_id_1, usdc_id);
// 		println!("Balance {:?}", (balance_1));
// 		let balance_2 = TradingAccounts::balances(user_id_2, usdc_id);
// 		println!("Balance {:?}", (balance_2));
// 		let locked_1 = TradingAccounts::locked_margin(user_id_1, usdc_id);
// 		println!("Locked margin {:?}", (locked_1));
// 	});
// }
