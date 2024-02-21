use crate::{mock::*, Event};
use frame_support::assert_ok;
use pallet_support::{
	test_helpers::{
		accounts_helper::{alice, bob, charlie, dave, get_private_key, get_trading_account_id},
		asset_helper::{btc, eth, link, usdc},
		market_helper::{btc_usdc, eth_usdc, link_usdc},
		setup_fee,
	},
	types::{Direction, Order, OrderSide, OrderType, Position, Side},
};
use primitive_types::U256;
use sp_arithmetic::{
	traits::{One, Zero},
	FixedI128,
};

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
	});

	env
}

#[test]
fn add_liquidator_signer_authorized() {
	// Get a test environment
	let mut env = setup();

	env.execute_with(|| {
		let liquidator_signer = U256::from(12345);
		// Add a signer
		Trading::add_liquidator_signer(RuntimeOrigin::root(), liquidator_signer)
			.expect("error while adding signer");
		assert_eq!(Trading::liquidator_signers(), vec![liquidator_signer]);
		assert_eq!(Trading::is_liquidator_signer_valid(liquidator_signer), true);
	});
}

#[test]
fn add_multiple_liquidator_signer_authorized() {
	// Get a test environment
	let mut env = setup();

	env.execute_with(|| {
		let liquidator_signer_1 = U256::from(12345);
		let liquidator_signer_2 = U256::from(12346);
		// Add a signer
		Trading::add_liquidator_signer(RuntimeOrigin::root(), liquidator_signer_1)
			.expect("error while adding signer");
		Trading::add_liquidator_signer(RuntimeOrigin::root(), liquidator_signer_2)
			.expect("error while adding signer");

		assert_eq!(Trading::liquidator_signers(), vec![liquidator_signer_1, liquidator_signer_2]);
		assert_eq!(Trading::is_liquidator_signer_valid(liquidator_signer_1), true);
		assert_eq!(Trading::is_liquidator_signer_valid(liquidator_signer_2), true);
	});
}

#[test]
#[should_panic(expected = "ZeroSigner")]
fn add_signer_authorized_0_pub_key() {
	// Get a test environment
	let mut env = setup();

	env.execute_with(|| {
		// Add signer
		Trading::add_liquidator_signer(RuntimeOrigin::root(), U256::from(0))
			.expect("Error in code");
	});
}

#[test]
#[should_panic(expected = "DuplicateSigner")]
fn add_signer_authorized_duplicate_pub_key() {
	// Get a test environment
	let mut env = setup();

	env.execute_with(|| {
		let liquidator_signer = U256::from(12345);
		// Add a signer
		Trading::add_liquidator_signer(RuntimeOrigin::root(), liquidator_signer)
			.expect("error while adding signer");
		Trading::add_liquidator_signer(RuntimeOrigin::root(), liquidator_signer)
			.expect("error while adding signer");
	});
}

#[test]
#[should_panic(expected = "SignerNotWhitelisted")]
fn remove_signer_authorized_invalid_signer() {
	// Get a test environment
	let mut env = setup();

	env.execute_with(|| {
		let liquidator_signer = U256::from(12345);
		// Remove signer; error
		Trading::remove_liquidator_signer(RuntimeOrigin::root(), liquidator_signer)
			.expect("Error in code");
	});
}

#[test]
fn remove_signer_authorized() {
	// Get a test environment
	let mut env = setup();

	env.execute_with(|| {
		let liquidator_signer = U256::from(12345);
		// Add a signer
		Trading::add_liquidator_signer(RuntimeOrigin::root(), liquidator_signer)
			.expect("error while adding signer");

		// Remove signer
		Trading::remove_liquidator_signer(RuntimeOrigin::root(), liquidator_signer)
			.expect("Error in code");

		assert_eq!(Trading::liquidator_signers(), vec![]);
		assert_eq!(Trading::is_liquidator_signer_valid(liquidator_signer), false);
	});
}

#[test]
// basic open trade without any leverage
fn it_works_for_open_trade_simple() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());

		// market id
		let market_id = btc_usdc().market.id;

		// Create orders
		let alice_order =
			Order::new(U256::from(201), alice_id).sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(U256::from(202), bob_id)
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

		// Check the execution of orders

		// Positions
		let alice_position = Trading::positions(alice_id, (market_id, alice_order.direction));
		let expected_position: Position = Position {
			market_id,
			avg_execution_price: 100.into(),
			size: 1.into(),
			direction: Direction::Long,
			margin_amount: 100.into(),
			borrowed_amount: 0.into(),
			leverage: 1.into(),
			created_timestamp: 1699940367,
			modified_timestamp: 1699940367,
			realized_pnl: 0.into(),
		};
		assert_eq!(expected_position, alice_position);

		let bob_position = Trading::positions(bob_id, (market_id, bob_order.direction));
		let expected_position: Position = Position {
			market_id,
			avg_execution_price: 100.into(),
			size: 1.into(),
			direction: Direction::Short,
			margin_amount: 100.into(),
			borrowed_amount: 0.into(),
			leverage: 1.into(),
			created_timestamp: 1699940367,
			modified_timestamp: 1699940367,
			realized_pnl: 0.into(),
		};
		assert_eq!(expected_position, bob_position);

		// Check for open interest
		let open_interest = Trading::open_interest(market_id);
		assert_eq!(open_interest, FixedI128::from_inner(2000000000000000000));

		// Check for events
		assert_has_events(vec![
			Event::TradeExecuted {
				batch_id: U256::from(1_u8),
				market_id,
				size: 1.into(),
				execution_price: 100.into(),
				direction: bob_order.direction.into(),
				side: bob_order.side.into(),
			}
			.into(),
			Event::OrderExecuted {
				account_id: alice_id,
				order_id: alice_order.order_id,
				market_id,
				size: 1.into(),
				direction: alice_order.direction.into(),
				side: alice_order.side.into(),
				order_type: alice_order.order_type.into(),
				execution_price: 100.into(),
				pnl: 0.into(),
				fee: 0.into(),
				is_final: true,
				is_maker: true,
			}
			.into(),
			Event::OrderExecuted {
				account_id: bob_id,
				order_id: bob_order.order_id,
				market_id,
				size: 1.into(),
				direction: bob_order.direction.into(),
				side: bob_order.side.into(),
				order_type: bob_order.order_type.into(),
				execution_price: 100.into(),
				pnl: 0.into(),
				fee: 0.into(),
				is_final: true,
				is_maker: false,
			}
			.into(),
		]);
	});
}

#[test]
// should emit an error when trying to open more than the maximum limit of position size
fn it_reverts_for_more_than_max_size() {
	let mut env = setup();

	env.execute_with(|| {
		let modified_btc_usdc = btc_usdc().set_maximum_position_size(2.into());
		assert_ok!(Markets::replace_all_markets(
			RuntimeOrigin::signed(1),
			vec![modified_btc_usdc, link_usdc()]
		));

		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());

		// market id
		let market_id = btc_usdc().market.id;

		// Create orders
		let alice_order = Order::new(U256::from(201), alice_id)
			.set_size(3.into())
			.sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(U256::from(202), bob_id)
			.set_size(3.into())
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch_id
			U256::from(1_u8),
			// quantity_locked
			3.into(),
			// market_id
			market_id,
			// oracle_price
			100.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			1699940367000,
		));

		assert_has_events(vec![
			Event::OrderError { order_id: U256::from(201), error_code: 548 }.into()
		]);
	});
}

#[test]
// basic open trade with leverage
fn it_works_for_open_trade_with_leverage() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());

		// market id
		let market_id = btc_usdc().market.id;

		// Create orders
		let alice_order = Order::new(U256::from(201), alice_id)
			.set_leverage(5.into())
			.sign_order(get_private_key(alice().pub_key));

		let bob_order = Order::new(U256::from(202), bob_id)
			.set_order_type(OrderType::Market)
			.set_direction(Direction::Short)
			.set_leverage(5.into())
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch id
			U256::from(1_u8),
			// size
			1.into(),
			// market
			market_id,
			// price
			100.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			1699940367000,
		));

		// Check the execution of orders
		let alice_position = Trading::positions(alice_id, (market_id, alice_order.direction));
		let expected_position: Position = Position {
			market_id,
			avg_execution_price: 100.into(),
			size: 1.into(),
			direction: Direction::Long,
			margin_amount: 20.into(),
			borrowed_amount: 80.into(),
			leverage: 5.into(),
			created_timestamp: 1699940367,
			modified_timestamp: 1699940367,
			realized_pnl: 0.into(),
		};
		assert_eq!(expected_position, alice_position);

		let bob_position = Trading::positions(bob_id, (market_id, bob_order.direction));
		let expected_position: Position = Position {
			market_id,
			avg_execution_price: 100.into(),
			size: 1.into(),
			direction: Direction::Short,
			margin_amount: 20.into(),
			borrowed_amount: 80.into(),
			leverage: 5.into(),
			created_timestamp: 1699940367,
			modified_timestamp: 1699940367,
			realized_pnl: 0.into(),
		};
		assert_eq!(expected_position, bob_position);

		// Check for events
		assert_has_events(vec![
			Event::TradeExecuted {
				batch_id: U256::from(1_u8),
				market_id,
				size: 1.into(),
				execution_price: 100.into(),
				direction: bob_order.direction.into(),
				side: bob_order.side.into(),
			}
			.into(),
			Event::OrderExecuted {
				account_id: alice_id,
				order_id: alice_order.order_id,
				market_id,
				size: 1.into(),
				direction: alice_order.direction.into(),
				side: alice_order.side.into(),
				order_type: alice_order.order_type.into(),
				execution_price: 100.into(),
				pnl: 0.into(),
				fee: 0.into(),
				is_final: true,
				is_maker: true,
			}
			.into(),
			Event::OrderExecuted {
				account_id: bob_id,
				order_id: bob_order.order_id,
				market_id,
				size: 1.into(),
				direction: bob_order.direction.into(),
				side: bob_order.side.into(),
				order_type: bob_order.order_type.into(),
				execution_price: 100.into(),
				pnl: 0.into(),
				fee: 0.into(),
				is_final: true,
				is_maker: false,
			}
			.into(),
		]);
	});
}

