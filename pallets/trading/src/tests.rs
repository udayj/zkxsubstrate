use crate::{mock::*, Event};
use frame_support::assert_ok;
use primitive_types::U256;
use sp_arithmetic::FixedI128;
use sp_io::hashing::blake2_256;
use sp_runtime::BoundedVec;
use starknet_crypto::{sign, FieldElement};
use zkx_support::test_helpers::accounts_helper::{
	alice, bob, charlie, create_withdrawal_request, dave, eduard, get_private_key,
	get_trading_account_id,
};
use zkx_support::test_helpers::asset_helper::{btc, eth, link, usdc};
use zkx_support::test_helpers::market_helper::{btc_usdc, link_usdc};
use zkx_support::test_helpers::trading_helper::setup_fee;
use zkx_support::traits::{FieldElementExt, Hashable, U256Ext};
use zkx_support::types::{
	BaseFee, Direction, Discount, ExtendedMarket, HashType, Market, Order, OrderType, Position,
	Side, SignatureInfo, TimeInForce, TradingAccountMinimal,
};

const ORDER_ID_1: u128 = 200_u128;
const ORDER_ID_2: u128 = 201_u128;
const ORDER_ID_3: u128 = 202_u128;
const ORDER_ID_4: u128 = 203_u128;
const ORDER_ID_5: u128 = 204_u128;
const ORDER_ID_6: u128 = 205_u128;

fn setup() -> sp_io::TestExternalities {
	// Create a new test environment
	let mut env = new_test_ext();

	// Set the block number in the environment
	env.execute_with(|| {
		// Set the block number
		System::set_block_number(1);
		assert_ok!(Timestamp::set(None.into(), 100));

		// Set the assets in the system
		assert_ok!(Assets::replace_all_assets(
			RuntimeOrigin::signed(1),
			vec![eth(), usdc(), link(), btc()]
		));
		assert_ok!(Markets::replace_all_markets(
			RuntimeOrigin::signed(1),
			vec![btc_usdc(), link_usdc()]
		));

		// Add accounts to the system
		assert_ok!(TradingAccounts::add_accounts(
			RuntimeOrigin::signed(1),
			vec![alice(), bob(), charlie(), dave()]
		));
	});

	env
}

#[test]
// basic open trade without any leverage
fn it_works_for_open_trade_simple() {
	let mut env = setup();

	env.execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());

		// market id
		let market_id = btc_usdc().market.id;

		// Create orders
		let alice_order = Order::new().sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new()
			.set_account_id(bob_id)
			.set_order_id(201_u128)
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
			vec![alice_order, bob_order]
		));

		// Check the execution of orders
		let alice_position = Trading::positions(alice_id, (market_id, Direction::Long));
		let expected_position: Position = Position {
			market_id,
			avg_execution_price: 100.into(),
			size: 1.into(),
			direction: Direction::Long,
			side: Side::Buy,
			margin_amount: 100.into(),
			borrowed_amount: 0.into(),
			leverage: 1.into(),
			realized_pnl: 0.into(),
		};
		assert_eq!(expected_position, alice_position);

		let bob_position = Trading::positions(bob_id, (market_id, Direction::Short));
		let expected_position: Position = Position {
			market_id,
			avg_execution_price: 100.into(),
			size: 1.into(),
			direction: Direction::Short,
			side: Side::Buy,
			margin_amount: 100.into(),
			borrowed_amount: 0.into(),
			leverage: 1.into(),
			realized_pnl: 0.into(),
		};
		assert_eq!(expected_position, bob_position);
	});
}

// #[test]
// // basic open trade with leverage
// fn it_works_for_open_trade_with_leverage() {
// 	new_test_ext().execute_with(|| {
// 		let (markets, accounts, private_keys) = setup();
// 		// Go past genesis block so events get deposited
// 		System::set_block_number(1);

// 		let account_id_1: U256 = get_trading_account_id(accounts.clone(), 0);
// 		let account_id_2: U256 = get_trading_account_id(accounts.clone(), 1);

// 		let order_1 = Order {
// 			account_id: account_id_1,
// 			order_id: ORDER_ID_1,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Limit,
// 			direction: Direction::Long,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: 1.into(),
// 			leverage: 5.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};
// 		let order_2 = Order {
// 			account_id: account_id_2,
// 			order_id: ORDER_ID_2,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Market,
// 			direction: Direction::Short,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: 1.into(),
// 			leverage: 5.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};

// 		let order_1 = sign_order(order_1, private_keys[0]);
// 		let order_2 = sign_order(order_2, private_keys[1]);
// 		let orders: Vec<Order> = vec![order_1, order_2];

// 		assert_ok!(Trading::execute_trade(
// 			RuntimeOrigin::signed(1),
// 			U256::from(1_u8),
// 			1.into(),
// 			markets[0].market.id,
// 			100.into(),
// 			orders
// 		));

// 		let position1 = Trading::positions(account_id_1, (markets[0].market.id, Direction::Long));
// 		let expected_position: Position = Position {
// 			market_id: markets[0].market.id,
// 			avg_execution_price: 100.into(),
// 			size: 1.into(),
// 			direction: Direction::Long,
// 			side: Side::Buy,
// 			margin_amount: 20.into(),
// 			borrowed_amount: 80.into(),
// 			leverage: 5.into(),
// 			realized_pnl: 0.into(),
// 		};
// 		assert_eq!(expected_position, position1);

// 		let position2 = Trading::positions(account_id_2, (markets[0].market.id, Direction::Short));
// 		let expected_position: Position = Position {
// 			market_id: markets[0].market.id,
// 			avg_execution_price: 100.into(),
// 			size: 1.into(),
// 			direction: Direction::Short,
// 			side: Side::Buy,
// 			margin_amount: 20.into(),
// 			borrowed_amount: 80.into(),
// 			leverage: 5.into(),
// 			realized_pnl: 0.into(),
// 		};
// 		assert_eq!(expected_position, position2);
// 	});
// }

// #[test]
// // basic open and close trade without any leverage
// fn it_works_for_close_trade_simple() {
// 	new_test_ext().execute_with(|| {
// 		let (markets, accounts, private_keys) = setup();
// 		// Go past genesis block so events get deposited
// 		System::set_block_number(1);

// 		let account_id_1: U256 = get_trading_account_id(accounts.clone(), 0);
// 		let account_id_2: U256 = get_trading_account_id(accounts.clone(), 1);

// 		let order_1 = Order {
// 			account_id: account_id_1,
// 			order_id: ORDER_ID_1,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Limit,
// 			direction: Direction::Long,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: 1.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};
// 		let order_2 = Order {
// 			account_id: account_id_2,
// 			order_id: ORDER_ID_2,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Market,
// 			direction: Direction::Short,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: 1.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};

// 		let order_1 = sign_order(order_1, private_keys[0]);
// 		let order_2 = sign_order(order_2, private_keys[1]);
// 		let orders: Vec<Order> = vec![order_1, order_2];

