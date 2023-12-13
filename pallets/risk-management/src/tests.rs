use crate::mock::*;
use frame_support::assert_ok;
use pallet_support::{
	test_helpers::{
		accounts_helper::{
			alice, bob, charlie, dave, eduard, get_private_key, get_trading_account_id,
		},
		asset_helper::{btc, eth, link, usdc},
		market_helper::{btc_usdc, link_usdc},
	},
	types::{Direction, MultiplePrices, Order, OrderType, Position, Side},
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

		// Add liquidator
		Trading::add_liquidator_signer(RuntimeOrigin::signed(1), eduard().pub_key)
			.expect("error while adding signer");
	});

	env
}

#[test]
fn test_liquidation() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());
		let charlie_id: U256 = get_trading_account_id(charlie());

		// market id
		let market_id = btc_usdc().market.id;

		// Create orders
		let alice_order = Order::new(U256::from(201), alice_id)
			.set_size(5.into())
			.set_leverage(5.into())
			.set_price(10000.into())
			.sign_order(get_private_key(alice().pub_key));

		let bob_order = Order::new(U256::from(202), bob_id)
			.set_size(5.into())
			.set_order_type(OrderType::Market)
			.set_direction(Direction::Short)
			.set_leverage(5.into())
			.set_price(10000.into())
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch id
			U256::from(1_u8),
			// size
			5.into(),
			// market
			market_id,
			// price
			10000.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			1699940278000,
		));

		// Decrease the price of the asset
		let mut index_prices: Vec<MultiplePrices> = Vec::new();
		let index_price1 =
			MultiplePrices { market_id, index_price: 5000.into(), mark_price: 5000.into() };
		index_prices.push(index_price1);
		assert_ok!(Prices::update_prices(RuntimeOrigin::signed(1), index_prices, 1699940278000));

		// Place Forced order for liquidation
		let charlie_order = Order::new(U256::from(204), charlie_id)
			.set_size(5.into())
			.set_price(5000.into())
			.set_leverage(5.into())
			.sign_order(get_private_key(charlie().pub_key));

		let alice_forced_order = Order::new(U256::from(203), alice_id)
			.set_size(5.into())
			.set_price(5000.into())
			.set_order_type(OrderType::Forced)
			.set_direction(Direction::Long)
			.set_side(Side::Sell)
			.sign_order_liquidator(get_private_key(eduard().pub_key), eduard().pub_key);

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch id
			U256::from(2_u8),
			// size
			5.into(),
			// market
			market_id,
			// price
			5000.into(),
			// orders
			vec![charlie_order, alice_forced_order],
			// batch_timestamp
			1699940278000,
		));

		let alice_position = Trading::positions(alice_id, (market_id, alice_order.direction));

		let expected_position: Position = Position {
			market_id: 0,
			avg_execution_price: 0.into(),
			size: 0.into(),
			direction: Direction::Long,
			margin_amount: 0.into(),
			borrowed_amount: 0.into(),
			leverage: 0.into(),
			created_timestamp: 0,
			modified_timestamp: 0,
			realized_pnl: 0.into(),
		};
		assert_eq!(expected_position, alice_position);

		let flag = Trading::force_closure_flag(alice_id, btc_usdc().market.asset_collateral);
		assert_eq!(flag.is_none(), true);
	});
}