#[test]
// basic open and close trade without any leverage
fn it_works_for_close_trade_simple() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());

		// market id
		let market_id = btc_usdc().market.id;
		let collateral_id = usdc().asset.id;

		// Create open orders
		let alice_open_order =
			Order::new(U256::from(201), alice_id).sign_order(get_private_key(alice().pub_key));
		let bob_open_order = Order::new(U256::from(202), bob_id)
			.set_order_type(OrderType::Market)
			.set_direction(Direction::Short)
			.sign_order(get_private_key(bob().pub_key));

		// Execute the trade
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
			vec![alice_open_order.clone(), bob_open_order.clone()],
			// batch_timestamp
			1699940367000,
		));

		// Close close orders
		let alice_close_order = Order::new(U256::from(203), alice_id)
			.set_side(Side::Sell)
			.set_price(105.into())
			.sign_order(get_private_key(alice().pub_key));

		let bob_close_order = Order::new(U256::from(204), bob_id)
			.set_side(Side::Sell)
			.set_price(100.into())
			.set_order_type(OrderType::Market)
			.set_direction(Direction::Short)
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch_id
			U256::from(2_u8),
			// quantity_locked
			1.into(),
			// market_id
			market_id,
			// oracle_price
			105.into(),
			// orders
			vec![alice_close_order.clone(), bob_close_order.clone()],
			// batch_timestamp
			1699940367000,
		));

		// Check for open interest
		let open_interest = Trading::open_interest(market_id);
		assert_eq!(open_interest, FixedI128::zero());

		// Check for balances
		assert_eq!(TradingAccounts::balances(alice_id, collateral_id), 10005.into());
		assert_eq!(TradingAccounts::balances(bob_id, collateral_id), 9995.into());

		// Check for locked margin
		assert_eq!(TradingAccounts::locked_margin(alice_id, collateral_id), 0.into());
		assert_eq!(TradingAccounts::locked_margin(bob_id, collateral_id), 0.into());

		// Check for events
		assert_has_events(vec![
			Event::TradeExecuted {
				batch_id: U256::from(1_u8),
				market_id,
				size: 1.into(),
				execution_price: 100.into(),
				direction: bob_open_order.direction.into(),
				side: bob_open_order.side.into(),
			}
			.into(),
			Event::TradeExecuted {
				batch_id: U256::from(2_u8),
				market_id,
				size: 1.into(),
				execution_price: 105.into(),
				direction: bob_close_order.direction.into(),
				side: bob_close_order.side.into(),
			}
			.into(),
			Event::OrderExecuted {
				account_id: alice_id,
				order_id: alice_close_order.order_id,
				market_id,
				size: 1.into(),
				direction: alice_close_order.direction.into(),
				side: alice_close_order.side.into(),
				order_type: alice_close_order.order_type.into(),
				execution_price: 105.into(),
				pnl: 5.into(),
				fee: 0.into(),
				is_final: true,
				is_maker: true,
			}
			.into(),
			Event::OrderExecuted {
				account_id: bob_id,
				order_id: bob_close_order.order_id,
				market_id,
				size: 1.into(),
				direction: bob_close_order.direction.into(),
				side: bob_close_order.side.into(),
				order_type: bob_close_order.order_type.into(),
				execution_price: 105.into(),
				pnl: (-5).into(),
				fee: 0.into(),
				is_final: true,
				is_maker: false,
			}
			.into(),
		]);
	});
}

#[test]
// partially open position by executing in different batches
fn it_works_for_open_trade_partial_open() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());

		// market id
		let market_id = btc_usdc().market.id;

		let alice_open_order_1 =
			Order::new(U256::from(201), alice_id).sign_order(get_private_key(alice().pub_key));

		let bob_open_order_1 = Order::new(U256::from(202), bob_id)
			.set_order_type(OrderType::Market)
			.set_direction(Direction::Short)
			.set_size(2.into())
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch id
			U256::from(1_u8),
			// size
			1.into(),
			// market
			market_id,
			// price
			100.into(),
			// order
			vec![alice_open_order_1.clone(), bob_open_order_1.clone()],
			// batch_timestamp
			1699940367000,
		));

		let alice_open_order_2 = Order::new(U256::from(203), alice_id)
			.set_price(98.into())
			.sign_order(get_private_key(alice().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch id
			U256::from(2_u8),
			// size
			1.into(),
			// market id
			market_id,
			// price
			98.into(),
			// order
			vec![alice_open_order_2.clone(), bob_open_order_1.clone()],
			// batch_timestamp
			1699940367000,
		));

		let position1 = Trading::positions(bob_id, (market_id, Direction::Short));
		let expected_position: Position = Position {
			market_id,
			avg_execution_price: 99.into(),
			size: 2.into(),
			direction: Direction::Short,
			margin_amount: 198.into(),
			borrowed_amount: 0.into(),
			leverage: 1.into(),
			created_timestamp: 1699940367,
			modified_timestamp: 1699940367,
			realized_pnl: 0.into(),
		};
		assert_eq!(expected_position, position1);

		// Check for events
		assert_has_events(vec![
			Event::TradeExecuted {
				batch_id: U256::from(1_u8),
				market_id,
				size: 1.into(),
				execution_price: 100.into(),
				direction: bob_open_order_1.direction.into(),
				side: bob_open_order_1.side.into(),
			}
			.into(),
			Event::TradeExecuted {
				batch_id: U256::from(2_u8),
				market_id,
				size: 1.into(),
				execution_price: 98.into(),
				direction: bob_open_order_1.direction.into(),
				side: bob_open_order_1.side.into(),
			}
			.into(),
			Event::OrderExecuted {
				account_id: alice_id,
				order_id: alice_open_order_1.order_id,
				market_id,
				size: 1.into(),
				direction: alice_open_order_1.direction.into(),
				side: alice_open_order_1.side.into(),
				order_type: alice_open_order_1.order_type.into(),
				execution_price: 100.into(),
				pnl: 0.into(),
				fee: 0.into(),
				is_final: true,
				is_maker: true,
			}
			.into(),
			Event::OrderExecuted {
				account_id: bob_id,
				order_id: bob_open_order_1.order_id,
				market_id,
				size: 1.into(),
				direction: bob_open_order_1.direction.into(),
				side: bob_open_order_1.side.into(),
				order_type: bob_open_order_1.order_type.into(),
				execution_price: 100.into(),
				pnl: 0.into(),
				fee: 0.into(),
				is_final: false,
				is_maker: false,
			}
			.into(),
			Event::OrderExecuted {
				account_id: alice_id,
				order_id: alice_open_order_2.order_id,
				market_id,
				size: 1.into(),
				direction: alice_open_order_2.direction.into(),
				side: alice_open_order_2.side.into(),
				order_type: alice_open_order_2.order_type.into(),
				execution_price: 98.into(),
				pnl: 0.into(),
				fee: 0.into(),
				is_final: true,
				is_maker: true,
			}
			.into(),
			Event::OrderExecuted {
				account_id: bob_id,
				order_id: bob_open_order_1.order_id,
				market_id,
				size: 1.into(),
				direction: bob_open_order_1.direction.into(),
				side: bob_open_order_1.side.into(),
				order_type: bob_open_order_1.order_type.into(),
				execution_price: 98.into(),
				pnl: 0.into(),
				fee: 0.into(),
				is_final: true,
				is_maker: false,
			}
			.into(),
		]);
	});
}