// 		assert_ok!(Trading::execute_trade(
// 			RuntimeOrigin::signed(1),
// 			U256::from(1_u8),
// 			1.into(),
// 			markets[0].market.id,
// 			100.into(),
// 			orders
// 		));

// 		// Close orders
// 		let order_3 = Order {
// 			account_id: account_id_1,
// 			order_id: ORDER_ID_3,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Limit,
// 			direction: Direction::Long,
// 			side: Side::Sell,
// 			price: 105.into(),
// 			size: 1.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};
// 		let order_4 = Order {
// 			account_id: account_id_2,
// 			order_id: ORDER_ID_4,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Market,
// 			direction: Direction::Short,
// 			side: Side::Sell,
// 			price: 100.into(),
// 			size: 1.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};

// 		let order_3 = sign_order(order_3, private_keys[0]);
// 		let order_4 = sign_order(order_4, private_keys[1]);
// 		let orders: Vec<Order> = vec![order_3, order_4];

// 		assert_ok!(Trading::execute_trade(
// 			RuntimeOrigin::signed(1),
// 			U256::from(2_u8),
// 			1.into(),
// 			markets[0].market.id,
// 			105.into(),
// 			orders
// 		));

// 		let usdc_id: u128 = usdc().asset.id;
// 		let balance_1 = TradingAccounts::balances(account_id_1, usdc_id);
// 		assert_eq!(balance_1, 10005.into());
// 		let balance_2 = TradingAccounts::balances(account_id_2, usdc_id);
// 		assert_eq!(balance_2, 9995.into());
// 		let locked_1 = TradingAccounts::locked_margin(account_id_1, usdc_id);
// 		assert_eq!(locked_1, 0.into());

// 		// let event_record: frame_system::EventRecord<_, _> = System::events().pop().unwrap();
// 		// println!("Events: {:?}", event_record);
// 	});
// }

// #[test]
// // partially open position by executing in different batches
// fn it_works_for_open_trade_partial_open() {
// 	new_test_ext().execute_with(|| {
// 		let (markets, accounts, private_keys) = setup();
// 		// Go past genesis block so events get deposited
// 		System::set_block_number(1);

// 		let account_id_1: U256 = get_trading_account_id(accounts.clone(), 0);
// 		let account_id_2: U256 = get_trading_account_id(accounts.clone(), 1);

// 		let order_1 = Order {
// 			account_id: account_id_1,
// 			order_id: ORDER_ID_1,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Limit,
// 			direction: Direction::Long,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: 1.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};
// 		let order_2 = Order {
// 			account_id: account_id_2,
// 			order_id: ORDER_ID_2,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Market,
// 			direction: Direction::Short,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: 2.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};

// 		let order_1 = sign_order(order_1, private_keys[0]);
// 		let order_2 = sign_order(order_2, private_keys[1]);
// 		let orders: Vec<Order> = vec![order_1, order_2.clone()];

// 		assert_ok!(Trading::execute_trade(
// 			RuntimeOrigin::signed(1),
// 			U256::from(1_u8),
// 			1.into(),
// 			markets[0].market.id,
// 			100.into(),
// 			orders
// 		));

// 		let order_3 = Order {
// 			account_id: account_id_1,
// 			order_id: ORDER_ID_3,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Limit,
// 			direction: Direction::Long,
// 			side: Side::Buy,
// 			price: 98.into(),
// 			size: 1.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};

// 		let order_3 = sign_order(order_3, private_keys[0]);
// 		let orders: Vec<Order> = vec![order_3, order_2];

// 		assert_ok!(Trading::execute_trade(
// 			RuntimeOrigin::signed(1),
// 			U256::from(2_u8),
// 			1.into(),
// 			markets[0].market.id,
// 			98.into(),
// 			orders
// 		));

// 		let position1 = Trading::positions(account_id_2, (markets[0].market.id, Direction::Short));
// 		let expected_position: Position = Position {
// 			market_id: markets[0].market.id,
// 			avg_execution_price: 99.into(),
// 			size: 2.into(),
// 			direction: Direction::Short,
// 			side: Side::Buy,
// 			margin_amount: 198.into(),
// 			borrowed_amount: 0.into(),
// 			leverage: 1.into(),
// 			realized_pnl: 0.into(),
// 		};
// 		assert_eq!(expected_position, position1);
// 	});
// }

// #[test]
// // partially close position by executing in different batches
// fn it_works_for_close_trade_partial_close() {
// 	new_test_ext().execute_with(|| {
// 		let (markets, accounts, private_keys) = setup();
// 		// Go past genesis block so events get deposited
// 		System::set_block_number(1);

// 		let account_id_1: U256 = get_trading_account_id(accounts.clone(), 0);
// 		let account_id_2: U256 = get_trading_account_id(accounts.clone(), 1);

// 		let order_1 = Order {
// 			account_id: account_id_1,
// 			order_id: ORDER_ID_1,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Limit,
// 			direction: Direction::Long,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: 2.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};
// 		let order_2 = Order {
// 			account_id: account_id_2,
// 			order_id: ORDER_ID_2,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Market,
// 			direction: Direction::Short,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: 2.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};

// 		let order_1 = sign_order(order_1, private_keys[0]);
// 		let order_2 = sign_order(order_2, private_keys[1]);
// 		let orders: Vec<Order> = vec![order_1, order_2.clone()];

// 		assert_ok!(Trading::execute_trade(
// 			RuntimeOrigin::signed(1),
// 			U256::from(1_u8),
// 			2.into(),
// 			markets[0].market.id,
// 			100.into(),
// 			orders
// 		));

// 		let order_3 = Order {
// 			account_id: account_id_1,
// 			order_id: ORDER_ID_3,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Limit,
// 			direction: Direction::Long,
// 			side: Side::Sell,
// 			price: 104.into(),
// 			size: 1.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};
// 		let order_4 = Order {
// 			account_id: account_id_2,
// 			order_id: ORDER_ID_4,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Market,
// 			direction: Direction::Short,
// 			side: Side::Sell,
// 			price: 100.into(),
// 			size: 2.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};

// 		let order_3 = sign_order(order_3, private_keys[0]);
// 		let order_4 = sign_order(order_4, private_keys[1]);
// 		let orders: Vec<Order> = vec![order_3, order_4.clone()];

// 		assert_ok!(Trading::execute_trade(
// 			RuntimeOrigin::signed(1),
// 			U256::from(2_u8),
// 			1.into(),
// 			markets[0].market.id,
// 			105.into(),
// 			orders
// 		));

// 		let order_5 = Order {
// 			account_id: account_id_1,
// 			order_id: ORDER_ID_5,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Limit,
// 			direction: Direction::Long,
// 			side: Side::Sell,
// 			price: 98.into(),
// 			size: 1.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};

// 		let order_5 = sign_order(order_5, private_keys[0]);
// 		let orders: Vec<Order> = vec![order_5, order_4];