#[test]
fn test_deleveraging() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());
		let charlie_id: U256 = get_trading_account_id(charlie());

		// market id
		let market_id = btc_usdc().market.id;

		// Create orders
		let alice_order = Order::new(U256::from(201), alice_id)
			.set_size(5.into())
			.set_leverage(5.into())
			.set_price(10000.into())
			.sign_order(get_private_key(alice().pub_key));

		let bob_order = Order::new(U256::from(202), bob_id)
			.set_size(5.into())
			.set_order_type(OrderType::Market)
			.set_direction(Direction::Short)
			.set_leverage(5.into())
			.set_price(10000.into())
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch id
			U256::from(1_u8),
			// size
			5.into(),
			// market
			market_id,
			// price
			10000.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			1699940278000,
		));

		// Decrease the price of the asset
		let mut index_prices: Vec<MultiplePrices> = Vec::new();
		let index_price1 =
			MultiplePrices { market_id, index_price: 8500.into(), mark_price: 8500.into() };
		index_prices.push(index_price1);
		assert_ok!(Prices::update_prices(RuntimeOrigin::signed(1), index_prices, 1699940278000));

		// Place Forced order for deleveraging
		let charlie_order = Order::new(U256::from(204), charlie_id)
			.set_size(5.into())
			.set_price(8500.into())
			.sign_order(get_private_key(charlie().pub_key));

		let alice_forced_order = Order::new(U256::from(203), alice_id)
			.set_size(5.into())
			.set_price(8500.into())
			.set_order_type(OrderType::Forced)
			.set_direction(Direction::Long)
			.set_side(Side::Sell)
			.sign_order_liquidator(get_private_key(eduard().pub_key), eduard().pub_key);

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch id
			U256::from(2_u8),
			// size
			5.into(),
			// market
			market_id,
			// price
			8500.into(),
			// orders
			vec![charlie_order, alice_forced_order],
			// batch_timestamp
			1699940278000,
		));

		let alice_position = Trading::positions(alice_id, (market_id, alice_order.direction));

		let expected_position: Position = Position {
			market_id,
			avg_execution_price: 10000.into(),
			size: FixedI128::from_inner(4700000000000000000),
			direction: Direction::Long,
			margin_amount: 10000.into(),
			borrowed_amount: 37450.into(),
			leverage: FixedI128::from_inner(4750000000000000000),
			created_timestamp: 1699940367,
			modified_timestamp: 1699940367,
			realized_pnl: 0.into(),
		};
		assert_eq!(expected_position, alice_position);

		let flag = Trading::force_closure_flag(alice_id, btc_usdc().market.asset_collateral);
		assert_eq!(flag.is_none(), true);
	});
}

#[test]
fn test_liquidation_after_deleveraging() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());
		let charlie_id: U256 = get_trading_account_id(charlie());
		let dave_id: U256 = get_trading_account_id(dave());

		// market id
		let market_id = btc_usdc().market.id;

		// Create orders
		let alice_order = Order::new(U256::from(201), alice_id)
			.set_size(5.into())
			.set_leverage(5.into())
			.set_price(10000.into())
			.sign_order(get_private_key(alice().pub_key));

		let bob_order = Order::new(U256::from(202), bob_id)
			.set_size(5.into())
			.set_order_type(OrderType::Market)
			.set_direction(Direction::Short)
			.set_leverage(5.into())
			.set_price(10000.into())
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch id
			U256::from(1_u8),
			// size
			5.into(),
			// market
			market_id,
			// price
			10000.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			1699940278000,
		));

		// Decrease the price of the asset
		let mut index_prices: Vec<MultiplePrices> = Vec::new();
		let index_price1 =
			MultiplePrices { market_id, index_price: 8500.into(), mark_price: 8500.into() };
		index_prices.push(index_price1);
		assert_ok!(Prices::update_prices(RuntimeOrigin::signed(1), index_prices, 1699940365000));

		// Place Forced order for deleveraging
		let charlie_order = Order::new(U256::from(204), charlie_id)
			.set_size(5.into())
			.set_price(8500.into())
			.sign_order(get_private_key(charlie().pub_key));

		let alice_forced_order = Order::new(U256::from(203), alice_id)
			.set_size(5.into())
			.set_price(8500.into())
			.set_order_type(OrderType::Forced)
			.set_direction(Direction::Long)
			.set_side(Side::Sell)
			.sign_order_liquidator(get_private_key(eduard().pub_key), eduard().pub_key);

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch id
			U256::from(2_u8),
			// size
			5.into(),
			// market
			market_id,
			// price
			8500.into(),
			// orders
			vec![charlie_order, alice_forced_order],
			// batch_timestamp
			1699940278000,
		));

		// Decrease the price of the asset for liquidation
		let mut index_prices: Vec<MultiplePrices> = Vec::new();
		let index_price1 =
			MultiplePrices { market_id, index_price: 6500.into(), mark_price: 6500.into() };
		index_prices.push(index_price1);
		assert_ok!(Prices::update_prices(RuntimeOrigin::signed(1), index_prices, 1699940366000));
		let _price = Prices::current_price(market_id);

		// Place Forced order for deleveraging
		let dave_order = Order::new(U256::from(206), dave_id)
			.set_size(5.into())
			.set_leverage(5.into())
			.set_price(6500.into())
			.sign_order(get_private_key(dave().pub_key));

		let alice_forced_order = Order::new(U256::from(205), alice_id)
			.set_size(5.into())
			.set_price(6500.into())
			.set_order_type(OrderType::Forced)
			.set_direction(Direction::Long)
			.set_side(Side::Sell)
			.sign_order_liquidator(get_private_key(eduard().pub_key), eduard().pub_key);

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch id
			U256::from(3_u8),
			// size
			5.into(),
			// market
			market_id,
			// price
			6500.into(),
			// orders
			vec![dave_order, alice_forced_order],
			// batch_timestamp
			1699940278000,
		));

		let alice_position = Trading::positions(alice_id, (market_id, alice_order.direction));

		let expected_position: Position = Position {
			market_id: 0,
			avg_execution_price: 0.into(),
			size: 0.into(),
			direction: Direction::Long,
			margin_amount: 0.into(),
			borrowed_amount: 0.into(),
			leverage: 0.into(),
			created_timestamp: 0,
			modified_timestamp: 0,
			realized_pnl: 0.into(),
		};
		assert_eq!(expected_position, alice_position);
	});
}