#[test]
// partially close position by executing in different batches
fn it_works_for_close_trade_partial_close() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());

		// market id
		let market_id = btc_usdc().market.id;

		let alice_open_order_1 = Order::new(U256::from(201), alice_id)
			.set_size(2.into())
			.sign_order(get_private_key(alice().pub_key));

		let bob_open_order_1 = Order::new(U256::from(202), bob_id)
			.set_order_type(OrderType::Market)
			.set_direction(Direction::Short)
			.set_size(2.into())
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch_id
			U256::from(1_u8),
			// size
			2.into(),
			// market_id
			market_id,
			// price
			100.into(),
			// order
			vec![alice_open_order_1.clone(), bob_open_order_1.clone()],
			// batch_timestamp
			1699940367000,
		));

		let alice_close_order_1 = Order::new(U256::from(203), alice_id)
			.set_price(104.into())
			.set_side(Side::Sell)
			.sign_order(get_private_key(alice().pub_key));

		let bob_close_order_1 = Order::new(U256::from(204), bob_id)
			.set_order_type(OrderType::Market)
			.set_direction(Direction::Short)
			.set_side(Side::Sell)
			.set_size(2.into())
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch id
			U256::from(2_u8),
			// size
			1.into(),
			// market_id
			market_id,
			// price
			105.into(),
			// orders
			vec![alice_close_order_1.clone(), bob_close_order_1.clone()],
			// batch_timestamp
			1699940367000,
		));

		// Check for open interest
		let open_interest = Trading::open_interest(market_id);
		assert_eq!(open_interest, FixedI128::from_inner(2000000000000000000));

		let alice_close_order_2 = Order::new(U256::from(205), alice_id)
			.set_price(98.into())
			.set_side(Side::Sell)
			.sign_order(get_private_key(alice().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch_id
			U256::from(3_u8),
			// size
			1.into(),
			// market_id
			market_id,
			// price
			100.into(),
			// order
			vec![alice_close_order_2.clone(), bob_close_order_1.clone()],
			// batch_timestamp
			1699940367000,
		));

		// Check for events
		assert_has_events(vec![
			Event::TradeExecuted {
				batch_id: U256::from(1_u8),
				market_id,
				size: 2.into(),
				execution_price: 100.into(),
				direction: bob_open_order_1.direction.into(),
				side: bob_open_order_1.side.into(),
			}
			.into(),
			Event::TradeExecuted {
				batch_id: U256::from(2_u8),
				market_id,
				size: 1.into(),
				execution_price: 104.into(),
				direction: bob_close_order_1.direction.into(),
				side: bob_close_order_1.side.into(),
			}
			.into(),
			Event::TradeExecuted {
				batch_id: U256::from(3_u8),
				market_id,
				size: 1.into(),
				execution_price: 98.into(),
				direction: bob_close_order_1.direction.into(),
				side: bob_close_order_1.side.into(),
			}
			.into(),
			Event::OrderExecuted {
				account_id: bob_id,
				order_id: bob_close_order_1.order_id,
				market_id,
				size: 1.into(),
				direction: bob_close_order_1.direction.into(),
				side: bob_close_order_1.side.into(),
				order_type: bob_close_order_1.order_type.into(),
				execution_price: 104.into(),
				pnl: (-4).into(),
				fee: 0.into(),
				is_final: false,
				is_maker: false,
			}
			.into(),
			Event::OrderExecuted {
				account_id: bob_id,
				order_id: bob_close_order_1.order_id,
				market_id,
				size: 1.into(),
				direction: bob_close_order_1.direction.into(),
				side: bob_close_order_1.side.into(),
				order_type: bob_close_order_1.order_type.into(),
				execution_price: 98.into(),
				pnl: 2.into(),
				fee: 0.into(),
				is_final: true,
				is_maker: false,
			}
			.into(),
		]);
	});
}

#[test]
// trade batch with multiple makers and a taker
fn it_works_for_open_trade_multiple_makers() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());
		let charlie_id: U256 = get_trading_account_id(charlie());
		let dave_id: U256 = get_trading_account_id(dave());

		// market id
		let market_id = btc_usdc().market.id;

		let alice_open_order_1 = Order::new(U256::from(201), alice_id)
			.set_price(105.into())
			.set_direction(Direction::Short)
			.sign_order(get_private_key(alice().pub_key));

		let bob_open_order_1 = Order::new(U256::from(202), bob_id)
			.set_price(99.into())
			.set_direction(Direction::Short)
			.sign_order(get_private_key(bob().pub_key));

		let charlie_open_order_1 = Order::new(U256::from(203), charlie_id)
			.set_price(102.into())
			.set_size(2.into())
			.set_direction(Direction::Short)
			.sign_order(get_private_key(charlie().pub_key));

		let dave_open_order_1 = Order::new(U256::from(204), dave_id)
			.set_price(100.into())
			.set_size(3.into())
			.set_order_type(OrderType::Market)
			.sign_order(get_private_key(dave().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch_id
			U256::from(1_u8),
			// size
			3.into(),
			// market_id
			market_id,
			// price
			100.into(),
			// orders
			vec![
				alice_open_order_1.clone(),
				bob_open_order_1.clone(),
				charlie_open_order_1.clone(),
				dave_open_order_1.clone()
			],
			// batch_timestamp
			1699940367000,
		));

		// Check for events
		assert_has_events(vec![Event::TradeExecuted {
			batch_id: U256::from(1_u8),
			market_id,
			size: 3.into(),
			execution_price: 102.into(),
			direction: dave_open_order_1.direction.into(),
			side: dave_open_order_1.side.into(),
		}
		.into()]);
	});
}

#[test]
#[should_panic(expected = "TradeBatchError525")]
// trade batch with previously executed batch_id
fn it_reverts_for_trade_with_same_batch_id() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());

		// market id
		let market_id = btc_usdc().market.id;

		// Create orders
		let alice_open_order_1 = Order::new(U256::from(201), alice_id)
			.set_leverage(5.into())
			.sign_order(get_private_key(alice().pub_key));

		let bob_open_order_1 = Order::new(U256::from(202), bob_id)
			.set_order_type(OrderType::Market)
			.set_direction(Direction::Short)
			.set_leverage(5.into())
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch id
			U256::from(1_u8),
			// size
			1.into(),
			// market
			market_id,
			// price
			100.into(),
			// orders
			vec![alice_open_order_1.clone(), bob_open_order_1.clone()],
			// batch_timestamp
			1699940367000,
		));

		// Create orders
		let alice_open_order_2 = Order::new(U256::from(203), alice_id)
			.set_leverage(5.into())
			.sign_order(get_private_key(alice().pub_key));

		let bob_open_order_2 = Order::new(U256::from(204), bob_id)
			.set_order_type(OrderType::Market)
			.set_direction(Direction::Short)
			.set_leverage(5.into())
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch id
			U256::from(1_u8),
			// size
			1.into(),
			// market
			market_id,
			// price
			100.into(),
			// orders
			vec![alice_open_order_2.clone(), bob_open_order_2.clone()],
			// batch_timestamp
			1699940367000,
		));
	});
}

#[test]
#[should_panic(expected = "TradeBatchError509")]
// trade batch with invalid market_id
fn it_reverts_for_trade_with_invalid_market() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());

		// Create orders
		let alice_open_order_1 = Order::new(U256::from(201), alice_id)
			.set_leverage(5.into())
			.sign_order(get_private_key(alice().pub_key));

		let bob_open_order_1 = Order::new(U256::from(202), bob_id)
			.set_order_type(OrderType::Market)
			.set_direction(Direction::Short)
			.set_leverage(5.into())
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch id
			U256::from(1_u8),
			// size
			1.into(),
			// market
			4_u128,
			// price
			100.into(),
			// orders
			vec![alice_open_order_1.clone(), bob_open_order_1.clone()],
			// batch_timestamp
			1699940367000,
		));
	});
}

#[test]
#[should_panic(expected = "TradeBatchError547")]
// trade batch with insuffient orders
fn it_reverts_for_trade_with_insufficient_orders_1_order() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());

		// market id
		let market_id = btc_usdc().market.id;

		// Create orders
		let alice_open_order_1 = Order::new(U256::from(201), alice_id)
			.set_leverage(5.into())
			.sign_order(get_private_key(alice().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch id
			U256::from(1_u8),
			// size
			1.into(),
			// market
			market_id,
			// price
			100.into(),
			// orders
			vec![alice_open_order_1.clone()],
			// batch_timestamp
			1699940367000,
		));
	});
}

#[test]
#[should_panic(expected = "TradeBatchError547")]
// trade batch with insuffient orders
fn it_reverts_for_trade_with_insufficient_orders_0_orders() {
	let mut env = setup();

	env.execute_with(|| {
		// market id
		let market_id = btc_usdc().market.id;

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch id
			U256::from(1_u8),
			// size
			1.into(),
			// market
			market_id,
			// price
			100.into(),
			// orders
			vec![],
			// batch_timestamp
			1699940367000,
		));
	});
}