// 		assert_ok!(Trading::execute_trade(
// 			RuntimeOrigin::signed(1),
// 			U256::from(3_u8),
// 			1.into(),
// 			markets[0].market.id,
// 			100.into(),
// 			orders
// 		));

// 		let usdc_id: u128 = usdc().asset.id;
// 		let balance_1 = TradingAccounts::balances(account_id_2, usdc_id);
// 		assert_eq!(balance_1, 9998.into());
// 	});
// }

// #[test]
// // trade batch with multiple makers and a taker
// fn it_works_for_open_trade_multiple_makers() {
// 	new_test_ext().execute_with(|| {
// 		let (markets, accounts, private_keys) = setup();
// 		// Go past genesis block so events get deposited
// 		System::set_block_number(1);

// 		let account_id_1: U256 = get_trading_account_id(accounts.clone(), 0);
// 		let account_id_2: U256 = get_trading_account_id(accounts.clone(), 1);
// 		let account_id_3: U256 = get_trading_account_id(accounts.clone(), 2);
// 		let account_id_4: U256 = get_trading_account_id(accounts.clone(), 3);

// 		let order_1 = Order {
// 			account_id: account_id_1,
// 			order_id: ORDER_ID_1,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Limit,
// 			direction: Direction::Short,
// 			side: Side::Buy,
// 			price: 105.into(),
// 			size: 1.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};
// 		let order_2 = Order {
// 			account_id: account_id_2,
// 			order_id: ORDER_ID_2,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Limit,
// 			direction: Direction::Short,
// 			side: Side::Buy,
// 			price: 99.into(),
// 			size: 1.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};
// 		let order_3 = Order {
// 			account_id: account_id_3,
// 			order_id: ORDER_ID_3,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Limit,
// 			direction: Direction::Short,
// 			side: Side::Buy,
// 			price: 104.into(),
// 			size: 2.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};
// 		let order_4 = Order {
// 			account_id: account_id_4,
// 			order_id: ORDER_ID_4,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Market,
// 			direction: Direction::Long,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: 3.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};

// 		let order_1 = sign_order(order_1, private_keys[0]);
// 		let order_2 = sign_order(order_2, private_keys[1]);
// 		let order_3 = sign_order(order_3, private_keys[2]);
// 		let order_4 = sign_order(order_4, private_keys[3]);
// 		let orders: Vec<Order> = vec![order_1, order_2, order_3, order_4];

// 		assert_ok!(Trading::execute_trade(
// 			RuntimeOrigin::signed(1),
// 			U256::from(1_u8),
// 			3.into(),
// 			markets[0].market.id,
// 			100.into(),
// 			orders
// 		));

// 		let event_record: frame_system::EventRecord<_, _> = System::events().pop().unwrap();
// 		println!("Events: {:?}", event_record);
// 	});
// }

// #[test]
// #[should_panic(expected = "TradeBatchError")]
// // trade batch with previously executed batch_id
// fn it_reverts_for_trade_with_same_batch_id() {
// 	new_test_ext().execute_with(|| {
// 		let (markets, accounts, private_keys) = setup();
// 		// Go past genesis block so events get deposited
// 		System::set_block_number(1);

// 		let account_id_1: U256 = get_trading_account_id(accounts.clone(), 0);
// 		let account_id_2: U256 = get_trading_account_id(accounts.clone(), 1);

// 		let order_1 = Order {
// 			account_id: account_id_1,
// 			order_id: ORDER_ID_1,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Limit,
// 			direction: Direction::Long,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: 2.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};
// 		let order_2 = Order {
// 			account_id: account_id_2,
// 			order_id: ORDER_ID_2,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Market,
// 			direction: Direction::Short,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: 2.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};

// 		let order_1 = sign_order(order_1, private_keys[0]);
// 		let order_2 = sign_order(order_2, private_keys[1]);
// 		let orders: Vec<Order> = vec![order_1, order_2];

// 		assert_ok!(Trading::execute_trade(
// 			RuntimeOrigin::signed(1),
// 			U256::from(1_u8),
// 			2.into(),
// 			markets[0].market.id,
// 			100.into(),
// 			orders
// 		));

// 		let order_3 = Order {
// 			account_id: account_id_1,
// 			order_id: ORDER_ID_3,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Limit,
// 			direction: Direction::Long,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: 1.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};
// 		let order_4 = Order {
// 			account_id: account_id_2,
// 			order_id: ORDER_ID_4,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Market,
// 			direction: Direction::Short,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: 1.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};

// 		let order_3 = sign_order(order_3, private_keys[0]);
// 		let order_4 = sign_order(order_4, private_keys[1]);
// 		let orders: Vec<Order> = vec![order_3, order_4];

// 		assert_ok!(Trading::execute_trade(
// 			RuntimeOrigin::signed(1),
// 			U256::from(1_u8),
// 			1.into(),
// 			markets[0].market.id,
// 			100.into(),
// 			orders
// 		));
// 	});
// }

// #[test]
// #[should_panic(expected = "TradeBatchError")]
// // trade batch with invalid market_id
// fn it_reverts_for_trade_with_invalid_market() {
// 	new_test_ext().execute_with(|| {
// 		let (markets, accounts, private_keys) = setup();
// 		// Go past genesis block so events get deposited
// 		System::set_block_number(1);

// 		let account_id_1: U256 = get_trading_account_id(accounts.clone(), 0);
// 		let account_id_2: U256 = get_trading_account_id(accounts.clone(), 1);

// 		let order_1 = Order {
// 			account_id: account_id_1,
// 			order_id: ORDER_ID_1,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Limit,
// 			direction: Direction::Long,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: 2.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};
// 		let order_2 = Order {
// 			account_id: account_id_2,
// 			order_id: ORDER_ID_2,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Market,
// 			direction: Direction::Short,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: 2.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};

// 		let order_1 = sign_order(order_1, private_keys[0]);
// 		let order_2 = sign_order(order_2, private_keys[1]);
// 		let orders: Vec<Order> = vec![order_1, order_2];

// 		assert_ok!(Trading::execute_trade(
// 			RuntimeOrigin::signed(1),
// 			U256::from(1_u8),
// 			2.into(),
// 			123,
// 			100.into(),
// 			orders
// 		));
// 	});
// }

// #[test]
// #[should_panic(expected = "TradeBatchError")]
// // trade batch with quantity_locked as 0
// fn it_reverts_for_trade_with_quantity_locked_zero() {
// 	new_test_ext().execute_with(|| {
// 		let (markets, accounts, private_keys) = setup();
// 		// Go past genesis block so events get deposited
// 		System::set_block_number(1);

// 		let account_id_1: U256 = get_trading_account_id(accounts.clone(), 0);
// 		let account_id_2: U256 = get_trading_account_id(accounts.clone(), 1);

