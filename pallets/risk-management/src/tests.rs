use crate::mock::*;
use frame_support::assert_ok;
use pallet_support::{
	test_helpers::{
		accounts_helper::{
			alice, bob, charlie, dave, eduard, get_private_key, get_trading_account_id,
		},
		asset_helper::{btc, eth, link, usdc},
		market_helper::{btc_usdc, eth_usdc, link_usdc},
		setup_fee,
	},
	types::{Direction, MultiplePrices, Order, OrderSide, OrderType, Position, Side},
};
use primitive_types::U256;

fn assert_has_events(expected_events: Vec<RuntimeEvent>) {
	for expected_event in &expected_events {
		if !System::events().iter().any(|event| event.event == *expected_event) {
			panic!("Expected event not found: {:?}", expected_event);
		}
	}
}

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
			vec![btc_usdc(), link_usdc(), eth_usdc()]
		));

		// Add accounts to the system
		assert_ok!(TradingAccounts::add_accounts(
			RuntimeOrigin::signed(1),
			vec![alice(), bob(), charlie(), dave()]
		));

		// Set matching_time_limit
		assert_ok!(Trading::set_matching_time_limit(
			RuntimeOrigin::root(),
			2419200 //4 weeks
		));

		// Add liquidator
		Trading::add_liquidator_signer(RuntimeOrigin::root(), eduard().pub_key)
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
		let alice_order = Order::new(201.into(), alice_id)
			.set_size(5.into())
			.set_leverage(5.into())
			.set_price(10000.into())
			.sign_order(get_private_key(alice().pub_key));

		let bob_order = Order::new(202.into(), bob_id)
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
		let charlie_order = Order::new(204.into(), charlie_id)
			.set_size(5.into())
			.set_price(5000.into())
			.set_leverage(5.into())
			.sign_order(get_private_key(charlie().pub_key));

		let alice_forced_order = Order::new(203.into(), alice_id)
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
fn test_liquidation_w_fees() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());
		let charlie_id: U256 = get_trading_account_id(charlie());

		// market id
		let market_id = btc_usdc().market.id;
		let collateral_id = usdc().asset.id;

		let (fee_details_maker, fee_details_taker) = setup_fee();
		// Dispatch a signed extrinsic.
		assert_ok!(TradingFees::update_base_fees(
			RuntimeOrigin::signed(1),
			collateral_id,
			Side::Sell,
			OrderSide::Maker,
			fee_details_maker.clone(),
		));
		assert_ok!(TradingFees::update_base_fees(
			RuntimeOrigin::signed(1),
			collateral_id,
			Side::Sell,
			OrderSide::Taker,
			fee_details_taker.clone(),
		));

		// Create orders
		let alice_order = Order::new(201.into(), alice_id)
			.set_size(5.into())
			.set_leverage(5.into())
			.set_price(10000.into())
			.sign_order(get_private_key(alice().pub_key));

		let bob_order = Order::new(202.into(), bob_id)
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
		let charlie_order = Order::new(204.into(), charlie_id)
			.set_size(5.into())
			.set_price(5000.into())
			.set_leverage(5.into())
			.sign_order(get_private_key(charlie().pub_key));

		let alice_forced_order = Order::new(203.into(), alice_id)
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
			vec![charlie_order, alice_forced_order.clone()],
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

		// Check for events
		assert_has_events(vec![pallet_trading::Event::OrderExecuted {
			account_id: alice_id,
			order_id: alice_forced_order.order_id,
			market_id,
			size: 5.into(),
			direction: alice_forced_order.direction.into(),
			side: alice_forced_order.side.into(),
			order_type: alice_forced_order.order_type.into(),
			execution_price: 5000.into(),
			pnl: (-25000).into(),
			fee: 0.into(),
			is_final: true,
			is_maker: false,
		}
		.into()]);

		let flag = Trading::force_closure_flag(alice_id, btc_usdc().market.asset_collateral);
		assert_eq!(flag.is_none(), true);
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
		let alice_order = Order::new(201.into(), alice_id)
			.set_size(5.into())
			.set_leverage(5.into())
			.set_price(10000.into())
			.sign_order(get_private_key(alice().pub_key));

		let bob_order = Order::new(202.into(), bob_id)
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
		let charlie_order = Order::new(204.into(), charlie_id)
			.set_size(5.into())
			.set_price(9500.into())
			.sign_order(get_private_key(charlie().pub_key));

		let alice_forced_order = Order::new(203.into(), alice_id)
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
		let alice_order = Order::new(201.into(), alice_id)
			.set_size(5.into())
			.set_leverage(5.into())
			.set_price(10000.into())
			.sign_order(get_private_key(alice().pub_key));

		let bob_order = Order::new(202.into(), bob_id)
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
		let charlie_order = Order::new(204.into(), charlie_id)
			.set_size(5.into())
			.set_price(8500.into())
			.sign_order(get_private_key(charlie().pub_key));

		let alice_forced_order = Order::new(203.into(), alice_id)
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