#[test]
#[should_panic(expected = "TradeBatchError522")]
// trade batch with quantity_locked as 0
fn it_reverts_for_trade_with_quantity_locked_zero() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());

		// market id
		let market_id = btc_usdc().market.id;

		// Create orders
		let alice_open_order_1 = Order::new(U256::from(201), alice_id)
			.set_leverage(5.into())
			.sign_order(get_private_key(alice().pub_key));

		let bob_open_order_1 = Order::new(U256::from(202), bob_id)
			.set_order_type(OrderType::Market)
			.set_direction(Direction::Short)
			.set_leverage(5.into())
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch id
			U256::from(1_u8),
			// size
			0.into(),
			// market
			market_id,
			// price
			100.into(),
			// orders
			vec![alice_open_order_1.clone(), bob_open_order_1.clone()],
			// batch_timestamp
			1699940367000,
		));
	});
}

#[test]
#[should_panic(expected = "TradeBatchError503")]
// trade batch with quantity_locked is not multiple of step size
fn it_reverts_for_trade_with_quantity_locked_is_not_multiple_of_step_size() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());

		// market id
		let market_id = btc_usdc().market.id;

		// Create orders
		let alice_open_order_1 = Order::new(U256::from(201), alice_id)
			.set_leverage(5.into())
			.set_size(2.into())
			.sign_order(get_private_key(alice().pub_key));

		let bob_open_order_1 = Order::new(U256::from(202), bob_id)
			.set_order_type(OrderType::Market)
			.set_direction(Direction::Short)
			.set_size(2.into())
			.set_leverage(5.into())
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch id
			U256::from(1_u8),
			// size
			FixedI128::from_inner(1500000000000000000),
			// market
			market_id,
			// price
			100.into(),
			// orders
			vec![alice_open_order_1.clone(), bob_open_order_1.clone()],
			// batch_timestamp
			1699940367000,
		));
	});
}

#[test]
#[should_panic(expected = "TradeBatchError517")]
// trade batch with taker order size is not multiple of step size
fn it_reverts_for_trade_with_taker_order_size_not_multiple_of_step_size() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());

		// market id
		let market_id = btc_usdc().market.id;

		// Create orders
		let alice_open_order_1 = Order::new(U256::from(201), alice_id)
			.set_leverage(5.into())
			.sign_order(get_private_key(alice().pub_key));

		let bob_open_order_1 = Order::new(U256::from(202), bob_id)
			.set_order_type(OrderType::Market)
			.set_direction(Direction::Short)
			.set_size(FixedI128::from_inner(1500000000000000000))
			.set_leverage(5.into())
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch id
			U256::from(1_u8),
			// size
			1.into(),
			// market
			market_id,
			// price
			100.into(),
			// orders
			vec![alice_open_order_1.clone(), bob_open_order_1.clone()],
			// batch_timestamp
			1699940367000,
		));
	});
}

#[test]
// trade batch with maker order size is not multiple of step size
fn it_emits_event_for_trade_with_maker_order_size_not_multiple_of_step_size() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());

		// market id
		let market_id = btc_usdc().market.id;

		// Create orders
		let alice_open_order_1 = Order::new(U256::from(201), alice_id)
			.set_leverage(5.into())
			.set_size(FixedI128::from_inner(1500000000000000000))
			.sign_order(get_private_key(alice().pub_key));

		let bob_open_order_1 = Order::new(U256::from(202), bob_id)
			.set_order_type(OrderType::Market)
			.set_direction(Direction::Short)
			.set_leverage(5.into())
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch id
			U256::from(1_u8),
			// size
			1.into(),
			// market
			market_id,
			// price
			100.into(),
			// orders
			vec![alice_open_order_1.clone(), bob_open_order_1.clone()],
			// batch_timestamp
			1699940367000,
		));
		System::assert_has_event(
			Event::OrderError { order_id: U256::from(201), error_code: 517 }.into(),
		);
	});
}

#[test]
#[should_panic(expected = "TradeBatchError524")]
// Taker tries to close a position which is already completely closed
fn it_reverts_when_taker_tries_to_close_already_closed_position() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());

		// market id
		let market_id = btc_usdc().market.id;

		let alice_open_order_1 = Order::new(U256::from(201), alice_id)
			.set_size(2.into())
			.sign_order(get_private_key(alice().pub_key));

		let bob_open_order_1 = Order::new(U256::from(202), bob_id)
			.set_order_type(OrderType::Market)
			.set_direction(Direction::Short)
			.set_size(2.into())
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch id
			U256::from(1_u8),
			// size
			2.into(),
			// market
			market_id,
			// price
			100.into(),
			// orders
			vec![alice_open_order_1.clone(), bob_open_order_1.clone()],
			// batch_timestamp
			1699940367000,
		));

		let alice_close_order_1 = Order::new(U256::from(203), alice_id)
			.set_side(Side::Sell)
			.set_size(2.into())
			.set_price(104.into())
			.sign_order(get_private_key(alice().pub_key));

		let bob_close_order_1 = Order::new(U256::from(204), bob_id)
			.set_order_type(OrderType::Market)
			.set_direction(Direction::Short)
			.set_side(Side::Sell)
			.set_size(2.into())
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch_id
			U256::from(2_u8),
			// size
			2.into(),
			// market_id
			market_id,
			// price
			105.into(),
			vec![alice_close_order_1, bob_close_order_1],
			// batch_timestamp
			1699940367000,
		));

		let alice_open_order_2 = Order::new(U256::from(205), alice_id)
			.set_direction(Direction::Short)
			.set_price(98.into())
			.sign_order(get_private_key(alice().pub_key));

		let bob_close_order_2 = Order::new(U256::from(206), bob_id)
			.set_direction(Direction::Short)
			.set_side(Side::Sell)
			.set_price(98.into())
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch_id
			U256::from(3_u8),
			// size
			1.into(),
			// market_id
			market_id,
			// price
			100.into(),
			vec![alice_open_order_2, bob_close_order_2],
			// batch_timestamp
			1699940367000,
		));
	});
}

#[test]
// Non registered user tries to open a position
fn it_produces_error_when_user_not_registered() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		// Adding 1 to simulate a non-registered user
		let alice_id: U256 = get_trading_account_id(alice()) + 1;
		let bob_id: U256 = get_trading_account_id(bob());

		// market id
		let market_id = btc_usdc().market.id;

		let alice_open_order_1 = Order::new(U256::from(201), alice_id)
			.set_size(2.into())
			.sign_order(get_private_key(alice().pub_key));

		let bob_open_order_1 = Order::new(U256::from(202), bob_id)
			.set_order_type(OrderType::Market)
			.set_direction(Direction::Short)
			.set_size(2.into())
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch_id
			U256::from(1_u8),
			// size
			2.into(),
			// market_id
			market_id,
			// price
			100.into(),
			// order
			vec![alice_open_order_1.clone(), bob_open_order_1.clone()],
			// batch_timestamp
			1699940367000,
		));

		System::assert_has_event(
			Event::OrderError { order_id: U256::from(201), error_code: 510 }.into(),
		);
	});
}

#[test]
#[should_panic(expected = "TradeBatchError505")]
// Tries to open a position with size lesser than allowed minimum order size
fn it_produces_error_when_size_too_small() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());

		// market id
		let market_id = btc_usdc().market.id;

		let alice_open_order_1 = Order::new(U256::from(201), alice_id)
			.set_size(2.into())
			.sign_order(get_private_key(alice().pub_key));

		let bob_open_order_1 = Order::new(U256::from(202), bob_id)
			.set_order_type(OrderType::Market)
			.set_direction(Direction::Short)
			.set_size(FixedI128::from_inner(500000000000000000))
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch_id
			U256::from(1_u8),
			// size
			1.into(),
			// market_id
			market_id,
			// price
			100.into(),
			// orders
			vec![alice_open_order_1.clone(), bob_open_order_1.clone()],
			// batch_timestamp
			1699940367000,
		));

		System::assert_has_event(
			Event::OrderError { order_id: U256::from(201), error_code: 505 }.into(),
		);
	});
}

#[test]
// Tries to open a position with different market_id compared to the one passed in argument
fn it_produces_error_when_market_id_is_different() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());

		// market id
		let market_id = btc_usdc().market.id;

		let alice_open_order_1 = Order::new(U256::from(201), alice_id)
			.set_market_id(789)
			.sign_order(get_private_key(alice().pub_key));

		let bob_open_order_1 = Order::new(U256::from(202), bob_id)
			.set_order_type(OrderType::Market)
			.set_direction(Direction::Short)
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch_id
			U256::from(1_u8),
			// size
			2.into(),
			// market_id
			market_id,
			// price
			100.into(),
			// order
			vec![alice_open_order_1.clone(), bob_open_order_1.clone()],
			// batch_timestamp
			1699940367000,
		));

		System::assert_has_event(
			Event::OrderError { order_id: U256::from(201), error_code: 504 }.into(),
		);
	});
}