// 		let order_1 = Order {
// 			account_id: account_id_1,
// 			order_id: ORDER_ID_1,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Limit,
// 			direction: Direction::Long,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: 2.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};
// 		let order_2 = Order {
// 			account_id: account_id_2,
// 			order_id: ORDER_ID_2,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Market,
// 			direction: Direction::Short,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: 2.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};

// 		let order_1 = sign_order(order_1, private_keys[0]);
// 		let order_2 = sign_order(order_2, private_keys[1]);
// 		let orders: Vec<Order> = vec![order_1, order_2];

// 		assert_ok!(Trading::execute_trade(
// 			RuntimeOrigin::signed(1),
// 			U256::from(1_u8),
// 			0.into(),
// 			markets[0].market.id,
// 			100.into(),
// 			orders
// 		));
// 	});
// }

// #[test]
// #[should_panic(expected = "TradeBatchError")]
// // Taker tries to close a position which is already completely closed
// fn it_reverts_when_taker_tries_to_close_already_closed_position() {
// 	new_test_ext().execute_with(|| {
// 		let (markets, accounts, private_keys) = setup();
// 		// Go past genesis block so events get deposited
// 		System::set_block_number(1);

// 		let account_id_1: U256 = get_trading_account_id(accounts.clone(), 0);
// 		let account_id_2: U256 = get_trading_account_id(accounts.clone(), 1);

// 		let order_1 = Order {
// 			account_id: account_id_1,
// 			order_id: ORDER_ID_1,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Limit,
// 			direction: Direction::Long,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: 2.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};
// 		let order_2 = Order {
// 			account_id: account_id_2,
// 			order_id: ORDER_ID_2,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Market,
// 			direction: Direction::Short,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: 2.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};

// 		let order_1 = sign_order(order_1, private_keys[0]);
// 		let order_2 = sign_order(order_2, private_keys[1]);
// 		let orders: Vec<Order> = vec![order_1, order_2.clone()];

// 		assert_ok!(Trading::execute_trade(
// 			RuntimeOrigin::signed(1),
// 			U256::from(1_u8),
// 			2.into(),
// 			markets[0].market.id,
// 			100.into(),
// 			orders
// 		));

// 		let order_3 = Order {
// 			account_id: account_id_1,
// 			order_id: ORDER_ID_3,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Limit,
// 			direction: Direction::Long,
// 			side: Side::Sell,
// 			price: 104.into(),
// 			size: 2.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};
// 		let order_4 = Order {
// 			account_id: account_id_2,
// 			order_id: ORDER_ID_4,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Market,
// 			direction: Direction::Short,
// 			side: Side::Sell,
// 			price: 100.into(),
// 			size: 2.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};

// 		let order_3 = sign_order(order_3, private_keys[0]);
// 		let order_4 = sign_order(order_4, private_keys[1]);
// 		let orders: Vec<Order> = vec![order_3, order_4.clone()];

// 		assert_ok!(Trading::execute_trade(
// 			RuntimeOrigin::signed(1),
// 			U256::from(2_u8),
// 			2.into(),
// 			markets[0].market.id,
// 			105.into(),
// 			orders
// 		));

// 		let order_5 = Order {
// 			account_id: account_id_1,
// 			order_id: ORDER_ID_5,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Limit,
// 			direction: Direction::Short,
// 			side: Side::Buy,
// 			price: 98.into(),
// 			size: 1.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};
// 		let order_6 = Order {
// 			account_id: account_id_2,
// 			order_id: ORDER_ID_6,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Limit,
// 			direction: Direction::Short,
// 			side: Side::Sell,
// 			price: 98.into(),
// 			size: 1.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};

// 		let order_5 = sign_order(order_5, private_keys[0]);
// 		let order_6 = sign_order(order_6, private_keys[1]);
// 		let orders: Vec<Order> = vec![order_5, order_6];

// 		assert_ok!(Trading::execute_trade(
// 			RuntimeOrigin::signed(1),
// 			U256::from(3_u8),
// 			1.into(),
// 			markets[0].market.id,
// 			100.into(),
// 			orders
// 		));
// 	});
// }

// #[test]
// // Non registered user tries to open a position
// fn it_produces_error_when_user_not_registered() {
// 	new_test_ext().execute_with(|| {
// 		let (markets, accounts, private_keys) = setup();
// 		// Go past genesis block so events get deposited
// 		System::set_block_number(1);

// 		let _account_id_1: U256 = get_trading_account_id(accounts.clone(), 0);
// 		let account_id_2: U256 = get_trading_account_id(accounts.clone(), 1);

// 		let order_1 = Order {
// 			account_id: 1.into(),
// 			order_id: ORDER_ID_1,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Limit,
// 			direction: Direction::Long,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: 1.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};
// 		let order_2 = Order {
// 			account_id: account_id_2,
// 			order_id: ORDER_ID_2,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Market,
// 			direction: Direction::Short,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: 1.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};

// 		let order_1 = sign_order(order_1, private_keys[0]);
// 		let order_2 = sign_order(order_2, private_keys[1]);
// 		let orders: Vec<Order> = vec![order_1, order_2];

// 		assert_ok!(Trading::execute_trade(
// 			RuntimeOrigin::signed(1),
// 			U256::from(1_u8),
// 			1.into(),
// 			markets[0].market.id,
// 			100.into(),
// 			orders
// 		));

// 		System::assert_has_event(Event::OrderError { order_id: 200, error_code: 510 }.into());
// 	});
// }

// #[test]
// // Tries to open a position with size lesser than allowed minimum order size
// fn it_produces_error_when_size_too_small() {
// 	new_test_ext().execute_with(|| {
// 		let (markets, accounts, private_keys) = setup();
// 		// Go past genesis block so events get deposited
// 		System::set_block_number(1);

// 		let account_id_1: U256 = get_trading_account_id(accounts.clone(), 0);
// 		let account_id_2: U256 = get_trading_account_id(accounts.clone(), 1);

// 		let order_1 = Order {
// 			account_id: account_id_1,
// 			order_id: ORDER_ID_1,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Limit,
// 			direction: Direction::Long,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: FixedI128::from_inner(500000000000000000),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};
// 		let order_2 = Order {
// 			account_id: account_id_2,
// 			order_id: ORDER_ID_2,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Market,
// 			direction: Direction::Short,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: 1.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};

// 		let order_1 = sign_order(order_1, private_keys[0]);
// 		let order_2 = sign_order(order_2, private_keys[1]);
// 		let orders: Vec<Order> = vec![order_1, order_2];

// 		assert_ok!(Trading::execute_trade(
// 			RuntimeOrigin::signed(1),
// 			U256::from(1_u8),
// 			1.into(),
// 			markets[0].market.id,
// 			100.into(),
// 			orders
// 		));

// 		System::assert_has_event(Event::OrderError { order_id: 200, error_code: 505 }.into());
// 	});
// }