#[test]
// When user has 2 positions and liquidation is triggered and complete liquidations
fn test_liquidation_multiple_positions() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());
		let charlie_id: U256 = get_trading_account_id(charlie());

		// Open BTCUSDC position
		let market_id = btc_usdc().market.id;

		// Create orders
		let alice_order = Order::new(201.into(), alice_id)
			.set_size(9.into())
			.set_leverage(8.into())
			.set_price(8500.into())
			.sign_order(get_private_key(alice().pub_key));

		let bob_order = Order::new(202.into(), bob_id)
			.set_size(9.into())
			.set_order_type(OrderType::Market)
			.set_direction(Direction::Short)
			.set_leverage(8.into())
			.set_price(8500.into())
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch id
			U256::from(1_u8),
			// size
			9.into(),
			// market
			market_id,
			// price
			8500.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			1699940278000,
		));

		// Open ETHUSDC position
		let market_id = eth_usdc().market.id;

		// Create orders
		let alice_order = Order::new(205.into(), alice_id)
			.set_size(32.into())
			.set_leverage(8.into())
			.set_market_id(market_id)
			.sign_order(get_private_key(alice().pub_key));

		let bob_order = Order::new(206.into(), bob_id)
			.set_size(32.into())
			.set_order_type(OrderType::Market)
			.set_direction(Direction::Short)
			.set_leverage(8.into())
			.set_market_id(market_id)
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch id
			U256::from(3_u8),
			// size
			32.into(),
			// market
			market_id,
			// price
			100.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			1699940278000,
		));

		Timestamp::set_timestamp(1699949278000);

		// Decrease the price of BTCUSDC
		let mut index_prices: Vec<MultiplePrices> = Vec::new();
		let index_price1 = MultiplePrices {
			market_id: btc_usdc().market.id,
			index_price: 8000.into(),
			mark_price: 8000.into(),
		};
		index_prices.push(index_price1);
		assert_ok!(Prices::update_prices(RuntimeOrigin::signed(1), index_prices, 1699949278000));

		// Decrease the price of ETHUSDC
		let mut index_prices: Vec<MultiplePrices> = Vec::new();
		let index_price1 =
			MultiplePrices { market_id, index_price: 95.into(), mark_price: 95.into() };
		index_prices.push(index_price1);
		assert_ok!(Prices::update_prices(RuntimeOrigin::signed(1), index_prices, 1699949278000));

		// Liquidation order for btc
		let market_id = btc_usdc().market.id;

		let charlie_order = Order::new(204.into(), charlie_id)
			.set_size(9.into())
			.set_price(8000.into())
			.set_leverage(8.into())
			.set_timestamp(1699949278000)
			.sign_order(get_private_key(charlie().pub_key));

		let alice_forced_order = Order::new(203.into(), alice_id)
			.set_size(9.into())
			.set_price(8000.into())
			.set_order_type(OrderType::Forced)
			.set_direction(Direction::Long)
			.set_side(Side::Sell)
			.set_timestamp(1699949278000)
			.sign_order_liquidator(get_private_key(eduard().pub_key), eduard().pub_key);

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch id
			U256::from(2_u8),
			// size
			9.into(),
			// market
			market_id,
			// price
			8000.into(),
			// orders
			vec![charlie_order, alice_forced_order],
			// batch_timestamp
			1699949278000,
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
		assert_eq!(flag.is_some(), true);

		// Liquidation order for eth
		let market_id = eth_usdc().market.id;

		let charlie_order = Order::new(208.into(), charlie_id)
			.set_size(32.into())
			.set_price(95.into())
			.set_leverage(8.into())
			.set_timestamp(1699949278000)
			.set_market_id(market_id)
			.sign_order(get_private_key(charlie().pub_key));

		let alice_forced_order = Order::new(207.into(), alice_id)
			.set_size(32.into())
			.set_price(95.into())
			.set_order_type(OrderType::Forced)
			.set_direction(Direction::Long)
			.set_side(Side::Sell)
			.set_timestamp(1699949278000)
			.set_market_id(market_id)
			.sign_order_liquidator(get_private_key(eduard().pub_key), eduard().pub_key);

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch id
			U256::from(4_u8),
			// size
			32.into(),
			// market
			market_id,
			// price
			95.into(),
			// orders
			vec![charlie_order, alice_forced_order],
			// batch_timestamp
			1699949278000,
		));

		let flag = Trading::force_closure_flag(alice_id, btc_usdc().market.asset_collateral);
		assert_eq!(flag.is_none(), true);
	});
}