#[test]
// Tries to open a position leverage more than currently allowed leverage
fn it_produces_error_when_leverage_is_invalid() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());

		// market id
		let market_id = btc_usdc().market.id;

		let alice_open_order_1 = Order::new(U256::from(201), alice_id)
			.set_leverage(9.into())
			.sign_order(get_private_key(alice().pub_key));

		let bob_open_order_1 = Order::new(U256::from(202), bob_id)
			.set_order_type(OrderType::Market)
			.set_direction(Direction::Short)
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch_id
			U256::from(1_u8),
			// size
			2.into(),
			// market_id
			market_id,
			// price
			100.into(),
			// order
			vec![alice_open_order_1.clone(), bob_open_order_1.clone()],
			// batch_timestamp
			1699940367000,
		));

		System::assert_has_event(
			Event::OrderError { order_id: U256::from(201), error_code: 502 }.into(),
		);
	});
}

#[test]
// Tries to open a position with invalid signature
fn it_produces_error_when_signature_is_invalid() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());

		// market id
		let market_id = btc_usdc().market.id;

		let alice_open_order_1 =
			Order::new(U256::from(201), alice_id).sign_order(get_private_key(charlie().pub_key));

		let bob_open_order_1 = Order::new(U256::from(202), bob_id)
			.set_order_type(OrderType::Market)
			.set_direction(Direction::Short)
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch_id
			U256::from(1_u8),
			// size
			2.into(),
			// market_id
			market_id,
			// price
			100.into(),
			// order
			vec![alice_open_order_1.clone(), bob_open_order_1.clone()],
			// batch_timestamp
			1699940367000,
		));

		System::assert_has_event(
			Event::OrderError { order_id: U256::from(201), error_code: 536 }.into(),
		);
	});
}

#[test]
// 2nd maker order with side and direction that does not match with the first maker
fn it_produces_error_for_maker_when_side_and_direction_is_invalid() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());
		let charlie_id: U256 = get_trading_account_id(charlie());

		// market id
		let market_id = btc_usdc().market.id;

		let alice_open_order_1 = Order::new(U256::from(201), alice_id)
			.set_direction(Direction::Short)
			.set_price(105.into())
			.sign_order(get_private_key(alice().pub_key));

		let bob_open_order_1 = Order::new(U256::from(202), bob_id)
			.set_order_type(OrderType::Market)
			.set_price(99.into())
			.sign_order(get_private_key(bob().pub_key));

		let charlie_open_order_1 = Order::new(U256::from(203), charlie_id)
			.set_size(3.into())
			.set_order_type(OrderType::Market)
			.sign_order(get_private_key(charlie().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch_id
			U256::from(1_u8),
			// size
			3.into(),
			// market_id
			market_id,
			// price
			100.into(),
			// orders
			vec![
				alice_open_order_1.clone(),
				bob_open_order_1.clone(),
				charlie_open_order_1.clone()
			],
			// batch_timestamp
			1699940367000,
		));

		System::assert_has_event(
			Event::OrderError { order_id: U256::from(202), error_code: 512 }.into(),
		);
	});
}

#[test]
// Maker order type is not limit
fn it_produces_error_when_maker_is_market_order() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());

		// market id
		let market_id = btc_usdc().market.id;

		// Create orders
		let alice_order = Order::new(U256::from(201), alice_id)
			.set_order_type(OrderType::Market)
			.sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(U256::from(202), bob_id)
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

		System::assert_has_event(
			Event::OrderError { order_id: U256::from(201), error_code: 518 }.into(),
		);
	});
}

#[test]
// Maker tries to close a position which is already completely closed
fn it_reverts_when_maker_tries_to_close_already_closed_position() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());

		// market id
		let market_id = btc_usdc().market.id;

		// Create orders
		let alice_open_order_1 = Order::new(U256::from(201), alice_id)
			.set_size(2.into())
			.sign_order(get_private_key(alice().pub_key));
		let bob_open_order_1 = Order::new(U256::from(202), bob_id)
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.set_size(2.into())
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch_id
			U256::from(1_u8),
			// quantity_locked
			2.into(),
			// market_id
			market_id,
			// oracle_price
			100.into(),
			// orders
			vec![alice_open_order_1.clone(), bob_open_order_1.clone()],
			// batch_timestamp
			1699940367000,
		));

		let alice_close_order_1 = Order::new(U256::from(203), alice_id)
			.set_side(Side::Sell)
			.set_size(2.into())
			.set_price(104.into())
			.sign_order(get_private_key(alice().pub_key));
		let bob_close_order_1 = Order::new(U256::from(204), bob_id)
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.set_side(Side::Sell)
			.set_size(2.into())
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch_id
			U256::from(2_u8),
			// quantity_locked
			2.into(),
			// market_id
			market_id,
			// oracle_price
			100.into(),
			// orders
			vec![alice_close_order_1.clone(), bob_close_order_1.clone()],
			// batch_timestamp
			1699940367000,
		));

		let alice_close_order_2 = Order::new(U256::from(205), alice_id)
			.set_side(Side::Sell)
			.set_price(98.into())
			.sign_order(get_private_key(alice().pub_key));
		let bob_close_order_2 = Order::new(U256::from(206), bob_id)
			.set_price(98.into())
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch_id
			U256::from(3_u8),
			// quantity_locked
			1.into(),
			// market_id
			market_id,
			// oracle_price
			100.into(),
			// orders
			vec![alice_close_order_2.clone(), bob_close_order_2.clone()],
			// batch_timestamp
			1699940367000,
		));

		let event_record: frame_system::EventRecord<_, _> = System::events().pop().unwrap();
		println!("Events: {:?}", event_record);

		System::assert_has_event(
			Event::OrderError { order_id: U256::from(205), error_code: 524 }.into(),
		);
	});
}

#[test]
#[should_panic(expected = "TradeBatchError")]
// taker order with side and direction that does not match with the maker
fn it_produces_error_for_taker_when_side_and_direction_is_invalid() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());
		let charlie_id: U256 = get_trading_account_id(charlie());

		// market id
		let market_id = btc_usdc().market.id;

		let alice_open_order_1 = Order::new(U256::from(201), alice_id)
			.set_price(105.into())
			.sign_order(get_private_key(alice().pub_key));

		let bob_open_order_1 = Order::new(U256::from(202), bob_id)
			.set_price(99.into())
			.sign_order(get_private_key(bob().pub_key));

		let charlie_open_order_1 = Order::new(U256::from(203), charlie_id)
			.set_size(3.into())
			.set_order_type(OrderType::Market)
			.sign_order(get_private_key(charlie().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch_id
			U256::from(1_u8),
			// size
			3.into(),
			// market_id
			market_id,
			// price
			100.into(),
			// orders
			vec![
				alice_open_order_1.clone(),
				bob_open_order_1.clone(),
				charlie_open_order_1.clone()
			],
			// batch_timestamp
			1699940367000,
		));

		System::assert_has_event(
			Event::OrderError { order_id: U256::from(203), error_code: 511 }.into(),
		);
	});
}

#[test]
// Taker long buy limit order execution price is invalid
fn it_produces_error_when_taker_long_buy_limit_price_invalid() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());

		// market id
		let market_id = btc_usdc().market.id;

		let alice_open_order_1 = Order::new(U256::from(201), alice_id)
			.set_direction(Direction::Short)
			.sign_order(get_private_key(alice().pub_key));

		let bob_open_order_1 = Order::new(U256::from(202), bob_id)
			.set_price(99.into())
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch_id
			U256::from(1_u8),
			// size
			1.into(),
			// market_id
			market_id,
			// price
			100.into(),
			// order
			vec![alice_open_order_1.clone(), bob_open_order_1.clone()],
			// batch_timestamp
			1699940367000,
		));

		System::assert_has_event(
			Event::OrderError { order_id: U256::from(201), error_code: 508 }.into(),
		);
	});
}

#[test]
// Taker short buy limit order execution price is invalid
fn it_produces_error_when_taker_short_buy_limit_price_invalid() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());

		// market id
		let market_id = btc_usdc().market.id;

		let alice_open_order_1 =
			Order::new(U256::from(201), alice_id).sign_order(get_private_key(alice().pub_key));

		let bob_open_order_1 = Order::new(U256::from(202), bob_id)
			.set_price(101.into())
			.set_direction(Direction::Short)
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch_id
			U256::from(1_u8),
			// size
			1.into(),
			// market_id
			market_id,
			// price
			100.into(),
			// order
			vec![alice_open_order_1.clone(), bob_open_order_1.clone()],
			// batch_timestamp
			1699940367000,
		));

		System::assert_has_event(
			Event::OrderError { order_id: U256::from(201), error_code: 507 }.into(),
		);
	});
}

#[test]
#[should_panic(expected = "TradeBatchError514")]
// Taker long buy slippage check
fn it_produces_error_when_taker_long_buy_price_not_within_slippage() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());

		// market id
		let market_id = btc_usdc().market.id;

		let alice_open_order_1 = Order::new(U256::from(201), alice_id)
			.set_direction(Direction::Short)
			.set_price(111.into())
			.sign_order(get_private_key(alice().pub_key));

		let bob_open_order_1 = Order::new(U256::from(202), bob_id)
			.set_order_type(OrderType::Market)
			.set_price(99.into())
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch_id
			U256::from(1_u8),
			// size
			1.into(),
			// market_id
			market_id,
			// price
			100.into(),
			// order
			vec![alice_open_order_1.clone(), bob_open_order_1.clone()],
			// batch_timestamp
			1699940367000,
		));
	});
}