#[test]
#[should_panic(expected = "TradeBatchError540")]
fn test_invalid_forced_order() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());
		let charlie_id: U256 = get_trading_account_id(charlie());

		// market id
		let market_id = btc_usdc().market.id;

		// Create orders
		let alice_order = Order::new(U256::from(201), alice_id)
			.set_size(5.into())
			.set_leverage(5.into())
			.set_price(10000.into())
			.sign_order(get_private_key(alice().pub_key));

		let bob_order = Order::new(U256::from(202), bob_id)
			.set_size(5.into())
			.set_order_type(OrderType::Market)
			.set_direction(Direction::Short)
			.set_leverage(5.into())
			.set_price(10000.into())
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch id
			U256::from(1_u8),
			// size
			5.into(),
			// market
			market_id,
			// price
			10000.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			1699940278000,
		));

		// Decrease the price of the asset
		let mut index_prices: Vec<MultiplePrices> = Vec::new();
		let index_price1 =
			MultiplePrices { market_id, index_price: 9500.into(), mark_price: 9500.into() };
		index_prices.push(index_price1);
		assert_ok!(Prices::update_prices(RuntimeOrigin::signed(1), index_prices, 1699940278000));

		// Place Forced order for liquidation
		let charlie_order = Order::new(U256::from(204), charlie_id)
			.set_size(5.into())
			.set_price(9500.into())
			.sign_order(get_private_key(charlie().pub_key));

		let alice_forced_order = Order::new(U256::from(203), alice_id)
			.set_size(5.into())
			.set_price(9500.into())
			.set_order_type(OrderType::Forced)
			.set_direction(Direction::Long)
			.set_side(Side::Sell)
			.sign_order_liquidator(get_private_key(eduard().pub_key), eduard().pub_key);

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch id
			U256::from(2_u8),
			// size
			5.into(),
			// market
			market_id,
			// price
			9500.into(),
			// orders
			vec![charlie_order, alice_forced_order],
			// batch_timestamp
			1699940278000,
		));
	});
}

#[test]
#[should_panic(expected = "TradeBatchError542")]
fn test_invalid_liquidator() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());
		let charlie_id: U256 = get_trading_account_id(charlie());

		// market id
		let market_id = btc_usdc().market.id;

		// Create orders
		let alice_order = Order::new(U256::from(201), alice_id)
			.set_size(5.into())
			.set_leverage(5.into())
			.set_price(10000.into())
			.sign_order(get_private_key(alice().pub_key));

		let bob_order = Order::new(U256::from(202), bob_id)
			.set_size(5.into())
			.set_order_type(OrderType::Market)
			.set_direction(Direction::Short)
			.set_leverage(5.into())
			.set_price(10000.into())
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch id
			U256::from(1_u8),
			// size
			5.into(),
			// market
			market_id,
			// price
			10000.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			1699940278000,
		));

		// Decrease the price of the asset
		let mut index_prices: Vec<MultiplePrices> = Vec::new();
		let index_price1 =
			MultiplePrices { market_id, index_price: 8500.into(), mark_price: 8500.into() };
		index_prices.push(index_price1);
		assert_ok!(Prices::update_prices(RuntimeOrigin::signed(1), index_prices, 1699940278000));

		// Place Forced order for liquidation
		let charlie_order = Order::new(U256::from(204), charlie_id)
			.set_size(5.into())
			.set_price(8500.into())
			.sign_order(get_private_key(charlie().pub_key));

		let alice_forced_order = Order::new(U256::from(203), alice_id)
			.set_size(5.into())
			.set_price(8500.into())
			.set_order_type(OrderType::Forced)
			.set_direction(Direction::Long)
			.set_side(Side::Sell)
			.sign_order_liquidator(get_private_key(dave().pub_key), dave().pub_key);

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch id
			U256::from(2_u8),
			// size
			5.into(),
			// market
			market_id,
			// price
			8500.into(),
			// orders
			vec![charlie_order, alice_forced_order],
			// batch_timestamp
			1699940278000,
		));
	});
}