// #[test]
// // Tries to open a position with different market_id compared to the one passed in argument
// fn it_produces_error_when_market_id_is_different() {
// 	new_test_ext().execute_with(|| {
// 		let (markets, accounts, private_keys) = setup();
// 		// Go past genesis block so events get deposited
// 		System::set_block_number(1);

// 		let account_id_1: U256 = get_trading_account_id(accounts.clone(), 0);
// 		let account_id_2: U256 = get_trading_account_id(accounts.clone(), 1);

// 		let order_1 = Order {
// 			account_id: account_id_1,
// 			order_id: ORDER_ID_1,
// 			market_id: 789,
// 			order_type: OrderType::Limit,
// 			direction: Direction::Long,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: 1.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};
// 		let order_2 = Order {
// 			account_id: account_id_2,
// 			order_id: ORDER_ID_2,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Market,
// 			direction: Direction::Short,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: 1.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};

// 		let order_1 = sign_order(order_1, private_keys[0]);
// 		let order_2 = sign_order(order_2, private_keys[1]);
// 		let orders: Vec<Order> = vec![order_1, order_2];

// 		assert_ok!(Trading::execute_trade(
// 			RuntimeOrigin::signed(1),
// 			U256::from(1_u8),
// 			1.into(),
// 			markets[0].market.id,
// 			100.into(),
// 			orders
// 		));

// 		System::assert_has_event(Event::OrderError { order_id: 200, error_code: 504 }.into());
// 	});
// }

// #[test]
// // Tries to open a position leverage more than currently allowed leverage
// fn it_produces_error_when_leverage_is_invalid() {
// 	new_test_ext().execute_with(|| {
// 		let (markets, accounts, private_keys) = setup();
// 		// Go past genesis block so events get deposited
// 		System::set_block_number(1);

// 		let account_id_1: U256 = get_trading_account_id(accounts.clone(), 0);
// 		let account_id_2: U256 = get_trading_account_id(accounts.clone(), 1);

// 		let order_1 = Order {
// 			account_id: account_id_1,
// 			order_id: ORDER_ID_1,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Limit,
// 			direction: Direction::Long,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: 1.into(),
// 			leverage: 9.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};
// 		let order_2 = Order {
// 			account_id: account_id_2,
// 			order_id: ORDER_ID_2,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Market,
// 			direction: Direction::Short,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: 1.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};

// 		let order_1 = sign_order(order_1, private_keys[0]);
// 		let order_2 = sign_order(order_2, private_keys[1]);
// 		let orders: Vec<Order> = vec![order_1, order_2];

// 		assert_ok!(Trading::execute_trade(
// 			RuntimeOrigin::signed(1),
// 			U256::from(1_u8),
// 			1.into(),
// 			markets[0].market.id,
// 			100.into(),
// 			orders
// 		));

// 		System::assert_has_event(Event::OrderError { order_id: 200, error_code: 502 }.into());
// 	});
// }

// #[test]
// // Tries to open a position with invalid signature
// fn it_produces_error_when_signature_is_invalid() {
// 	new_test_ext().execute_with(|| {
// 		let (markets, accounts, private_keys) = setup();
// 		// Go past genesis block so events get deposited
// 		System::set_block_number(1);

// 		let account_id_1: U256 = get_trading_account_id(accounts.clone(), 0);
// 		let account_id_2: U256 = get_trading_account_id(accounts.clone(), 1);

// 		let order_1 = Order {
// 			account_id: account_id_1,
// 			order_id: ORDER_ID_1,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Limit,
// 			direction: Direction::Long,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: 1.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};
// 		let order_2 = Order {
// 			account_id: account_id_2,
// 			order_id: ORDER_ID_2,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Market,
// 			direction: Direction::Short,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: 1.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 123.into(),
// 			sig_s: 456.into(),
// 			hash_type: HashType::Pedersen,
// 		};

// 		// let order_1 = sign_order(order_1, private_keys[0]);
// 		let order_2 = sign_order(order_2, private_keys[1]);
// 		let orders: Vec<Order> = vec![order_1, order_2];

// 		assert_ok!(Trading::execute_trade(
// 			RuntimeOrigin::signed(1),
// 			U256::from(1_u8),
// 			1.into(),
// 			markets[0].market.id,
// 			100.into(),
// 			orders
// 		));

// 		System::assert_has_event(Event::OrderError { order_id: 200, error_code: 536 }.into());
// 	});
// }

// #[test]
// // 2nd maker order with side and direction that does not match with the first maker
// fn it_produces_error_for_maker_when_side_and_direction_is_invalid() {
// 	new_test_ext().execute_with(|| {
// 		let (markets, accounts, private_keys) = setup();
// 		// Go past genesis block so events get deposited
// 		System::set_block_number(1);

// 		let account_id_1: U256 = get_trading_account_id(accounts.clone(), 0);
// 		let account_id_2: U256 = get_trading_account_id(accounts.clone(), 1);
// 		let _account_id_3: U256 = get_trading_account_id(accounts.clone(), 2);
// 		let account_id_4: U256 = get_trading_account_id(accounts.clone(), 3);

// 		let order_1 = Order {
// 			account_id: account_id_1,
// 			order_id: ORDER_ID_1,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Limit,
// 			direction: Direction::Short,
// 			side: Side::Buy,
// 			price: 105.into(),
// 			size: 1.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};
// 		let order_2 = Order {
// 			account_id: account_id_2,
// 			order_id: ORDER_ID_2,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Limit,
// 			direction: Direction::Long,
// 			side: Side::Buy,
// 			price: 99.into(),
// 			size: 1.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};
// 		let order_4 = Order {
// 			account_id: account_id_4,
// 			order_id: ORDER_ID_4,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Market,
// 			direction: Direction::Long,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: 3.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};

// 		let order_1 = sign_order(order_1, private_keys[0]);
// 		let order_2 = sign_order(order_2, private_keys[1]);
// 		let order_4 = sign_order(order_4, private_keys[3]);
// 		let orders: Vec<Order> = vec![order_1, order_2, order_4];

// 		assert_ok!(Trading::execute_trade(
// 			RuntimeOrigin::signed(1),
// 			U256::from(1_u8),
// 			3.into(),
// 			markets[0].market.id,
// 			100.into(),
// 			orders
// 		));

// 		System::assert_has_event(Event::OrderError { order_id: 201, error_code: 512 }.into());
// 	});
// }

// #[test]
// // Maker order type is not limit
// fn it_produces_error_when_maker_is_market_order() {
// 	new_test_ext().execute_with(|| {
// 		let (markets, accounts, private_keys) = setup();
// 		// Go past genesis block so events get deposited
// 		System::set_block_number(1);

// 		let account_id_1: U256 = get_trading_account_id(accounts.clone(), 0);
// 		let account_id_2: U256 = get_trading_account_id(accounts.clone(), 1);

// 		let order_1 = Order {
// 			account_id: account_id_1,
// 			order_id: ORDER_ID_1,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Market,
// 			direction: Direction::Long,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: 1.into(),
// 			leverage: 8.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};
// 		let order_2 = Order {
// 			account_id: account_id_2,
// 			order_id: ORDER_ID_2,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Market,
// 			direction: Direction::Short,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: 1.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};