#[test]
// Taker long buy slippage check when execution price very low
fn it_works_when_taker_long_buy_price_very_low() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());

		// market id
		let market_id = btc_usdc().market.id;

		let alice_open_order_1 = Order::new(U256::from(201), alice_id)
			.set_direction(Direction::Short)
			.set_price(80.into())
			.sign_order(get_private_key(alice().pub_key));

		let bob_open_order_1 = Order::new(U256::from(202), bob_id)
			.set_order_type(OrderType::Market)
			.set_price(100.into())
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch_id
			U256::from(1_u8),
			// size
			1.into(),
			// market_id
			market_id,
			// price
			100.into(),
			// order
			vec![alice_open_order_1.clone(), bob_open_order_1.clone()],
			// batch_timestamp
			1699940367000,
		));
	});
}

#[test]
fn test_fee_while_opening_order() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());

		// market id
		let market_id = btc_usdc().market.id;
		let collateral_id = usdc().asset.id;

		let (fee_details_maker, fee_details_taker) = setup_fee();
		// Dispatch a signed extrinsic.
		assert_ok!(TradingFees::update_base_fees(
			RuntimeOrigin::root(),
			collateral_id,
			Side::Buy,
			OrderSide::Maker,
			fee_details_maker.clone(),
		));
		assert_ok!(TradingFees::update_base_fees(
			RuntimeOrigin::root(),
			collateral_id,
			Side::Buy,
			OrderSide::Taker,
			fee_details_taker.clone(),
		));

		// Create orders
		let alice_open_order_1 =
			Order::new(U256::from(201), alice_id).sign_order(get_private_key(alice().pub_key));
		let bob_open_order_1 = Order::new(U256::from(202), bob_id)
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
			vec![alice_open_order_1.clone(), bob_open_order_1.clone()],
			// batch_timestamp
			1699940367000,
		));

		// Check for events
		assert_has_events(vec![
			Event::TradeExecuted {
				batch_id: U256::from(1_u8),
				market_id,
				size: 1.into(),
				execution_price: 100.into(),
				direction: bob_open_order_1.direction.into(),
				side: bob_open_order_1.side.into(),
			}
			.into(),
			Event::OrderExecuted {
				account_id: alice_id,
				order_id: alice_open_order_1.order_id,
				market_id,
				size: 1.into(),
				direction: alice_open_order_1.direction.into(),
				side: alice_open_order_1.side.into(),
				order_type: alice_open_order_1.order_type.into(),
				execution_price: 100.into(),
				pnl: (-2).into(),
				fee: 2.into(),
				is_final: true,
				is_maker: true,
			}
			.into(),
			Event::OrderExecuted {
				account_id: bob_id,
				order_id: bob_open_order_1.order_id,
				market_id,
				size: 1.into(),
				direction: bob_open_order_1.direction.into(),
				side: bob_open_order_1.side.into(),
				order_type: bob_open_order_1.order_type.into(),
				execution_price: 100.into(),
				pnl: (-5).into(),
				fee: 5.into(),
				is_final: true,
				is_maker: false,
			}
			.into(),
		]);

		assert_eq!(
			TradingAccounts::balances(alice_id, collateral_id),
			FixedI128::from_inner(9998000000000000000000)
		);
		assert_eq!(
			TradingAccounts::balances(bob_id, collateral_id),
			FixedI128::from_inner(9995000000000000000000)
		);

		let alice_close_order_1 = Order::new(U256::from(203), alice_id)
			.set_side(Side::Sell)
			.set_price(105.into())
			.sign_order(get_private_key(alice().pub_key));

		let bob_close_order_1 = Order::new(U256::from(204), bob_id)
			.set_side(Side::Sell)
			.set_price(100.into())
			.set_order_type(OrderType::Market)
			.set_direction(Direction::Short)
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch_id
			U256::from(2_u8),
			// quantity_locked
			1.into(),
			// market_id
			market_id,
			// oracle_price
			105.into(),
			// orders
			vec![alice_close_order_1.clone(), bob_close_order_1.clone()],
			// batch_timestamp
			1699940367000,
		));

		assert_eq!(
			TradingAccounts::balances(alice_id, collateral_id),
			FixedI128::from_inner(10003000000000000000000)
		);
		assert_eq!(
			TradingAccounts::balances(bob_id, collateral_id),
			FixedI128::from_inner(9990000000000000000000)
		);
		assert_eq!(TradingAccounts::locked_margin(alice_id, collateral_id), 0.into());

		// Check for events
		assert_has_events(vec![
			Event::TradeExecuted {
				batch_id: U256::from(2_u8),
				market_id,
				size: 1.into(),
				execution_price: 105.into(),
				direction: bob_close_order_1.direction.into(),
				side: bob_close_order_1.side.into(),
			}
			.into(),
			Event::OrderExecuted {
				account_id: alice_id,
				order_id: alice_close_order_1.order_id,
				market_id,
				size: 1.into(),
				direction: alice_close_order_1.direction.into(),
				side: alice_close_order_1.side.into(),
				order_type: alice_close_order_1.order_type.into(),
				execution_price: 105.into(),
				pnl: 5.into(),
				fee: 0.into(),
				is_final: true,
				is_maker: true,
			}
			.into(),
			Event::OrderExecuted {
				account_id: bob_id,
				order_id: bob_close_order_1.order_id,
				market_id,
				size: 1.into(),
				direction: bob_close_order_1.direction.into(),
				side: bob_close_order_1.side.into(),
				order_type: bob_close_order_1.order_type.into(),
				execution_price: 105.into(),
				pnl: (-5).into(),
				fee: 0.into(),
				is_final: true,
				is_maker: false,
			}
			.into(),
		]);
	});
}

#[test]
fn test_fee_while_closing_order() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());

		// market id
		let market_id = btc_usdc().market.id;
		let collateral_id = usdc().asset.id;

		let (fee_details_maker, fee_details_taker) = setup_fee();
		// Dispatch a signed extrinsic.
		assert_ok!(TradingFees::update_base_fees(
			RuntimeOrigin::root(),
			collateral_id,
			Side::Sell,
			OrderSide::Maker,
			fee_details_maker.clone(),
		));
		assert_ok!(TradingFees::update_base_fees(
			RuntimeOrigin::root(),
			collateral_id,
			Side::Sell,
			OrderSide::Taker,
			fee_details_taker.clone(),
		));

		// Create orders
		let alice_open_order_1 =
			Order::new(U256::from(201), alice_id).sign_order(get_private_key(alice().pub_key));
		let bob_open_order_1 = Order::new(U256::from(202), bob_id)
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
			vec![alice_open_order_1.clone(), bob_open_order_1.clone()],
			// batch_timestamp
			1699940367000,
		));

		// Check for events
		assert_has_events(vec![
			Event::TradeExecuted {
				batch_id: U256::from(1_u8),
				market_id,
				size: 1.into(),
				execution_price: 100.into(),
				direction: bob_open_order_1.direction.into(),
				side: bob_open_order_1.side.into(),
			}
			.into(),
			Event::OrderExecuted {
				account_id: alice_id,
				order_id: alice_open_order_1.order_id,
				market_id,
				size: 1.into(),
				direction: alice_open_order_1.direction.into(),
				side: alice_open_order_1.side.into(),
				order_type: alice_open_order_1.order_type.into(),
				execution_price: 100.into(),
				pnl: 0.into(),
				fee: 0.into(),
				is_final: true,
				is_maker: true,
			}
			.into(),
			Event::OrderExecuted {
				account_id: bob_id,
				order_id: bob_open_order_1.order_id,
				market_id,
				size: 1.into(),
				direction: bob_open_order_1.direction.into(),
				side: bob_open_order_1.side.into(),
				order_type: bob_open_order_1.order_type.into(),
				execution_price: 100.into(),
				pnl: 0.into(),
				fee: 0.into(),
				is_final: true,
				is_maker: false,
			}
			.into(),
		]);

		// Since we are opening orders without setting the fee for open orders, fee won't be
		// deducted from balance
		let usdc_id: u128 = usdc().asset.id;
		let balance_1 = TradingAccounts::balances(alice_id, usdc_id);
		assert_eq!(balance_1, 10000.into());
		let balance_2 = TradingAccounts::balances(bob_id, usdc_id);
		assert_eq!(balance_2, 10000.into());

		// Close orders
		let alice_close_order_1 = Order::new(U256::from(203), alice_id)
			.set_side(Side::Sell)
			.set_price(105.into())
			.sign_order(get_private_key(alice().pub_key));

		let bob_close_order_1 = Order::new(U256::from(204), bob_id)
			.set_side(Side::Sell)
			.set_price(100.into())
			.set_order_type(OrderType::Market)
			.set_direction(Direction::Short)
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch_id
			U256::from(2_u8),
			// quantity_locked
			1.into(),
			// market_id
			market_id,
			// oracle_price
			105.into(),
			// orders
			vec![alice_close_order_1.clone(), bob_close_order_1.clone()],
			// batch_timestamp
			1699940367000,
		));

		// Check for events
		assert_has_events(vec![
			Event::TradeExecuted {
				batch_id: U256::from(2_u8),
				market_id,
				size: 1.into(),
				execution_price: 105.into(),
				direction: bob_close_order_1.direction.into(),
				side: bob_close_order_1.side.into(),
			}
			.into(),
			Event::OrderExecuted {
				account_id: alice_id,
				order_id: alice_close_order_1.order_id,
				market_id,
				size: 1.into(),
				direction: alice_close_order_1.direction.into(),
				side: alice_close_order_1.side.into(),
				order_type: alice_close_order_1.order_type.into(),
				execution_price: 105.into(),
				pnl: FixedI128::from_inner((2.9 * 10u128.pow(18) as f64) as i128),
				fee: FixedI128::from_inner((2.1 * 10u128.pow(18) as f64) as i128),
				is_final: true,
				is_maker: true,
			}
			.into(),
			Event::OrderExecuted {
				account_id: bob_id,
				order_id: bob_close_order_1.order_id,
				market_id,
				size: 1.into(),
				direction: bob_close_order_1.direction.into(),
				side: bob_close_order_1.side.into(),
				order_type: bob_close_order_1.order_type.into(),
				execution_price: 105.into(),
				pnl: FixedI128::from_inner((-9.75 * 10u128.pow(18) as f64) as i128),
				fee: FixedI128::from_inner((4.75 * 10u128.pow(18) as f64) as i128),
				is_final: true,
				is_maker: false,
			}
			.into(),
		]);

		assert_eq!(
			TradingAccounts::balances(alice_id, collateral_id),
			FixedI128::from_inner(10002900000000000000000)
		);
		assert_eq!(
			TradingAccounts::balances(bob_id, collateral_id),
			FixedI128::from_inner(9990250000000000000000)
		);
		assert_eq!(TradingAccounts::locked_margin(alice_id, collateral_id), 0.into());
	});
}

#[test]
// cleanup of order and batch details
fn it_works_for_cleanup() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());

		// market id
		let market_id = btc_usdc().market.id;

		// Create order 1
		let alice_order =
			Order::new(U256::from(201), alice_id).sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(U256::from(202), bob_id)
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
			1699940360000,
		));

		Timestamp::set_timestamp(1702359600000);
		let b = Timestamp::now();
		print!("Block time {:?}", b);

		// Create order 2
		let alice_order = Order::new(U256::from(203), alice_id)
			.set_timestamp(1702359500000)
			.sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(U256::from(204), bob_id)
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.set_timestamp(1702359400000)
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch_id
			U256::from(2_u8),
			// quantity_locked
			1.into(),
			// market_id
			market_id,
			// oracle_price
			100.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			1702359550000,
		));

		assert_ok!(Trading::perform_cleanup(RuntimeOrigin::signed(1)));

		let order1 = Trading::order_state(U256::from(201));
		assert_eq!(order1.0, FixedI128::zero());
		let order2 = Trading::order_state(U256::from(202));
		assert_eq!(order2.0, FixedI128::zero());
		let order3 = Trading::order_state(U256::from(203));
		assert_eq!(order3.0, FixedI128::one());
		let order4 = Trading::order_state(U256::from(204));
		assert_eq!(order4.0, FixedI128::one());

		let order1 = Trading::order_hash(U256::from(201));
		assert_eq!(order1, U256::zero());
		let order2 = Trading::order_hash(U256::from(202));
		assert_eq!(order2, U256::zero());
		let order3 = Trading::order_hash(U256::from(203));
		assert_ne!(order3, U256::zero());
		let order4 = Trading::order_hash(U256::from(204));
		assert_ne!(order4, U256::zero());

		let batch1 = Trading::batch_status(U256::from(1_u8));
		assert_eq!(batch1, false);
		let batch2 = Trading::batch_status(U256::from(2_u8));
		assert_eq!(batch2, true);

		let start_timestamp = Trading::start_timestamp();
		assert_eq!(1699940400, start_timestamp.unwrap());

		let timestamp1 = Trading::orders(1699940278);
		assert_eq!(false, timestamp1.is_some());
		let timestamp2 = Trading::orders(1702359500);
		assert_eq!(vec![U256::from(203)], timestamp2.unwrap());
		let timestamp3 = Trading::orders(1702359400);
		assert_eq!(vec![U256::from(204)], timestamp3.unwrap());

		let timestamp1 = Trading::batches(1699940360);
		assert_eq!(false, timestamp1.is_some());
		let timestamp2 = Trading::batches(1702359550);
		assert_eq!(vec![U256::from(2_u8)], timestamp2.unwrap());
	});
}

#[test]
#[should_panic(expected = "TradeBatchError545")]
// batch older than 4 weeks
fn it_does_not_work_for_old_batch() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());

		// market id
		let market_id = btc_usdc().market.id;

		// Create order 1
		let alice_order =
			Order::new(U256::from(201), alice_id).sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(U256::from(202), bob_id)
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
			1697521100000,
		));
	});
}

#[test]
#[should_panic(expected = "TradeBatchError544")]
// batch older than 4 weeks
fn it_does_not_work_for_old_order() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());

		// market id
		let market_id = btc_usdc().market.id;

		// Create order 1
		let alice_order =
			Order::new(U256::from(201), alice_id).sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(U256::from(202), bob_id)
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.set_timestamp(1697521100)
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
			1699940360000,
		));
	});
}

#[test]
fn it_does_not_work_for_not_enough_balance() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());

		// market id
		let market_id = btc_usdc().market.id;
		let collateral_id = usdc().asset.id;

		// Check for balances
		assert_eq!(TradingAccounts::balances(alice_id, collateral_id), 10000.into());
		assert_eq!(TradingAccounts::balances(bob_id, collateral_id), 10000.into());

		// Create orders
		let alice_order = Order::new(U256::from(301), alice_id)
			.set_leverage(8.into())
			.set_size(10.into())
			.set_price(35000.into())
			.sign_order(get_private_key(alice().pub_key));

		let bob_order = Order::new(U256::from(302), bob_id)
			.set_order_type(OrderType::Market)
			.set_direction(Direction::Short)
			.set_leverage(8.into())
			.set_size(10.into())
			.set_price(35000.into())
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch id
			U256::from(1_u8),
			// size
			10.into(),
			// market
			market_id,
			// price
			35000.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			1699940367000,
		));

		System::assert_has_event(
			Event::OrderError { order_id: U256::from(301), error_code: 501 }.into(),
		);
		System::assert_has_event(Event::TradeExecutionFailed { batch_id: U256::from(1_u8) }.into());
	});
}

#[test]
// trade batch with 2 takers in which one taker's price is valid to taker and other taker's price
// is invalid to taker, so second maker should not get executed
fn it_works_when_one_maker_price_is_valid_for_taker() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());
		let dave_id: U256 = get_trading_account_id(dave());

		// market id
		let market_id = btc_usdc().market.id;

		let alice_open_order_1 = Order::new(U256::from(201), alice_id)
			.set_price(3000.into())
			.set_direction(Direction::Short)
			.sign_order(get_private_key(alice().pub_key));

		let bob_open_order_1 = Order::new(U256::from(202), bob_id)
			.set_price(2010.into())
			.set_direction(Direction::Short)
			.sign_order(get_private_key(bob().pub_key));

		let dave_open_order_1 = Order::new(U256::from(204), dave_id)
			.set_price(0.into())
			.set_size(2.into())
			.set_order_type(OrderType::Market)
			.set_slippage(FixedI128::from_inner(50000000000000000))
			.sign_order(get_private_key(dave().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch_id
			U256::from(1_u8),
			// size
			2.into(),
			// market_id
			market_id,
			// price
			2000.into(),
			// orders
			vec![alice_open_order_1.clone(), bob_open_order_1.clone(), dave_open_order_1],
			// batch_timestamp
			1699940367000,
		));

		System::assert_has_event(
			Event::OrderError { order_id: U256::from(201), error_code: 506 }.into(),
		);
	});
}

#[test]
// Taker short buy limit order has 0 slippage - which shouldn't make any difference
fn it_works_when_taker_limit_order_has_0_slippage() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());

		// market id
		let market_id = btc_usdc().market.id;

		let alice_open_order_1 =
			Order::new(U256::from(201), alice_id).sign_order(get_private_key(alice().pub_key));

		let bob_open_order_1 = Order::new(U256::from(202), bob_id)
			.set_price(101.into())
			.set_direction(Direction::Short)
			.set_slippage(FixedI128::zero())
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch_id
			U256::from(1_u8),
			// size
			1.into(),
			// market_id
			market_id,
			// price
			102.into(),
			// order
			vec![alice_open_order_1.clone(), bob_open_order_1.clone()],
			// batch_timestamp
			1699940367000,
		));
	});
}