// 		let order_1 = sign_order(order_1, private_keys[0]);
// 		let order_2 = sign_order(order_2, private_keys[1]);
// 		let orders: Vec<Order> = vec![order_1, order_2];

// 		assert_ok!(Trading::execute_trade(
// 			RuntimeOrigin::signed(1),
// 			U256::from(1_u8),
// 			1.into(),
// 			markets[0].market.id,
// 			100.into(),
// 			orders
// 		));

// 		System::assert_has_event(Event::OrderError { order_id: 200, error_code: 518 }.into());
// 	});
// }

// #[test]
// // Maker tries to close a position which is already completely closed
// fn it_reverts_when_maker_tries_to_close_already_closed_position() {
// 	new_test_ext().execute_with(|| {
// 		let (markets, accounts, private_keys) = setup();
// 		// Go past genesis block so events get deposited
// 		System::set_block_number(1);

// 		let account_id_1: U256 = get_trading_account_id(accounts.clone(), 0);
// 		let account_id_2: U256 = get_trading_account_id(accounts.clone(), 1);

// 		let order_1 = Order {
// 			account_id: account_id_1,
// 			order_id: ORDER_ID_1,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Limit,
// 			direction: Direction::Long,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: 2.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};
// 		let order_2 = Order {
// 			account_id: account_id_2,
// 			order_id: ORDER_ID_2,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Market,
// 			direction: Direction::Short,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: 2.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};

// 		let order_1 = sign_order(order_1, private_keys[0]);
// 		let order_2 = sign_order(order_2, private_keys[1]);
// 		let orders: Vec<Order> = vec![order_1, order_2.clone()];

// 		assert_ok!(Trading::execute_trade(
// 			RuntimeOrigin::signed(1),
// 			U256::from(1_u8),
// 			2.into(),
// 			markets[0].market.id,
// 			100.into(),
// 			orders
// 		));

// 		let order_3 = Order {
// 			account_id: account_id_1,
// 			order_id: ORDER_ID_3,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Limit,
// 			direction: Direction::Long,
// 			side: Side::Sell,
// 			price: 104.into(),
// 			size: 2.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};
// 		let order_4 = Order {
// 			account_id: account_id_2,
// 			order_id: ORDER_ID_4,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Market,
// 			direction: Direction::Short,
// 			side: Side::Sell,
// 			price: 100.into(),
// 			size: 2.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};

// 		let order_3 = sign_order(order_3, private_keys[0]);
// 		let order_4 = sign_order(order_4, private_keys[1]);
// 		let orders: Vec<Order> = vec![order_3, order_4.clone()];

// 		assert_ok!(Trading::execute_trade(
// 			RuntimeOrigin::signed(1),
// 			U256::from(2_u8),
// 			2.into(),
// 			markets[0].market.id,
// 			105.into(),
// 			orders
// 		));

// 		let order_5 = Order {
// 			account_id: account_id_1,
// 			order_id: ORDER_ID_5,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Limit,
// 			direction: Direction::Long,
// 			side: Side::Sell,
// 			price: 98.into(),
// 			size: 1.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};
// 		let order_6 = Order {
// 			account_id: account_id_2,
// 			order_id: ORDER_ID_6,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Limit,
// 			direction: Direction::Long,
// 			side: Side::Buy,
// 			price: 98.into(),
// 			size: 1.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};

// 		let order_5 = sign_order(order_5, private_keys[0]);
// 		let order_6 = sign_order(order_6, private_keys[1]);
// 		let orders: Vec<Order> = vec![order_5, order_6];

// 		assert_ok!(Trading::execute_trade(
// 			RuntimeOrigin::signed(1),
// 			U256::from(3_u8),
// 			1.into(),
// 			markets[0].market.id,
// 			100.into(),
// 			orders
// 		));

// 		let event_record: frame_system::EventRecord<_, _> = System::events().pop().unwrap();
// 		println!("Events: {:?}", event_record);

// 		System::assert_has_event(Event::OrderError { order_id: 204, error_code: 524 }.into());
// 	});
// }

// #[test]
// #[should_panic(expected = "TradeBatchError")]
// // taker order with side and direction that does not match with the maker
// fn it_produces_error_for_taker_when_side_and_direction_is_invalid() {
// 	new_test_ext().execute_with(|| {
// 		let (markets, accounts, private_keys) = setup();
// 		// Go past genesis block so events get deposited
// 		System::set_block_number(1);

// 		let account_id_1: U256 = get_trading_account_id(accounts.clone(), 0);
// 		let account_id_4: U256 = get_trading_account_id(accounts.clone(), 3);

// 		let order_1 = Order {
// 			account_id: account_id_1,
// 			order_id: ORDER_ID_1,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Limit,
// 			direction: Direction::Short,
// 			side: Side::Buy,
// 			price: 105.into(),
// 			size: 1.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};
// 		let order_4 = Order {
// 			account_id: account_id_4,
// 			order_id: ORDER_ID_4,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Market,
// 			direction: Direction::Long,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: 3.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};

// 		let order_1 = sign_order(order_1, private_keys[0]);
// 		let order_4 = sign_order(order_4, private_keys[3]);
// 		let orders: Vec<Order> = vec![order_1, order_4];

// 		assert_ok!(Trading::execute_trade(
// 			RuntimeOrigin::signed(1),
// 			U256::from(1_u8),
// 			3.into(),
// 			markets[0].market.id,
// 			100.into(),
// 			orders
// 		));

// 		let order_1 = Order {
// 			account_id: account_id_1,
// 			order_id: ORDER_ID_1,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Limit,
// 			direction: Direction::Short,
// 			side: Side::Buy,
// 			price: 105.into(),
// 			size: 1.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};
// 		let order_4 = Order {
// 			account_id: account_id_4,
// 			order_id: ORDER_ID_4,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Market,
// 			direction: Direction::Long,
// 			side: Side::Sell,
// 			price: 100.into(),
// 			size: 3.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};

// 		let order_1 = sign_order(order_1, private_keys[0]);
// 		let order_4 = sign_order(order_4, private_keys[3]);
// 		let orders: Vec<Order> = vec![order_1, order_4];

// 		assert_ok!(Trading::execute_trade(
// 			RuntimeOrigin::signed(1),
// 			U256::from(2_u8),
// 			3.into(),
// 			markets[0].market.id,
// 			100.into(),
// 			orders
// 		));

// 		let event_record: frame_system::EventRecord<_, _> = System::events().pop().unwrap();
// 		println!("Events: {:?}", event_record);

// 		System::assert_has_event(Event::OrderError { order_id: 203, error_code: 511 }.into());
// 	});
// }

// #[test]
// #[should_panic(expected = "TradeBatchError")]
// // Taker long buy limit order execution price is invalid
// fn it_produces_error_when_taker_long_buy_limit_price_invalid() {
// 	new_test_ext().execute_with(|| {
// 		let (markets, accounts, private_keys) = setup();
// 		// Go past genesis block so events get deposited
// 		System::set_block_number(1);

// 		let account_id_1: U256 = get_trading_account_id(accounts.clone(), 0);
// 		let account_id_2: U256 = get_trading_account_id(accounts.clone(), 1);

// 		let order_1 = Order {
// 			account_id: account_id_1,
// 			order_id: ORDER_ID_1,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Limit,
// 			direction: Direction::Short,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: 1.into(),
// 			leverage: 8.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};
// 		let order_2 = Order {
// 			account_id: account_id_2,
// 			order_id: ORDER_ID_2,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Limit,
// 			direction: Direction::Long,
// 			side: Side::Buy,
// 			price: 99.into(),
// 			size: 1.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};

// 		let order_1 = sign_order(order_1, private_keys[0]);
// 		let order_2 = sign_order(order_2, private_keys[1]);
// 		let orders: Vec<Order> = vec![order_1, order_2];

// 		assert_ok!(Trading::execute_trade(
// 			RuntimeOrigin::signed(1),
// 			U256::from(1_u8),
// 			1.into(),
// 			markets[0].market.id,
// 			100.into(),
// 			orders
// 		));

// 		System::assert_has_event(Event::OrderError { order_id: 201, error_code: 508 }.into());
// 	});
// }

// #[test]
// #[should_panic(expected = "TradeBatchError")]
// // Taker short buy limit order execution price is invalid
// fn it_produces_error_when_taker_short_buy_limit_price_invalid() {
// 	new_test_ext().execute_with(|| {
// 		let (markets, accounts, private_keys) = setup();
// 		// Go past genesis block so events get deposited
// 		System::set_block_number(1);

// 		let account_id_1: U256 = get_trading_account_id(accounts.clone(), 0);
// 		let account_id_2: U256 = get_trading_account_id(accounts.clone(), 1);

// 		let order_1 = Order {
// 			account_id: account_id_1,
// 			order_id: ORDER_ID_1,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Limit,
// 			direction: Direction::Long,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: 1.into(),
// 			leverage: 8.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};
// 		let order_2 = Order {
// 			account_id: account_id_2,
// 			order_id: ORDER_ID_2,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Limit,
// 			direction: Direction::Short,
// 			side: Side::Buy,
// 			price: 101.into(),
// 			size: 1.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};

// 		let order_1 = sign_order(order_1, private_keys[0]);
// 		let order_2 = sign_order(order_2, private_keys[1]);
// 		let orders: Vec<Order> = vec![order_1, order_2];

// 		assert_ok!(Trading::execute_trade(
// 			RuntimeOrigin::signed(1),
// 			U256::from(1_u8),
// 			1.into(),
// 			markets[0].market.id,
// 			100.into(),
// 			orders
// 		));

// 		System::assert_has_event(Event::OrderError { order_id: 201, error_code: 507 }.into());
// 	});
// }

// #[test]
// #[should_panic(expected = "TradeBatchError")]
// // Taker long buy slippage check
// fn it_produces_error_when_taker_long_buy_price_not_within_slippage() {
// 	new_test_ext().execute_with(|| {
// 		let (markets, accounts, private_keys) = setup();
// 		// Go past genesis block so events get deposited
// 		System::set_block_number(1);

// 		let account_id_1: U256 = get_trading_account_id(accounts.clone(), 0);
// 		let account_id_2: U256 = get_trading_account_id(accounts.clone(), 1);

// 		let order_1 = Order {
// 			account_id: account_id_1,
// 			order_id: ORDER_ID_1,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Limit,
// 			direction: Direction::Short,
// 			side: Side::Buy,
// 			price: 111.into(),
// 			size: 1.into(),
// 			leverage: 8.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};
// 		let order_2 = Order {
// 			account_id: account_id_2,
// 			order_id: ORDER_ID_2,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Market,
// 			direction: Direction::Long,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: 1.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};

// 		let order_1 = sign_order(order_1, private_keys[0]);
// 		let order_2 = sign_order(order_2, private_keys[1]);
// 		let orders: Vec<Order> = vec![order_1, order_2.clone()];

// 		assert_ok!(Trading::execute_trade(
// 			RuntimeOrigin::signed(1),
// 			U256::from(1_u8),
// 			1.into(),
// 			markets[0].market.id,
// 			100.into(),
// 			orders
// 		));
// 	});
// }

// #[test]
// // Taker long buy slippage check when execution price very low
// fn it_works_when_taker_long_buy_price_very_low() {
// 	new_test_ext().execute_with(|| {
// 		let (markets, accounts, private_keys) = setup();
// 		// Go past genesis block so events get deposited
// 		System::set_block_number(1);

// 		let account_id_1: U256 = get_trading_account_id(accounts.clone(), 0);
// 		let account_id_2: U256 = get_trading_account_id(accounts.clone(), 1);

// 		let order_1 = Order {
// 			account_id: account_id_1,
// 			order_id: ORDER_ID_1,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Limit,
// 			direction: Direction::Short,
// 			side: Side::Buy,
// 			price: 80.into(),
// 			size: 1.into(),
// 			leverage: 8.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};
// 		let order_2 = Order {
// 			account_id: account_id_2,
// 			order_id: ORDER_ID_2,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Market,
// 			direction: Direction::Long,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: 1.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};

// 		let order_1 = sign_order(order_1, private_keys[0]);
// 		let order_2 = sign_order(order_2, private_keys[1]);
// 		let orders: Vec<Order> = vec![order_1, order_2.clone()];

// 		assert_ok!(Trading::execute_trade(
// 			RuntimeOrigin::signed(1),
// 			U256::from(1_u8),
// 			1.into(),
// 			markets[0].market.id,
// 			100.into(),
// 			orders
// 		));
// 	});
// }

// #[test]
// fn test_fee_while_opening_order() {
// 	new_test_ext().execute_with(|| {
// 		let (fee_tiers, fee_details, discount_tiers, discount_details) = setup_fee();
// 		// Go past genesis block so events get deposited
// 		System::set_block_number(1);

// 		let side: Side = Side::Buy;
// 		// Dispatch a signed extrinsic.
// 		assert_ok!(TradingFees::update_base_fees_and_discounts(
// 			RuntimeOrigin::signed(1),
// 			side,
// 			fee_tiers,
// 			fee_details.clone(),
// 			discount_tiers,
// 			discount_details.clone()
// 		));

// 		let (markets, accounts, private_keys) = setup();
// 		// Go past genesis block so events get deposited
// 		System::set_block_number(1);

// 		let account_id_1: U256 = get_trading_account_id(accounts.clone(), 0);
// 		let account_id_2: U256 = get_trading_account_id(accounts.clone(), 1);