#[test]
#[should_panic(expected = "TradeBatchError514")]
// Only one maker and maker fails due to slippage error
// Taker also should emit OrderError event
fn it_emits_error_for_taker_for_slippage_validation() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());

		// market id
		let market_id = btc_usdc().market.id;

		let alice_open_order_1 = Order::new(U256::from(201), alice_id)
			.set_price(80.into())
			.sign_order(get_private_key(alice().pub_key));

		let bob_open_order_1 = Order::new(U256::from(202), bob_id)
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch_id
			U256::from(1_u8),
			// size
			1.into(),
			// market_id
			market_id,
			// price
			100.into(),
			// order
			vec![alice_open_order_1.clone(), bob_open_order_1.clone()],
			// batch_timestamp
			1699940367000,
		));
	});
}

#[test]
// When there are 3 makers and one maker fails with 506 and another maker fails with some other
// error and 3rd maker executes, taker's is_final flag should be made false
fn it_makes_taker_is_final_as_false() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());
		let charlie_id: U256 = get_trading_account_id(charlie());
		let dave_id: U256 = get_trading_account_id(dave());

		// market id
		let market_id = btc_usdc().market.id;

		let alice_open_order_1 = Order::new(U256::from(201), alice_id)
			.set_price(80.into())
			.sign_order(get_private_key(alice().pub_key));

		let charlie_open_order_1 = Order::new(U256::from(203), charlie_id)
			.set_price(102.into())
			.sign_order(get_private_key(charlie().pub_key));

		let dave_open_order_1 = Order::new(U256::from(204), dave_id)
			.set_price(100.into())
			.set_order_type(OrderType::Market)
			.sign_order(get_private_key(dave().pub_key));

		let bob_open_order_1 = Order::new(U256::from(202), bob_id)
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.set_size(3.into())
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch_id
			U256::from(1_u8),
			// size
			3.into(),
			// market_id
			market_id,
			// price
			100.into(),
			// order
			vec![
				alice_open_order_1.clone(),
				charlie_open_order_1,
				dave_open_order_1,
				bob_open_order_1.clone()
			],
			// batch_timestamp
			1699940367000,
		));

		assert_has_events(vec![Event::OrderExecuted {
			account_id: bob_id,
			order_id: bob_open_order_1.order_id,
			market_id,
			size: 1.into(),
			direction: bob_open_order_1.direction.into(),
			side: bob_open_order_1.side.into(),
			order_type: bob_open_order_1.order_type.into(),
			execution_price: 102.into(),
			pnl: 0.into(),
			fee: 0.into(),
			is_final: false,
			is_maker: false,
		}
		.into()]);
	});
}

#[test]
// When there are 3 makers and one maker fails with 506
// and other makers execute, taker's is_final flag should be made true
fn it_makes_taker_is_final_as_true() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());
		let charlie_id: U256 = get_trading_account_id(charlie());
		let dave_id: U256 = get_trading_account_id(dave());

		// market id
		let market_id = btc_usdc().market.id;

		let alice_open_order_1 = Order::new(U256::from(201), alice_id)
			.set_price(80.into())
			.sign_order(get_private_key(alice().pub_key));

		let charlie_open_order_1 = Order::new(U256::from(203), charlie_id)
			.set_price(102.into())
			.sign_order(get_private_key(charlie().pub_key));

		let dave_open_order_1 = Order::new(U256::from(204), dave_id)
			.set_price(100.into())
			.sign_order(get_private_key(dave().pub_key));

		let bob_open_order_1 = Order::new(U256::from(202), bob_id)
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.set_size(3.into())
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch_id
			U256::from(1_u8),
			// size
			3.into(),
			// market_id
			market_id,
			// price
			100.into(),
			// order
			vec![
				alice_open_order_1.clone(),
				charlie_open_order_1,
				dave_open_order_1,
				bob_open_order_1.clone()
			],
			// batch_timestamp
			1699940367000,
		));

		assert_has_events(vec![Event::OrderExecuted {
			account_id: bob_id,
			order_id: bob_open_order_1.order_id,
			market_id,
			size: 2.into(),
			direction: bob_open_order_1.direction.into(),
			side: bob_open_order_1.side.into(),
			order_type: bob_open_order_1.order_type.into(),
			execution_price: 101.into(),
			pnl: 0.into(),
			fee: 0.into(),
			is_final: true,
			is_maker: false,
		}
		.into()]);
	});
}

#[test]
// open positions in 2 markets and close position in one market
fn it_works_for_multiple_open_and_single_close_trade() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());
		let charlie_id: U256 = get_trading_account_id(charlie());

		// Open order for BTC-USDC
		let market_id = btc_usdc().market.id;

		let alice_open_order =
			Order::new(U256::from(201), alice_id).sign_order(get_private_key(alice().pub_key));
		let bob_open_order = Order::new(U256::from(202), bob_id)
			.set_order_type(OrderType::Market)
			.set_direction(Direction::Short)
			.sign_order(get_private_key(bob().pub_key));

		// Execute the trade
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
			vec![alice_open_order.clone(), bob_open_order.clone()],
			// batch_timestamp
			1699940367000,
		));

		// Open order for ETH-USDC
		let market_id = eth_usdc().market.id;

		let alice_open_order = Order::new(U256::from(205), alice_id)
			.set_price(10.into())
			.set_market_id(market_id)
			.sign_order(get_private_key(alice().pub_key));
		let charlie_open_order = Order::new(U256::from(206), charlie_id)
			.set_order_type(OrderType::Market)
			.set_direction(Direction::Short)
			.set_price(10.into())
			.set_market_id(market_id)
			.sign_order(get_private_key(charlie().pub_key));

		// Execute the trade
		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch_id
			U256::from(3_u8),
			// quantity_locked
			1.into(),
			// market_id
			market_id,
			// oracle_price
			10.into(),
			// orders
			vec![alice_open_order.clone(), charlie_open_order.clone()],
			// batch_timestamp
			1699940367000,
		));

		// Close orders
		let market_id = btc_usdc().market.id;

		let alice_close_order = Order::new(U256::from(203), alice_id)
			.set_side(Side::Sell)
			.set_price(105.into())
			.sign_order(get_private_key(alice().pub_key));

		let bob_close_order = Order::new(U256::from(204), bob_id)
			.set_side(Side::Sell)
			.set_price(100.into())
			.set_order_type(OrderType::Market)
			.set_direction(Direction::Short)
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch_id
			U256::from(2_u8),
			// quantity_locked
			1.into(),
			// market_id
			market_id,
			// oracle_price
			105.into(),
			// orders
			vec![alice_close_order.clone(), bob_close_order.clone()],
			// batch_timestamp
			1699940367000,
		));

		let markets = Trading::collateral_to_market(alice_id, usdc().asset.id);
		assert_eq!(markets, vec![eth_usdc().market.id]);
	});
}

#[test]
#[should_panic(expected = "TradeBatchError532")]
// user tries to close a position, but does not have enough balance to cover loses
fn it_reverts_when_user_cant_cover_losses() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());

		// market id
		let market_id = btc_usdc().market.id;
		let collateral_id = usdc().asset.id;

		// Create open orders
		let alice_open_order = Order::new(U256::from(201), alice_id)
			.set_price(7500.into())
			.set_size(8.into())
			.set_leverage(8.into())
			.sign_order(get_private_key(alice().pub_key));
		let bob_open_order = Order::new(U256::from(202), bob_id)
			.set_order_type(OrderType::Market)
			.set_direction(Direction::Short)
			.set_price(7500.into())
			.set_size(8.into())
			.set_leverage(8.into())
			.sign_order(get_private_key(bob().pub_key));

		// Execute the trade
		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch_id
			U256::from(1_u8),
			// quantity_locked
			8.into(),
			// market_id
			market_id,
			// oracle_price
			7500.into(),
			// orders
			vec![alice_open_order.clone(), bob_open_order.clone()],
			// batch_timestamp
			1699940367000,
		));

		// Close close orders
		let alice_close_order = Order::new(U256::from(203), alice_id)
			.set_side(Side::Sell)
			.set_price(9000.into())
			.set_size(8.into())
			.sign_order(get_private_key(alice().pub_key));

		let bob_close_order = Order::new(U256::from(204), bob_id)
			.set_side(Side::Sell)
			.set_price(9000.into())
			.set_order_type(OrderType::Market)
			.set_direction(Direction::Short)
			.set_size(8.into())
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch_id
			U256::from(2_u8),
			// quantity_locked
			8.into(),
			// market_id
			market_id,
			// oracle_price
			9000.into(),
			// orders
			vec![alice_close_order.clone(), bob_close_order.clone()],
			// batch_timestamp
			1699940367000,
		));

		println!("Eventssss: {:?}", System::events());
	});
}