// 		let order_1 = Order {
// 			account_id: account_id_1,
// 			order_id: ORDER_ID_1,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Limit,
// 			direction: Direction::Long,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: 1.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};
// 		let order_2 = Order {
// 			account_id: account_id_2,
// 			order_id: ORDER_ID_2,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Market,
// 			direction: Direction::Short,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: 1.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};

// 		let order_1 = sign_order(order_1, private_keys[0]);
// 		let order_2 = sign_order(order_2, private_keys[1]);
// 		let orders: Vec<Order> = vec![order_1, order_2];

// 		assert_ok!(Trading::execute_trade(
// 			RuntimeOrigin::signed(1),
// 			U256::from(1_u8),
// 			1.into(),
// 			markets[0].market.id,
// 			100.into(),
// 			orders
// 		));

// 		let usdc_id: u128 = usdc().asset.id;
// 		let balance_1 = TradingAccounts::balances(account_id_1, usdc_id);
// 		assert_eq!(balance_1, FixedI128::from_inner(9998060000000000000000));
// 		let balance_2 = TradingAccounts::balances(account_id_2, usdc_id);
// 		assert_eq!(balance_2, FixedI128::from_inner(9995150000000000000000));

// 		// Close orders
// 		// Since we are closing orders without setting the fee for close orders, fee won't be deducted from balance
// 		let order_3 = Order {
// 			account_id: account_id_1,
// 			order_id: ORDER_ID_3,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Limit,
// 			direction: Direction::Long,
// 			side: Side::Sell,
// 			price: 105.into(),
// 			size: 1.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};
// 		let order_4 = Order {
// 			account_id: account_id_2,
// 			order_id: ORDER_ID_4,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Market,
// 			direction: Direction::Short,
// 			side: Side::Sell,
// 			price: 100.into(),
// 			size: 1.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};

// 		let order_3 = sign_order(order_3, private_keys[0]);
// 		let order_4 = sign_order(order_4, private_keys[1]);
// 		let orders: Vec<Order> = vec![order_3, order_4];

// 		assert_ok!(Trading::execute_trade(
// 			RuntimeOrigin::signed(1),
// 			U256::from(2_u8),
// 			1.into(),
// 			markets[0].market.id,
// 			105.into(),
// 			orders
// 		));

// 		let usdc_id: u128 = usdc().asset.id;
// 		let balance_1 = TradingAccounts::balances(account_id_1, usdc_id);
// 		assert_eq!(balance_1, FixedI128::from_inner(10003060000000000000000));
// 		let balance_2 = TradingAccounts::balances(account_id_2, usdc_id);
// 		assert_eq!(balance_2, FixedI128::from_inner(9990150000000000000000));
// 		let locked_1 = TradingAccounts::locked_margin(account_id_1, usdc_id);
// 		assert_eq!(locked_1, 0.into());
// 	});
// }

// #[test]
// fn test_fee_while_closing_order() {
// 	new_test_ext().execute_with(|| {
// 		let (fee_tiers, fee_details, discount_tiers, discount_details) = setup_fee();
// 		// Go past genesis block so events get deposited
// 		System::set_block_number(1);

// 		let side: Side = Side::Sell;
// 		// Dispatch a signed extrinsic.
// 		assert_ok!(TradingFees::update_base_fees_and_discounts(
// 			RuntimeOrigin::signed(1),
// 			side,
// 			fee_tiers,
// 			fee_details.clone(),
// 			discount_tiers,
// 			discount_details.clone()
// 		));

// 		let (markets, accounts, private_keys) = setup();
// 		// Go past genesis block so events get deposited
// 		System::set_block_number(1);

// 		let account_id_1: U256 = get_trading_account_id(accounts.clone(), 0);
// 		let account_id_2: U256 = get_trading_account_id(accounts.clone(), 1);

// 		let order_1 = Order {
// 			account_id: account_id_1,
// 			order_id: ORDER_ID_1,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Limit,
// 			direction: Direction::Long,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: 1.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};
// 		let order_2 = Order {
// 			account_id: account_id_2,
// 			order_id: ORDER_ID_2,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Market,
// 			direction: Direction::Short,
// 			side: Side::Buy,
// 			price: 100.into(),
// 			size: 1.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};

// 		let order_1 = sign_order(order_1, private_keys[0]);
// 		let order_2 = sign_order(order_2, private_keys[1]);
// 		let orders: Vec<Order> = vec![order_1, order_2];

// 		assert_ok!(Trading::execute_trade(
// 			RuntimeOrigin::signed(1),
// 			U256::from(1_u8),
// 			1.into(),
// 			markets[0].market.id,
// 			100.into(),
// 			orders
// 		));

// 		// Since we are opening orders without setting the fee for open orders, fee won't be deducted from balance
// 		let usdc_id: u128 = usdc().asset.id;
// 		let balance_1 = TradingAccounts::balances(account_id_1, usdc_id);
// 		assert_eq!(balance_1, 10000.into());
// 		let balance_2 = TradingAccounts::balances(account_id_2, usdc_id);
// 		assert_eq!(balance_2, 10000.into());

// 		// Close orders
// 		let order_3 = Order {
// 			account_id: account_id_1,
// 			order_id: ORDER_ID_3,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Limit,
// 			direction: Direction::Long,
// 			side: Side::Sell,
// 			price: 105.into(),
// 			size: 1.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};
// 		let order_4 = Order {
// 			account_id: account_id_2,
// 			order_id: ORDER_ID_4,
// 			market_id: markets[0].market.id,
// 			order_type: OrderType::Market,
// 			direction: Direction::Short,
// 			side: Side::Sell,
// 			price: 100.into(),
// 			size: 1.into(),
// 			leverage: 1.into(),
// 			slippage: FixedI128::from_inner(100000000000000000),
// 			post_only: false,
// 			time_in_force: TimeInForce::GTC,
// 			sig_r: 0.into(),
// 			sig_s: 0.into(),
// 			hash_type: HashType::Pedersen,
// 		};

// 		let order_3 = sign_order(order_3, private_keys[0]);
// 		let order_4 = sign_order(order_4, private_keys[1]);
// 		let orders: Vec<Order> = vec![order_3, order_4];

// 		assert_ok!(Trading::execute_trade(
// 			RuntimeOrigin::signed(1),
// 			U256::from(2_u8),
// 			1.into(),
// 			markets[0].market.id,
// 			105.into(),
// 			orders
// 		));

// 		let usdc_id: u128 = usdc().asset.id;
// 		let balance_1 = TradingAccounts::balances(account_id_1, usdc_id);
// 		assert_eq!(balance_1, FixedI128::from_inner(10002963000000000000000));
// 		let balance_2 = TradingAccounts::balances(account_id_2, usdc_id);
// 		assert_eq!(balance_2, FixedI128::from_inner(9990392500000000000000));
// 		let locked_1 = TradingAccounts::locked_margin(account_id_1, usdc_id);
// 		assert_eq!(locked_1, 0.into());
// 	});
// }
