use crate::{mock::*, Event};
use frame_support::assert_ok;
use pallet_support::{
	test_helpers::{
		accounts_helper::{alice, bob, charlie, dave, get_private_key, get_trading_account_id},
		asset_helper::{btc, eth, link, usdc},
		market_helper::{btc_usdc, eth_usdc, link_usdc},
		setup_fee,
	},
	traits::{FixedI128Ext, TradingAccountInterface, TradingInterface},
	types::{
		BaseFee, BaseFeeAggregate, Direction, FeeRates, FeeShareDetails, FeeSharesInput, Order,
		OrderSide, OrderType, Position, ReferralDetails, Side, TradingAccount,
	},
};
use pallet_trading_account::Event as TradingAccountEvent;
use primitive_types::U256;
use sp_arithmetic::{
	traits::{One, Zero},
	FixedI128,
};
use sp_runtime::print;

fn assert_has_events(expected_events: Vec<RuntimeEvent>) {
	for expected_event in &expected_events {
		if !System::events().iter().any(|event| event.event == *expected_event) {
			panic!("Expected event not found: {:?}", expected_event);
		}
	}
}

pub fn get_usdc_fee_shares() -> Vec<Vec<FeeShareDetails>> {
	vec![
		vec![
			FeeShareDetails {
				volume: FixedI128::from_u32(0),
				fee_share: FixedI128::from_float(0.0),
			},
			FeeShareDetails {
				volume: FixedI128::from_u32(1000),
				fee_share: FixedI128::from_float(0.05),
			},
			FeeShareDetails {
				volume: FixedI128::from_u32(2000),
				fee_share: FixedI128::from_float(0.08),
			},
			FeeShareDetails {
				volume: FixedI128::from_u32(6000),
				fee_share: FixedI128::from_float(0.1),
			},
			FeeShareDetails {
				volume: FixedI128::from_u32(25000),
				fee_share: FixedI128::from_float(0.12),
			},
			FeeShareDetails {
				volume: FixedI128::from_u32(50000),
				fee_share: FixedI128::from_float(0.15),
			},
		],
		vec![
			FeeShareDetails {
				volume: FixedI128::from_u32(0),
				fee_share: FixedI128::from_float(0.0),
			},
			FeeShareDetails {
				volume: FixedI128::from_u32(1000),
				fee_share: FixedI128::from_float(0.5),
			},
			FeeShareDetails {
				volume: FixedI128::from_u32(2000),
				fee_share: FixedI128::from_float(0.5),
			},
			FeeShareDetails {
				volume: FixedI128::from_u32(6000),
				fee_share: FixedI128::from_float(0.5),
			},
			FeeShareDetails {
				volume: FixedI128::from_u32(25000),
				fee_share: FixedI128::from_float(0.5),
			},
			FeeShareDetails {
				volume: FixedI128::from_u32(50000),
				fee_share: FixedI128::from_float(0.5),
			},
		],
	]
}

pub fn get_usdc_aggregate_fees() -> BaseFeeAggregate {
	BaseFeeAggregate {
		maker_buy: vec![
			BaseFee { volume: FixedI128::from_u32(0), fee: FixedI128::from_float(0.001) },
			BaseFee { volume: FixedI128::from_u32(1000), fee: FixedI128::from_float(0.00050) },
			BaseFee { volume: FixedI128::from_u32(4000), fee: FixedI128::from_float(0.00020) },
			BaseFee { volume: FixedI128::from_u32(10000), fee: FixedI128::from_float(0.00010) },
			BaseFee { volume: FixedI128::from_u32(50000), fee: FixedI128::from_float(0.0) },
		],
		maker_sell: vec![
			BaseFee { volume: FixedI128::from_u32(0), fee: FixedI128::from_float(0.001) },
			BaseFee { volume: FixedI128::from_u32(1000), fee: FixedI128::from_float(0.00050) },
			BaseFee { volume: FixedI128::from_u32(4000), fee: FixedI128::from_float(0.00020) },
			BaseFee { volume: FixedI128::from_u32(10000), fee: FixedI128::from_float(0.00010) },
			BaseFee { volume: FixedI128::from_u32(50000), fee: FixedI128::from_float(0.0) },
		],
		taker_buy: vec![
			BaseFee { volume: FixedI128::from_u32(0), fee: FixedI128::from_float(0.001) },
			BaseFee { volume: FixedI128::from_u32(1000), fee: FixedI128::from_float(0.00080) },
			BaseFee { volume: FixedI128::from_u32(4000), fee: FixedI128::from_float(0.00050) },
			BaseFee { volume: FixedI128::from_u32(10000), fee: FixedI128::from_float(0.00040) },
			BaseFee { volume: FixedI128::from_u32(50000), fee: FixedI128::from_float(0.00020) },
		],
		taker_sell: vec![
			BaseFee { volume: FixedI128::from_u32(0), fee: FixedI128::from_float(0.001) },
			BaseFee { volume: FixedI128::from_u32(1000), fee: FixedI128::from_float(0.00080) },
			BaseFee { volume: FixedI128::from_u32(4000), fee: FixedI128::from_float(0.00050) },
			BaseFee { volume: FixedI128::from_u32(10000), fee: FixedI128::from_float(0.00040) },
			BaseFee { volume: FixedI128::from_u32(50000), fee: FixedI128::from_float(0.00020) },
		],
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			vec![eth(), usdc(), link(), btc()]
		));
		assert_ok!(Markets::replace_all_markets(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			vec![btc_usdc(), link_usdc(), eth_usdc()]
		));

		// Add accounts to the system
		assert_ok!(TradingAccounts::add_accounts(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			vec![alice(), bob(), charlie(), dave()]
		));

		// Set matching_time_limit
		assert_ok!(Trading::set_matching_time_limit(
			RuntimeOrigin::root(),
			2419200 //4 weeks
		));

		// Set default insurance fund
		assert_ok!(TradingAccounts::set_default_insurance_fund(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			U256::from(1_u8),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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

		let account_id = Trading::market_to_user((market_id, Direction::Long), alice_id);
		assert_eq!(account_id.unwrap(), alice_id);
		let account_id = Trading::market_to_user((market_id, Direction::Short), alice_id);
		assert_eq!(account_id, None);
		let account_id = Trading::market_to_user((market_id, Direction::Long), bob_id);
		assert_eq!(account_id, None);
		let account_id = Trading::market_to_user((market_id, Direction::Short), bob_id);
		assert_eq!(account_id.unwrap(), bob_id);

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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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

		assert_has_events(vec![Event::OrderError {
			order_id: U256::from(201),
			account_id: alice_id,
			error_code: 548,
		}
		.into()]);
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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

		let account_id = Trading::market_to_user((market_id, Direction::Long), alice_id);
		assert_eq!(account_id.unwrap(), alice_id);

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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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

		let account_id = Trading::market_to_user((market_id, Direction::Long), alice_id);
		assert_eq!(account_id, None);

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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			Event::OrderError { order_id: U256::from(201), account_id: alice_id, error_code: 517 }
				.into(),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			Event::OrderError { order_id: U256::from(201), account_id: alice_id, error_code: 510 }
				.into(),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			Event::OrderError { order_id: U256::from(201), account_id: alice_id, error_code: 505 }
				.into(),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			Event::OrderError { order_id: U256::from(201), account_id: alice_id, error_code: 504 }
				.into(),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			Event::OrderError { order_id: U256::from(201), account_id: alice_id, error_code: 502 }
				.into(),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			Event::OrderError { order_id: U256::from(201), account_id: alice_id, error_code: 536 }
				.into(),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			Event::OrderError { order_id: U256::from(202), account_id: bob_id, error_code: 512 }
				.into(),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			Event::OrderError { order_id: U256::from(201), account_id: alice_id, error_code: 518 }
				.into(),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			Event::OrderError { order_id: U256::from(205), account_id: alice_id, error_code: 524 }
				.into(),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			Event::OrderError {
				order_id: U256::from(203),
				account_id: charlie_id,
				error_code: 511,
			}
			.into(),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			Event::OrderError { order_id: U256::from(201), account_id: alice_id, error_code: 508 }
				.into(),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			Event::OrderError { order_id: U256::from(201), account_id: alice_id, error_code: 507 }
				.into(),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			BaseFeeAggregate {
				maker_buy: fee_details_maker.clone(),
				maker_sell: vec![BaseFee { volume: FixedI128::zero(), fee: FixedI128::zero() }],
				taker_buy: fee_details_taker.clone(),
				taker_sell: vec![BaseFee { volume: FixedI128::zero(), fee: FixedI128::zero() }],
			}
		));

		// Create orders
		let alice_open_order_1 =
			Order::new(U256::from(201), alice_id).sign_order(get_private_key(alice().pub_key));
		let bob_open_order_1 = Order::new(U256::from(202), bob_id)
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
				fee: FixedI128::from_inner(0),
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
				fee: FixedI128::from_inner(0),
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
			BaseFeeAggregate {
				maker_buy: vec![BaseFee { volume: FixedI128::zero(), fee: FixedI128::zero() }],
				maker_sell: fee_details_maker.clone(),
				taker_buy: vec![BaseFee { volume: FixedI128::zero(), fee: FixedI128::zero() }],
				taker_sell: fee_details_taker.clone(),
			}
		));

		// Create orders
		let alice_open_order_1 =
			Order::new(U256::from(201), alice_id).sign_order(get_private_key(alice().pub_key));
		let bob_open_order_1 = Order::new(U256::from(202), bob_id)
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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

		print!("Events: {:?}", System::events());

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
fn test_fee_share_1() {
	let mut env = setup();

	// test env
	// Generate account_ids
	let alice_id: U256 = get_trading_account_id(alice());
	let bob_id: U256 = get_trading_account_id(bob());
	let charlie_account_address = charlie().account_address;

	let market_id = btc_usdc().market.id;
	let collateral_id = usdc().asset.id;

	let init_timestamp: u64 = 1699940367;
	let one_day: u64 = 24 * 60 * 60;

	let initial_balance = FixedI128::from_float(10000.0);

	env.execute_with(|| {
		// Add referral data
		assert_ok!(TradingAccounts::add_referral(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			alice().account_address,
			ReferralDetails {
				master_account_address: charlie().account_address,
				fee_discount: FixedI128::from_float(0.1),
			},
			U256::from(123),
		));

		assert_ok!(TradingAccounts::add_referral(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			bob().account_address,
			ReferralDetails {
				master_account_address: charlie().account_address,
				fee_discount: FixedI128::from_float(0.1),
			},
			U256::from(123),
		));

		// Add fee data
		assert_ok!(TradingFees::update_base_fees(
			RuntimeOrigin::root(),
			collateral_id,
			get_usdc_aggregate_fees()
		));

		// Add fee_share_data
		assert_ok!(TradingFees::update_fee_share(
			RuntimeOrigin::root(),
			collateral_id,
			get_usdc_fee_shares()
		));

		////////////////////
		// Day 1: Batch 1 //
		////////////////////

		// Create orders
		let alice_order = Order::new(201.into(), alice_id)
			.set_price(FixedI128::from_float(1001.0))
			.sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(202.into(), bob_id)
			.set_price(FixedI128::from_float(1001.0))
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			// batch_id
			2.into(),
			// quantity_locked
			1.into(),
			// market_id
			market_id,
			// oracle_price
			1001.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			init_timestamp * 1000,
		));

		let charlie_30day_master_volume =
			TradingAccounts::get_30day_master_volume(charlie_account_address, market_id).unwrap();
		assert_eq!(charlie_30day_master_volume, 0.into(), "Error in 30 day volume");

		let master_fee_share_1 =
			TradingAccounts::master_account_fee_share(charlie_account_address, collateral_id);
		assert!(master_fee_share_1 == FixedI128::zero(), "wrong master fee share");

		// Alice's current tier is 1
		// fee_rate = 0.001 (0.1%)
		// fee_discount = 0.1 (10%)
		// effective_fee_rate = 0.001 * (1-0.1)
		let alice_balance_1 = TradingAccounts::balances(alice_id, collateral_id);
		assert!(
			alice_balance_1 ==
				initial_balance -
					(FixedI128::from_float(1001.0) * FixedI128::from_float(0.0009)),
			"Invalid fee rate for Alice day 1 batch 1"
		);

		// Bob's current tier is 1
		// fee_rate = 0.001 (0.1%)
		// fee_discount = 0.1 (10%)
		// effective_fee_rate = 0.001 * (1-0.1)
		let bob_balance_1 = TradingAccounts::balances(bob_id, collateral_id);
		assert!(
			bob_balance_1 ==
				initial_balance -
					(FixedI128::from_float(1001.0) * FixedI128::from_float(0.0009)),
			"Invalid fee rate for Bob day 1 batch 1"
		);

		////////////////////
		// Day 1: Batch 2 //
		////////////////////

		// Create orders
		let alice_order = Order::new(203.into(), alice_id)
			.set_price(FixedI128::from_float(1001.0))
			.sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(204.into(), bob_id)
			.set_price(FixedI128::from_float(1001.0))
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			// batch_id
			3.into(),
			// quantity_locked
			1.into(),
			// market_id
			market_id,
			// oracle_price
			1001.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			init_timestamp * 1000,
		));

		let charlie_30day_master_volume =
			TradingAccounts::get_30day_master_volume(charlie_account_address, market_id).unwrap();
		assert_eq!(charlie_30day_master_volume, 0.into(), "Error in 30 day volume");

		let master_fee_share_2 =
			TradingAccounts::master_account_fee_share(charlie_account_address, collateral_id);
		assert!(master_fee_share_2 == FixedI128::zero(), "wrong master fee share");

		// Alice's current tier is 1
		// fee_rate = 0.001 (0.1%)
		// fee_discount = 0.1 (10%)
		// effective_fee_rate = 0.001 * (1-0.1)
		let alice_balance_2 = TradingAccounts::balances(alice_id, collateral_id);
		assert!(
			alice_balance_2 ==
				alice_balance_1 -
					(FixedI128::from_float(1001.0) * FixedI128::from_float(0.0009)),
			"Invalid fee rate for Alice day 1 batch 2"
		);

		// Bob's current tier is 1
		// fee_rate = 0.001 (0.1%)
		// fee_discount = 0.1 (10%)
		// effective_fee_rate = 0.001 * (1-0.1)
		let bob_balance_2 = TradingAccounts::balances(bob_id, collateral_id);
		assert!(
			bob_balance_2 ==
				bob_balance_1 - (FixedI128::from_float(1001.0) * FixedI128::from_float(0.0009)),
			"Invalid fee rate for Bob day 1 batch 2"
		);

		////////////////////
		// Day 2: Batch 1 //
		////////////////////

		// next trade on next day i.e. day 2
		Timestamp::set_timestamp((init_timestamp + one_day + 1) * 1000);

		// Create orders
		let alice_order = Order::new(205.into(), alice_id)
			.set_price(FixedI128::from_float(1001.0))
			.sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(206.into(), bob_id)
			.set_price(FixedI128::from_float(1001.0))
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			// batch_id
			4.into(),
			// quantity_locked
			1.into(),
			// market_id
			market_id,
			// oracle_price
			1001.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			(init_timestamp + one_day) * 1000,
		));

		let charlie_30day_master_volume =
			TradingAccounts::get_30day_master_volume(charlie_account_address, market_id).unwrap();
		assert_eq!(charlie_30day_master_volume, 4004.into(), "Error in 30 day volume");

		// Alice's current tier is 2
		// fee_rate = 0.00050 (0.05%)
		// fee_discount = 0.1 (10%)
		// effective_fee_rate = 0.00050 * (1-0.1)
		let alice_balance_3 = TradingAccounts::balances(alice_id, collateral_id);
		let alice_fee_3 = FixedI128::from_float(1001.0) * FixedI128::from_float(0.00045);
		assert!(
			alice_balance_3 == alice_balance_2 - alice_fee_3,
			"Invalid fee rate for Alice day 2 batch 1"
		);

		// Bob's current tier is 2
		// fee_rate = 0.0008 (0.08%)
		// fee_discount = 0.1 (10%)
		// effective_fee_rate = 0.0008 * (1-0.1)
		let bob_balance_3 = TradingAccounts::balances(bob_id, collateral_id);
		let bob_fee_3 = FixedI128::from_float(1001.0) * FixedI128::from_float(0.00072);
		assert!(
			bob_balance_3 == bob_balance_2 - bob_fee_3,
			"Invalid fee rate for Bob day 2 batch 1"
		);

		// Charlie's master's current tier is 3
		// fee_share_rate = 0.08 (8%)
		let master_fee_share_3 =
			TradingAccounts::master_account_fee_share(charlie_account_address, collateral_id);
		let expected_master_fee_share_3 =
			alice_fee_3 * FixedI128::from_float(0.08) + bob_fee_3 * FixedI128::from_float(0.08);
		assert!(
			master_fee_share_3 == expected_master_fee_share_3.round_to_precision(6),
			"wrong master fee share"
		);

		////////////////////
		// Day 2: Batch 2 //
		////////////////////

		// Create orders
		let alice_order = Order::new(207.into(), alice_id)
			.set_price(FixedI128::from_float(1001.0))
			.sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(208.into(), bob_id)
			.set_price(FixedI128::from_float(1001.0))
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			// batch_id
			5.into(),
			// quantity_locked
			1.into(),
			// market_id
			market_id,
			// oracle_price
			1001.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			(init_timestamp + one_day) * 1000,
		));

		let charlie_30day_master_volume =
			TradingAccounts::get_30day_master_volume(charlie_account_address, market_id).unwrap();
		assert_eq!(charlie_30day_master_volume, 4004.into(), "Error in 30 day volume");

		// Alice's current tier is 2
		// fee_rate = 0.00050 (0.05%)
		// fee_discount = 0.1 (10%)
		// effective_fee_rate = 0.00050 * (1-0.1)
		let alice_balance_4 = TradingAccounts::balances(alice_id, collateral_id);
		let alice_fee_4 = FixedI128::from_float(1001.0) * FixedI128::from_float(0.00045);
		assert!(
			alice_balance_4 == alice_balance_3 - alice_fee_4,
			"Invalid fee rate for Alice day 2 batch 2"
		);

		// Bob's current tier is 2
		// fee_rate = 0.0008 (0.08%)
		// fee_discount = 0.1 (10%)
		// effective_fee_rate = 0.0008 * (1-0.1)
		let bob_balance_4 = TradingAccounts::balances(bob_id, collateral_id);
		let bob_fee_4 = FixedI128::from_float(1001.0) * FixedI128::from_float(0.00072);
		assert!(
			bob_balance_4 == bob_balance_3 - bob_fee_4,
			"Invalid fee rate for Bob day 2 batch 2"
		);

		// Charlie's master's current tier is 3
		// fee_share_rate = 0.08 (8%)
		let master_fee_share_4 =
			TradingAccounts::master_account_fee_share(charlie_account_address, collateral_id);
		let expected_master_fee_share_4 =
			alice_fee_4 * FixedI128::from_float(0.08) + bob_fee_4 * FixedI128::from_float(0.08);
		assert!(
			master_fee_share_4 ==
				master_fee_share_3 + expected_master_fee_share_4.round_to_precision(6),
			"wrong master fee share"
		);

		////////////////////
		// Day 3: Batch 1 //
		////////////////////

		// next trade on next day i.e. day 2
		Timestamp::set_timestamp((init_timestamp + one_day * 2 + 1) * 1000);

		// Create orders
		let alice_order = Order::new(209.into(), alice_id)
			.set_price(FixedI128::from_float(1001.0))
			.sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(210.into(), bob_id)
			.set_price(FixedI128::from_float(1001.0))
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			// batch_id
			6.into(),
			// quantity_locked
			1.into(),
			// market_id
			market_id,
			// oracle_price
			1001.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			(init_timestamp + one_day) * 1000,
		));

		let charlie_30day_master_volume =
			TradingAccounts::get_30day_master_volume(charlie_account_address, market_id).unwrap();
		assert_eq!(charlie_30day_master_volume, 8008.into(), "Error in 30 day volume");

		// Alice's current tier is 3
		// fee_rate = 0.00020 (0.02%)
		// fee_discount = 0.1 (10%)
		// effective_fee_rate = 0.00020 * (1-0.1)
		let alice_balance_5 = TradingAccounts::balances(alice_id, collateral_id);
		let alice_fee_5 = FixedI128::from_float(1001.0) * FixedI128::from_float(0.00018);
		assert!(
			alice_balance_5 == alice_balance_4 - alice_fee_5,
			"Invalid fee rate for Alice day 3 batch 1"
		);

		// Bob's current tier is 3
		// fee_rate = 0.0005 (0.05%)
		// fee_discount = 0.1 (10%)
		// effective_fee_rate = 0.0005 * (1-0.1)
		let bob_balance_5 = TradingAccounts::balances(bob_id, collateral_id);
		let bob_fee_5 = FixedI128::from_float(1001.0) * FixedI128::from_float(0.00045);
		assert!(
			bob_balance_5 == bob_balance_4 - bob_fee_5,
			"Invalid fee rate for Bob day 3 batch 1"
		);

		// Charlie's master's current tier is 4
		// fee_share_rate = 0.1 (10%)
		let master_fee_share_5 =
			TradingAccounts::master_account_fee_share(charlie_account_address, collateral_id);
		let expected_master_fee_share_5 =
			alice_fee_5 * FixedI128::from_float(0.1) + bob_fee_5 * FixedI128::from_float(0.1);
		assert!(
			master_fee_share_5 ==
				master_fee_share_4 + expected_master_fee_share_5.round_to_precision(6),
			"wrong master fee share"
		);

		////////////////////
		// Day 3: Batch 2 //
		////////////////////

		// Upgrade charlie to level 1
		assert_ok!(TradingAccounts::update_master_account_level(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			charlie_account_address,
			1
		));

		// Create orders
		let alice_order = Order::new(211.into(), alice_id)
			.set_price(FixedI128::from_float(1001.0))
			.sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(212.into(), bob_id)
			.set_price(FixedI128::from_float(1001.0))
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			// batch_id
			7.into(),
			// quantity_locked
			1.into(),
			// market_id
			market_id,
			// oracle_price
			1001.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			(init_timestamp + one_day) * 1000,
		));

		let charlie_30day_master_volume =
			TradingAccounts::get_30day_master_volume(charlie_account_address, market_id).unwrap();
		assert_eq!(charlie_30day_master_volume, 8008.into(), "Error in 30 day volume");

		// Alice's current tier is 3
		// fee_rate = 0.00020 (0.02%)
		// fee_discount = 0.1 (10%)
		// effective_fee_rate = 0.00020 * (1-0.1)
		let alice_balance_6 = TradingAccounts::balances(alice_id, collateral_id);
		let alice_fee_6 = FixedI128::from_float(1001.0) * FixedI128::from_float(0.00018);
		assert!(
			alice_balance_6 == alice_balance_5 - alice_fee_6,
			"Invalid fee rate for Alice day 3 batch 1"
		);

		// Bob's current tier is 3
		// fee_rate = 0.0005 (0.05%)
		// fee_discount = 0.1 (10%)
		// effective_fee_rate = 0.0005 * (1-0.1)
		let bob_balance_6 = TradingAccounts::balances(bob_id, collateral_id);
		let bob_fee_6 = FixedI128::from_float(1001.0) * FixedI128::from_float(0.00045);
		assert!(
			bob_balance_6 == bob_balance_5 - bob_fee_6,
			"Invalid fee rate for Bob day 3 batch 1"
		);

		// Charlie's master's current tier is 4
		// but level 1
		// fee_share_rate = 0.5 (50%)
		let master_fee_share_6 =
			TradingAccounts::master_account_fee_share(charlie_account_address, collateral_id);
		let expected_master_fee_share_6 =
			alice_fee_6 * FixedI128::from_float(0.5) + bob_fee_6 * FixedI128::from_float(0.5);
		assert!(
			master_fee_share_6 ==
				master_fee_share_5 + expected_master_fee_share_6.round_to_precision(6),
			"wrong master fee share"
		);

		// Emit FeeShareTransfer for Charlie
		assert_ok!(TradingAccounts::pay_fee_shares(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			vec![FeeSharesInput {
				master_account_address: charlie_account_address,
				collateral_id,
				amount: master_fee_share_6,
			},]
		));

		assert_has_events(vec![TradingAccountEvent::FeeShareTransfer {
			master_account_address: charlie_account_address,
			collateral_id,
			amount: master_fee_share_6,
			block_number: 1,
		}
		.into()]);

		assert!(
			TradingAccounts::master_account_fee_share(charlie_account_address, collateral_id) ==
				FixedI128::zero(),
			"wrong master fee share"
		);
	});
}

#[test]
#[should_panic(expected = "No default insurance fund set")]
fn test_trade_without_default_insurance_fund() {
	// Create a new test environment
	let mut env = new_test_ext();

	// Generate account_ids
	let alice_id: U256 = get_trading_account_id(alice());
	let bob_id: U256 = get_trading_account_id(bob());

	let init_timestamp: u64 = 1699940367;

	// market id
	let btc_market_id = btc_usdc().market.id;
	let collateral_id = usdc().asset.id;

	env.execute_with(|| {
		// Set the block number
		System::set_block_number(1);
		assert_ok!(Timestamp::set(None.into(), 1699940367000));

		// Set the assets in the system
		assert_ok!(Assets::replace_all_assets(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			vec![eth(), usdc(), link(), btc()]
		));
		assert_ok!(Markets::replace_all_markets(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			vec![btc_usdc(), link_usdc(), eth_usdc()]
		));

		// Add accounts to the system
		assert_ok!(TradingAccounts::add_accounts(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			vec![alice(), bob(), charlie(), dave()]
		));

		// Set matching_time_limit
		assert_ok!(Trading::set_matching_time_limit(
			RuntimeOrigin::root(),
			2419200 //4 weeks
		));

		// Add fee data
		assert_ok!(TradingFees::update_base_fees(
			RuntimeOrigin::root(),
			collateral_id,
			get_usdc_aggregate_fees()
		));

		// Create orders
		let alice_order = Order::new(201.into(), alice_id)
			.set_price(FixedI128::from_float(1001.0))
			.sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(202.into(), bob_id)
			.set_price(FixedI128::from_float(1001.0))
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			// batch_id
			1.into(),
			// quantity_locked
			1.into(),
			// market_id
			btc_market_id,
			// oracle_price
			1001.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			init_timestamp * 1000,
		));

		print!("Events: {:?}", System::events());
	});
}

#[test]
fn test_insurance_fund_replacement() {
	let mut env = setup();

	// Generate account_ids
	let alice_id: U256 = get_trading_account_id(alice());
	let bob_id: U256 = get_trading_account_id(bob());

	let init_timestamp: u64 = 1699940367;

	// market id
	let btc_market_id = btc_usdc().market.id;
	let collateral_id = usdc().asset.id;

	// Insurance funds
	let btc_insurance_fund_1: U256 = 2.into();
	let btc_fee_split_1 = FixedI128::from_float(0.1_f64);
	let btc_insurance_fund_2: U256 = 3.into();
	let btc_fee_split_2 = FixedI128::from_float(0.8_f64);

	env.execute_with(|| {
		// Add fee data
		assert_ok!(TradingFees::update_base_fees(
			RuntimeOrigin::root(),
			collateral_id,
			get_usdc_aggregate_fees()
		));

		///////////////////////////////
		// Isolated 1: BTC-USDC Open //
		///////////////////////////////

		// Set insurance fund for BTC
		assert_ok!(TradingAccounts::update_fee_split_details(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			btc_market_id,
			btc_insurance_fund_1,
			btc_fee_split_1
		));

		// balance check
		assert!(
			TradingAccounts::insurance_fund_balance(btc_insurance_fund_1, collateral_id) ==
				FixedI128::zero(),
			"Invalid balance for isolated insurance balance 1 before trade"
		);

		// Create orders
		let alice_order = Order::new(201.into(), alice_id)
			.set_price(FixedI128::from_float(1001.0))
			.sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(202.into(), bob_id)
			.set_price(FixedI128::from_float(1001.0))
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			// batch_id
			1.into(),
			// quantity_locked
			1.into(),
			// market_id
			btc_market_id,
			// oracle_price
			1001.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			init_timestamp * 1000,
		));

		let alice_fees_1 = FixedI128::from_float(1001.0) * FixedI128::from_float(0.001);
		let alice_fees_contribution_1 = alice_fees_1 * (FixedI128::one() - btc_fee_split_1);
		let bob_fees_1 = FixedI128::from_float(1001.0) * FixedI128::from_float(0.001);
		let bob_fees_contribution_1 = bob_fees_1 * (FixedI128::one() - btc_fee_split_1);

		let expected_btc_insurance_fund_balance_1 =
			alice_fees_contribution_1 + bob_fees_contribution_1;
		// balance check
		assert!(
			TradingAccounts::insurance_fund_balance(btc_insurance_fund_1, collateral_id) ==
				expected_btc_insurance_fund_balance_1,
			"Invalid balance for isolated insurance balance 1 after trade"
		);

		//////////////////////////////
		// Isolated: BTC-USDC Close //
		//////////////////////////////
		// Create orders
		let alice_order = Order::new(203.into(), alice_id)
			.set_side(Side::Sell)
			.set_price(FixedI128::from_float(1001.0))
			.sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(204.into(), bob_id)
			.set_side(Side::Sell)
			.set_price(FixedI128::from_float(1001.0))
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			// batch_id
			2.into(),
			// quantity_locked
			1.into(),
			// market_id
			btc_market_id,
			// oracle_price
			1001.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			init_timestamp * 1000,
		));

		let alice_fees_2 = FixedI128::from_float(1001.0) * FixedI128::from_float(0.001);
		let alice_fees_contribution_2 = alice_fees_2 * (FixedI128::one() - btc_fee_split_1);
		let bob_fees_2: FixedI128 = FixedI128::from_float(1001.0) * FixedI128::from_float(0.001);
		let bob_fees_contribution_2 = bob_fees_2 * (FixedI128::one() - btc_fee_split_1);

		let expected_btc_insurance_fund_balance_2 = expected_btc_insurance_fund_balance_1 +
			alice_fees_contribution_2 +
			bob_fees_contribution_2;

		// balance check
		assert!(
			TradingAccounts::insurance_fund_balance(btc_insurance_fund_1, collateral_id) ==
				expected_btc_insurance_fund_balance_2,
			"Invalid balance for btc insurance balance"
		);

		///////////////////////////////
		// Isolated 2: BTC-USDC Open //
		///////////////////////////////

		// Set insurance fund for BTC
		assert_ok!(TradingAccounts::update_fee_split_details(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			btc_market_id,
			btc_insurance_fund_2,
			btc_fee_split_2
		));

		// balance check
		assert!(
			TradingAccounts::insurance_fund_balance(btc_insurance_fund_2, collateral_id) ==
				FixedI128::zero(),
			"Invalid balance for isolated insurance balance 2 before trade"
		);

		// Create orders
		let alice_order = Order::new(205.into(), alice_id)
			.set_price(FixedI128::from_float(1001.0))
			.sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(206.into(), bob_id)
			.set_price(FixedI128::from_float(1001.0))
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			// batch_id
			3.into(),
			// quantity_locked
			1.into(),
			// market_id
			btc_market_id,
			// oracle_price
			1001.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			init_timestamp * 1000,
		));

		let alice_fees_1 = FixedI128::from_float(1001.0) * FixedI128::from_float(0.001);
		let alice_fees_contribution_1 = alice_fees_1 * (FixedI128::one() - btc_fee_split_2);
		let bob_fees_1 = FixedI128::from_float(1001.0) * FixedI128::from_float(0.001);
		let bob_fees_contribution_1 = bob_fees_1 * (FixedI128::one() - btc_fee_split_2);

		let expected_default_insurance_fund_balance_1 =
			alice_fees_contribution_1 + bob_fees_contribution_1;

		// balance check
		assert!(
			TradingAccounts::insurance_fund_balance(btc_insurance_fund_2, collateral_id) ==
				expected_default_insurance_fund_balance_1,
			"Invalid balance for isolated insurance balance 2 after trade"
		);

		////////////////////////////////
		// Isolated 2: BTC-USDC Close //
		////////////////////////////////

		// Create orders
		let alice_order = Order::new(207.into(), alice_id)
			.set_price(FixedI128::from_float(1001.0))
			.sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(208.into(), bob_id)
			.set_price(FixedI128::from_float(1001.0))
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			// batch_id
			4.into(),
			// quantity_locked
			1.into(),
			// market_id
			btc_market_id,
			// oracle_price
			1001.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			init_timestamp * 1000,
		));

		let alice_fees_2 = FixedI128::from_float(1001.0) * FixedI128::from_float(0.001);
		let alice_fees_contribution_2 = alice_fees_2 * (FixedI128::one() - btc_fee_split_2);
		let bob_fees_2 = FixedI128::from_float(1001.0) * FixedI128::from_float(0.001);
		let bob_fees_contribution_2 = bob_fees_2 * (FixedI128::one() - btc_fee_split_2);

		let expected_default_insurance_fund_balance_2 = expected_default_insurance_fund_balance_1 +
			alice_fees_contribution_2 +
			bob_fees_contribution_2;

		// balance check
		assert!(
			TradingAccounts::insurance_fund_balance(btc_insurance_fund_2, collateral_id) ==
				expected_default_insurance_fund_balance_2,
			"Invalid balance for isolated insurance balance 2 after trade"
		);
	});
}

#[test]
fn test_insurance_fund_update() {
	let mut env = setup();

	// Generate account_ids
	let alice_id: U256 = get_trading_account_id(alice());
	let bob_id: U256 = get_trading_account_id(bob());

	let init_timestamp: u64 = 1699940367;

	// market id
	let btc_market_id = btc_usdc().market.id;
	let eth_market_id = eth_usdc().market.id;
	let collateral_id = usdc().asset.id;

	// Insurance funds
	let default_insurance_fund: U256 = 1.into();
	let btc_insurance_fund: U256 = 2.into();
	let btc_fee_split = FixedI128::from_float(0.1_f64);
	let eth_insurance_fund: U256 = 3.into();
	let eth_fee_split = FixedI128::from_float(0.5_f64);

	env.execute_with(|| {
		// Add fee data
		assert_ok!(TradingFees::update_base_fees(
			RuntimeOrigin::root(),
			collateral_id,
			get_usdc_aggregate_fees()
		));

		// Set default insurance fund
		assert_ok!(TradingAccounts::set_default_insurance_fund(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			default_insurance_fund,
		));

		// balance check
		assert!(
			TradingAccounts::insurance_fund_balance(default_insurance_fund, collateral_id) ==
				FixedI128::zero(),
			"Invalid balance for default insurance balance"
		);

		////////////////////////////
		// Default: BTC-USDC Open //
		////////////////////////////

		// Create orders
		let alice_order = Order::new(201.into(), alice_id)
			.set_price(FixedI128::from_float(1001.0))
			.sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(202.into(), bob_id)
			.set_price(FixedI128::from_float(1001.0))
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			// batch_id
			1.into(),
			// quantity_locked
			1.into(),
			// market_id
			btc_market_id,
			// oracle_price
			1001.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			init_timestamp * 1000,
		));

		let alice_fees_1 = FixedI128::from_float(1001.0) * FixedI128::from_float(0.001);
		let bob_fees_1 = FixedI128::from_float(1001.0) * FixedI128::from_float(0.001);

		let expected_default_insurance_fund_balance_1 = alice_fees_1 + bob_fees_1;
		// balance check
		assert!(
			TradingAccounts::insurance_fund_balance(default_insurance_fund, collateral_id) ==
				expected_default_insurance_fund_balance_1,
			"Invalid balance for default insurance balance"
		);

		/////////////////////////////
		// Default: BTC-USDC Close //
		/////////////////////////////

		// Create orders
		let alice_order = Order::new(203.into(), alice_id)
			.set_price(FixedI128::from_float(1001.0))
			.set_side(Side::Sell)
			.sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(204.into(), bob_id)
			.set_side(Side::Sell)
			.set_price(FixedI128::from_float(1001.0))
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			// batch_id
			2.into(),
			// quantity_locked
			1.into(),
			// market_id
			btc_market_id,
			// oracle_price
			1001.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			init_timestamp * 1000,
		));

		let alice_fees_2 = FixedI128::from_float(1001.0) * FixedI128::from_float(0.001);
		let bob_fees_2 = FixedI128::from_float(1001.0) * FixedI128::from_float(0.001);

		let expected_default_insurance_fund_balance_2 =
			expected_default_insurance_fund_balance_1 + alice_fees_2 + bob_fees_2;
		// balance check
		assert!(
			TradingAccounts::insurance_fund_balance(default_insurance_fund, collateral_id) ==
				expected_default_insurance_fund_balance_2,
			"Invalid balance for default insurance balance"
		);

		////////////////////////////
		// Default: ETH-USDC Open //
		////////////////////////////

		// Create orders
		let alice_order = Order::new(205.into(), alice_id)
			.set_price(FixedI128::from_float(101.0))
			.set_market_id(eth_market_id)
			.sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(206.into(), bob_id)
			.set_price(FixedI128::from_float(101.0))
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.set_market_id(eth_market_id)
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			// batch_id
			3.into(),
			// quantity_locked
			1.into(),
			// market_id
			eth_market_id,
			// oracle_price
			101.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			init_timestamp * 1000,
		));

		let alice_fees_3 = FixedI128::from_float(101.0) * FixedI128::from_float(0.001);
		let bob_fees_3 = FixedI128::from_float(101.0) * FixedI128::from_float(0.001);

		let expected_default_insurance_fund_balance_3 =
			expected_default_insurance_fund_balance_2 + alice_fees_3 + bob_fees_3;

		// balance check
		assert!(
			TradingAccounts::insurance_fund_balance(default_insurance_fund, collateral_id) ==
				expected_default_insurance_fund_balance_3,
			"Invalid balance for default insurance balance"
		);

		/////////////////////////////
		// Default: ETH-USDC Close //
		/////////////////////////////

		// Create orders
		let alice_order = Order::new(207.into(), alice_id)
			.set_price(FixedI128::from_float(101.0))
			.set_side(Side::Sell)
			.set_market_id(eth_market_id)
			.sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(208.into(), bob_id)
			.set_price(FixedI128::from_float(101.0))
			.set_side(Side::Sell)
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.set_market_id(eth_market_id)
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			// batch_id
			4.into(),
			// quantity_locked
			1.into(),
			// market_id
			eth_market_id,
			// oracle_price
			101.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			init_timestamp * 1000,
		));

		let alice_fees_4 = FixedI128::from_float(101.0) * FixedI128::from_float(0.001);
		let bob_fees_4 = FixedI128::from_float(101.0) * FixedI128::from_float(0.001);

		let expected_default_insurance_fund_balance_4 =
			expected_default_insurance_fund_balance_3 + alice_fees_4 + bob_fees_4;

		// balance check
		assert!(
			TradingAccounts::insurance_fund_balance(default_insurance_fund, collateral_id) ==
				expected_default_insurance_fund_balance_4,
			"Invalid balance for default insurance balance"
		);

		/////////////////////////////
		// Isolated: BTC-USDC Open //
		/////////////////////////////

		// Set insurance fund for BTC
		assert_ok!(TradingAccounts::update_fee_split_details(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			btc_market_id,
			btc_insurance_fund,
			btc_fee_split
		));

		// Create orders
		let alice_order = Order::new(209.into(), alice_id)
			.set_price(FixedI128::from_float(1001.0))
			.sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(210.into(), bob_id)
			.set_price(FixedI128::from_float(1001.0))
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			// batch_id
			5.into(),
			// quantity_locked
			1.into(),
			// market_id
			btc_market_id,
			// oracle_price
			1001.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			init_timestamp * 1000,
		));

		let alice_fees_1 = FixedI128::from_float(1001.0) * FixedI128::from_float(0.001);
		let alice_fees_contribution_1 = alice_fees_1 * (FixedI128::one() - btc_fee_split);
		let bob_fees_1 = FixedI128::from_float(1001.0) * FixedI128::from_float(0.001);
		let bob_fees_contribution_1 = bob_fees_1 * (FixedI128::one() - btc_fee_split);

		let expected_btc_insurance_fund_balance_1 =
			alice_fees_contribution_1 + bob_fees_contribution_1;

		// balance check
		assert!(
			TradingAccounts::insurance_fund_balance(btc_insurance_fund, collateral_id) ==
				expected_btc_insurance_fund_balance_1,
			"Invalid balance for btc insurance balance"
		);

		/////////////////////////////
		// Isolated: BTC-USDC Close //
		/////////////////////////////

		// Set insurance fund for BTC
		assert_ok!(TradingAccounts::update_fee_split_details(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			btc_market_id,
			btc_insurance_fund,
			btc_fee_split
		));

		// Create orders
		let alice_order = Order::new(211.into(), alice_id)
			.set_side(Side::Sell)
			.set_price(FixedI128::from_float(1001.0))
			.sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(212.into(), bob_id)
			.set_side(Side::Sell)
			.set_price(FixedI128::from_float(1001.0))
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			// batch_id
			6.into(),
			// quantity_locked
			1.into(),
			// market_id
			btc_market_id,
			// oracle_price
			1001.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			init_timestamp * 1000,
		));

		let alice_fees_2 = FixedI128::from_float(1001.0) * FixedI128::from_float(0.001);
		let alice_fees_contribution_2 = alice_fees_2 * (FixedI128::one() - btc_fee_split);
		let bob_fees_2: FixedI128 = FixedI128::from_float(1001.0) * FixedI128::from_float(0.001);
		let bob_fees_contribution_2 = bob_fees_2 * (FixedI128::one() - btc_fee_split);

		let expected_btc_insurance_fund_balance_2 = expected_btc_insurance_fund_balance_1 +
			alice_fees_contribution_2 +
			bob_fees_contribution_2;

		// balance check
		assert!(
			TradingAccounts::insurance_fund_balance(btc_insurance_fund, collateral_id) ==
				expected_btc_insurance_fund_balance_2,
			"Invalid balance for btc insurance balance"
		);

		////////////////////////////
		// Default: ETH-USDC Open //
		////////////////////////////

		// Create orders
		let alice_order = Order::new(213.into(), alice_id)
			.set_price(FixedI128::from_float(101.0))
			.set_market_id(eth_market_id)
			.sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(214.into(), bob_id)
			.set_price(FixedI128::from_float(101.0))
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.set_market_id(eth_market_id)
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			// batch_id
			7.into(),
			// quantity_locked
			1.into(),
			// market_id
			eth_market_id,
			// oracle_price
			101.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			init_timestamp * 1000,
		));

		let alice_fees_5 = FixedI128::from_float(101.0) * FixedI128::from_float(0.001);
		let bob_fees_5 = FixedI128::from_float(101.0) * FixedI128::from_float(0.001);

		let expected_default_insurance_fund_balance_5 =
			expected_default_insurance_fund_balance_4 + alice_fees_5 + bob_fees_5;

		// balance check
		assert!(
			TradingAccounts::insurance_fund_balance(default_insurance_fund, collateral_id) ==
				expected_default_insurance_fund_balance_5,
			"Invalid balance for default insurance balance"
		);

		/////////////////////////////
		// Default: ETH-USDC Close //
		/////////////////////////////

		// Create orders
		let alice_order = Order::new(215.into(), alice_id)
			.set_side(Side::Sell)
			.set_price(FixedI128::from_float(101.0))
			.set_market_id(eth_market_id)
			.sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(216.into(), bob_id)
			.set_side(Side::Sell)
			.set_price(FixedI128::from_float(101.0))
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.set_market_id(eth_market_id)
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			// batch_id
			8.into(),
			// quantity_locked
			1.into(),
			// market_id
			eth_market_id,
			// oracle_price
			101.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			init_timestamp * 1000,
		));

		let alice_fees_6 = FixedI128::from_float(101.0) * FixedI128::from_float(0.001);
		let bob_fees_6 = FixedI128::from_float(101.0) * FixedI128::from_float(0.001);

		let expected_default_insurance_fund_balance_6 =
			expected_default_insurance_fund_balance_5 + alice_fees_6 + bob_fees_6;

		// balance check
		assert!(
			TradingAccounts::insurance_fund_balance(default_insurance_fund, collateral_id) ==
				expected_default_insurance_fund_balance_6,
			"Invalid balance for default insurance balance"
		);

		/////////////////////////////
		// Isolated: ETH-USDC Open //
		/////////////////////////////

		// Set insurance fund for ETH
		assert_ok!(TradingAccounts::update_fee_split_details(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			eth_market_id,
			eth_insurance_fund,
			eth_fee_split
		));

		// Create orders
		let alice_order = Order::new(217.into(), alice_id)
			.set_price(FixedI128::from_float(101.0))
			.set_market_id(eth_market_id)
			.sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(218.into(), bob_id)
			.set_price(FixedI128::from_float(101.0))
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.set_market_id(eth_market_id)
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			// batch_id
			9.into(),
			// quantity_locked
			1.into(),
			// market_id
			eth_market_id,
			// oracle_price
			101.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			init_timestamp * 1000,
		));

		let alice_fees_1 = FixedI128::from_float(101.0) * FixedI128::from_float(0.001);
		let alice_fees_contribution_1 = alice_fees_1 * (FixedI128::one() - eth_fee_split);
		let bob_fees_1 = FixedI128::from_float(101.0) * FixedI128::from_float(0.001);
		let bob_fees_contribution_1 = bob_fees_1 * (FixedI128::one() - eth_fee_split);

		let expected_eth_insurance_fund_balance_1 =
			alice_fees_contribution_1 + bob_fees_contribution_1;

		// balance check
		assert!(
			TradingAccounts::insurance_fund_balance(eth_insurance_fund, collateral_id) ==
				expected_eth_insurance_fund_balance_1,
			"Invalid balance for eth insurance balance"
		);

		//////////////////////////////
		// Isolated: ETH-USDC Close //
		//////////////////////////////

		// Set insurance fund for ETH
		assert_ok!(TradingAccounts::update_fee_split_details(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			eth_market_id,
			eth_insurance_fund,
			eth_fee_split
		));

		// Create orders
		let alice_order = Order::new(219.into(), alice_id)
			.set_price(FixedI128::from_float(101.0))
			.set_market_id(eth_market_id)
			.sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(220.into(), bob_id)
			.set_price(FixedI128::from_float(101.0))
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.set_market_id(eth_market_id)
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			// batch_id
			10.into(),
			// quantity_locked
			1.into(),
			// market_id
			eth_market_id,
			// oracle_price
			101.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			init_timestamp * 1000,
		));

		let alice_fees_2 = FixedI128::from_float(101.0) * FixedI128::from_float(0.001);
		let alice_fees_contribution_2 = alice_fees_2 * (FixedI128::one() - eth_fee_split);
		let bob_fees_2 = FixedI128::from_float(101.0) * FixedI128::from_float(0.001);
		let bob_fees_contribution_2 = bob_fees_2 * (FixedI128::one() - eth_fee_split);

		let expected_eth_insurance_fund_balance_2 = expected_eth_insurance_fund_balance_1 +
			alice_fees_contribution_2 +
			bob_fees_contribution_2;

		// balance check
		assert!(
			TradingAccounts::insurance_fund_balance(eth_insurance_fund, collateral_id) ==
				expected_eth_insurance_fund_balance_2,
			"Invalid balance for eth insurance balance"
		);
	});
}

#[test]
fn test_fee_share_2() {
	let mut env = setup();

	// test env
	// Generate account_ids
	let alice_id: U256 = get_trading_account_id(alice());
	let bob_id: U256 = get_trading_account_id(bob());
	let charlie_account_address = charlie().account_address;
	let bob_account_address = bob().account_address;

	let market_id = btc_usdc().market.id;
	let collateral_id = usdc().asset.id;

	let init_timestamp: u64 = 1699940367;
	let one_day: u64 = 24 * 60 * 60;

	let initial_balance = FixedI128::from_float(10000.0);

	env.execute_with(|| {
		// Add referral data
		assert_ok!(TradingAccounts::add_referral(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			alice().account_address,
			ReferralDetails {
				master_account_address: bob().account_address,
				fee_discount: FixedI128::from_float(0.1),
			},
			U256::from(123),
		));

		assert_ok!(TradingAccounts::add_referral(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			bob().account_address,
			ReferralDetails {
				master_account_address: charlie().account_address,
				fee_discount: FixedI128::from_float(0.1),
			},
			U256::from(123),
		));

		// Set Charlie's level as 1
		assert_ok!(TradingAccounts::update_master_account_level(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			charlie_account_address,
			1
		));

		// Add fee data
		assert_ok!(TradingFees::update_base_fees(
			RuntimeOrigin::root(),
			collateral_id,
			get_usdc_aggregate_fees()
		));

		// Add fee_share_data
		assert_ok!(TradingFees::update_fee_share(
			RuntimeOrigin::root(),
			collateral_id,
			get_usdc_fee_shares()
		));

		////////////////////
		// Day 1: Batch 1 //
		////////////////////

		// Create orders
		let alice_order = Order::new(201.into(), alice_id)
			.set_price(FixedI128::from_float(1001.0))
			.sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(202.into(), bob_id)
			.set_price(FixedI128::from_float(1001.0))
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			// batch_id
			2.into(),
			// quantity_locked
			1.into(),
			// market_id
			market_id,
			// oracle_price
			1001.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			init_timestamp * 1000,
		));

		let bob_30day_master_volume_1 =
			TradingAccounts::get_30day_master_volume(bob_account_address, market_id).unwrap();
		assert_eq!(bob_30day_master_volume_1, FixedI128::zero(), "Error in 30 day volume");

		let bob_master_fee_share_1 =
			TradingAccounts::master_account_fee_share(bob_account_address, collateral_id);
		assert!(bob_master_fee_share_1 == FixedI128::zero(), "wrong master fee share");

		let chalie_30day_master_volume_1 =
			TradingAccounts::get_30day_master_volume(charlie_account_address, market_id).unwrap();
		assert_eq!(chalie_30day_master_volume_1, FixedI128::zero(), "Error in 30 day volume");

		let charlie_master_fee_share_1 =
			TradingAccounts::master_account_fee_share(charlie_account_address, collateral_id);
		assert!(charlie_master_fee_share_1 == FixedI128::zero(), "wrong master fee share");

		// Alice's current tier is 1
		// fee_rate = 0.001 (0.1%)
		// fee_discount = 0.1 (10%)
		// effective_fee_rate = 0.001 * (1-0.1)
		let alice_balance_1 = TradingAccounts::balances(alice_id, collateral_id);
		assert!(
			alice_balance_1 ==
				initial_balance -
					(FixedI128::from_float(1001.0) * FixedI128::from_float(0.0009)),
			"Invalid fee rate for Alice day 1 batch 1"
		);

		// Bob's current tier is 1
		// fee_rate = 0.001 (0.1%)
		// fee_discount = 0.1 (10%)
		// effective_fee_rate = 0.001 * (1-0.1)
		let bob_balance_1 = TradingAccounts::balances(bob_id, collateral_id);
		assert!(
			bob_balance_1 ==
				initial_balance -
					(FixedI128::from_float(1001.0) * FixedI128::from_float(0.0009)),
			"Invalid fee rate for Bob day 1 batch 1"
		);

		////////////////////
		// Day 1: Batch 2 //
		////////////////////

		// Create orders
		let alice_order = Order::new(203.into(), alice_id)
			.set_price(FixedI128::from_float(1001.0))
			.sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(204.into(), bob_id)
			.set_price(FixedI128::from_float(1001.0))
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			// batch_id
			3.into(),
			// quantity_locked
			1.into(),
			// market_id
			market_id,
			// oracle_price
			1001.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			init_timestamp * 1000,
		));

		let bob_30day_master_volume_2 =
			TradingAccounts::get_30day_master_volume(bob_account_address, market_id).unwrap();
		assert_eq!(bob_30day_master_volume_2, FixedI128::zero(), "Error in 30 day volume");

		let bob_master_fee_share_2 =
			TradingAccounts::master_account_fee_share(bob_account_address, collateral_id);
		assert!(bob_master_fee_share_2 == FixedI128::zero(), "wrong master fee share");

		let chalie_30day_master_volume_2 =
			TradingAccounts::get_30day_master_volume(charlie_account_address, market_id).unwrap();
		assert_eq!(chalie_30day_master_volume_2, FixedI128::zero(), "Error in 30 day volume");

		let charlie_master_fee_share_2 =
			TradingAccounts::master_account_fee_share(charlie_account_address, collateral_id);
		assert!(charlie_master_fee_share_2 == FixedI128::zero(), "wrong master fee share");

		// Alice's current tier is 1
		// fee_rate = 0.001 (0.1%)
		// fee_discount = 0.1 (10%)
		// effective_fee_rate = 0.001 * (1-0.1)
		let alice_balance_2 = TradingAccounts::balances(alice_id, collateral_id);
		assert!(
			alice_balance_2 ==
				alice_balance_1 -
					(FixedI128::from_float(1001.0) * FixedI128::from_float(0.0009)),
			"Invalid fee rate for Alice day 1 batch 2"
		);

		// Bob's current tier is 1
		// fee_rate = 0.001 (0.1%)
		// fee_discount = 0.1 (10%)
		// effective_fee_rate = 0.001 * (1-0.1)
		let bob_balance_2 = TradingAccounts::balances(bob_id, collateral_id);
		assert!(
			bob_balance_2 ==
				bob_balance_1 - (FixedI128::from_float(1001.0) * FixedI128::from_float(0.0009)),
			"Invalid fee rate for Bob day 1 batch 2"
		);

		////////////////////
		// Day 2: Batch 1 //
		////////////////////

		// next trade on next day i.e. day 2
		Timestamp::set_timestamp((init_timestamp + one_day + 1) * 1000);

		// Create orders
		let alice_order = Order::new(205.into(), alice_id)
			.set_price(FixedI128::from_float(1001.0))
			.sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(206.into(), bob_id)
			.set_price(FixedI128::from_float(1001.0))
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			// batch_id
			4.into(),
			// quantity_locked
			1.into(),
			// market_id
			market_id,
			// oracle_price
			1001.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			(init_timestamp + one_day) * 1000,
		));

		let bob_30day_master_volume_3 =
			TradingAccounts::get_30day_master_volume(bob_account_address, market_id).unwrap();
		assert_eq!(bob_30day_master_volume_3, 2002.into(), "Error in 30 day volume");

		let chalie_30day_master_volume_3 =
			TradingAccounts::get_30day_master_volume(charlie_account_address, market_id).unwrap();
		assert_eq!(chalie_30day_master_volume_3, 2002.into(), "Error in 30 day volume");

		// Alice's current tier is 2
		// fee_rate = 0.00050 (0.05%)
		// fee_discount = 0.1 (10%)
		// effective_fee_rate = 0.00050 * (1-0.1)
		let alice_balance_3 = TradingAccounts::balances(alice_id, collateral_id);
		let alice_fee_3 = FixedI128::from_float(1001.0) * FixedI128::from_float(0.00045);
		assert!(
			alice_balance_3 == alice_balance_2 - alice_fee_3,
			"Invalid fee rate for Alice day 2 batch 1"
		);

		// Bob's current tier is 2
		// fee_rate = 0.0008 (0.08%)
		// fee_discount = 0.1 (10%)
		// effective_fee_rate = 0.0008 * (1-0.1)
		let bob_balance_3 = TradingAccounts::balances(bob_id, collateral_id);
		let bob_fee_3 = FixedI128::from_float(1001.0) * FixedI128::from_float(0.00072);
		assert!(
			bob_balance_3 == bob_balance_2 - bob_fee_3,
			"Invalid fee rate for Bob day 2 batch 1"
		);

		// Bob's master's current tier is 3
		// fee_share_rate = 0.08 (8%)
		let bob_master_fee_share_3 =
			TradingAccounts::master_account_fee_share(bob_account_address, collateral_id);
		let expected_bob_master_fee_share_3 = alice_fee_3 * FixedI128::from_float(0.08);
		assert!(
			bob_master_fee_share_3 == expected_bob_master_fee_share_3.round_to_precision(6),
			"wrong master fee share"
		);

		// Charlie's master's current tier is 3
		// And level is 1
		let charlie_master_fee_share_3 =
			TradingAccounts::master_account_fee_share(charlie_account_address, collateral_id);
		let expected_charlie_master_fee_share_3 = bob_fee_3 * FixedI128::from_float(0.5);
		assert!(
			charlie_master_fee_share_3 == expected_charlie_master_fee_share_3.round_to_precision(6),
			"wrong master fee share"
		);

		// Check MasterFeeShareUpdated event
		assert_has_events(vec![
			Event::MasterFeeShareUpdated {
				master_account_address: bob_account_address,
				referral_account_address: alice().account_address,
				order_volume: 1001.into(),
				collateral_id,
				fee_share: (alice_fee_3 * FixedI128::from_float(0.08)).round_to_precision(6),
			}
			.into(),
			Event::MasterFeeShareUpdated {
				master_account_address: charlie_account_address,
				referral_account_address: bob().account_address,
				order_volume: 1001.into(),
				collateral_id,
				fee_share: (bob_fee_3 * FixedI128::from_float(0.5)).round_to_precision(6),
			}
			.into(),
		]);

		////////////////////
		// Day 2: Batch 2 //
		////////////////////

		// Create orders
		let alice_order = Order::new(207.into(), alice_id)
			.set_price(FixedI128::from_float(1001.0))
			.sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(208.into(), bob_id)
			.set_price(FixedI128::from_float(1001.0))
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			// batch_id
			5.into(),
			// quantity_locked
			1.into(),
			// market_id
			market_id,
			// oracle_price
			1001.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			(init_timestamp + one_day) * 1000,
		));

		let bob_30day_master_volume_4 =
			TradingAccounts::get_30day_master_volume(bob_account_address, market_id).unwrap();
		assert_eq!(bob_30day_master_volume_4, 2002.into(), "Error in 30 day volume");

		let chalie_30day_master_volume_4 =
			TradingAccounts::get_30day_master_volume(charlie_account_address, market_id).unwrap();
		assert_eq!(chalie_30day_master_volume_4, 2002.into(), "Error in 30 day volume");

		// Alice's current tier is 2
		// fee_rate = 0.00050 (0.05%)
		// fee_discount = 0.1 (10%)
		// effective_fee_rate = 0.00050 * (1-0.1)
		let alice_balance_4 = TradingAccounts::balances(alice_id, collateral_id);
		let alice_fee_4 = FixedI128::from_float(1001.0) * FixedI128::from_float(0.00045);
		assert!(
			alice_balance_4 == alice_balance_3 - alice_fee_4,
			"Invalid fee rate for Alice day 2 batch 2"
		);

		// Bob's current tier is 2
		// fee_rate = 0.0008 (0.08%)
		// fee_discount = 0.1 (10%)
		// effective_fee_rate = 0.0008 * (1-0.1)
		let bob_balance_4 = TradingAccounts::balances(bob_id, collateral_id);
		let bob_fee_4 = FixedI128::from_float(1001.0) * FixedI128::from_float(0.00072);
		assert!(
			bob_balance_4 == bob_balance_3 - bob_fee_4,
			"Invalid fee rate for Bob day 2 batch 2"
		);

		// Bob's master's current tier is 3
		// fee_share_rate = 0.08 (8%)
		let bob_master_fee_share_4 =
			TradingAccounts::master_account_fee_share(bob_account_address, collateral_id);
		let expected_bob_master_fee_share_4 = alice_fee_4 * FixedI128::from_float(0.08);
		assert!(
			bob_master_fee_share_4 ==
				(bob_master_fee_share_3 + expected_bob_master_fee_share_4).round_to_precision(6),
			"wrong master fee share"
		);

		// Charlie's master's current tier is 3
		// And level is 1
		let charlie_master_fee_share_4 =
			TradingAccounts::master_account_fee_share(charlie_account_address, collateral_id);
		let expected_charlie_master_fee_share_4 = bob_fee_4 * FixedI128::from_float(0.5);
		assert!(
			charlie_master_fee_share_4 ==
				(charlie_master_fee_share_3 + expected_charlie_master_fee_share_4)
					.round_to_precision(6),
			"wrong master fee share"
		);

		// Check MasterFeeShareUpdated event
		assert_has_events(vec![
			Event::MasterFeeShareUpdated {
				master_account_address: bob_account_address,
				referral_account_address: alice().account_address,
				order_volume: 1001.into(),
				collateral_id,
				fee_share: (alice_fee_4 * FixedI128::from_float(0.08)).round_to_precision(6),
			}
			.into(),
			Event::MasterFeeShareUpdated {
				master_account_address: charlie_account_address,
				referral_account_address: bob().account_address,
				order_volume: 1001.into(),
				collateral_id,
				fee_share: (bob_fee_4 * FixedI128::from_float(0.5)).round_to_precision(6),
			}
			.into(),
		]);

		// ////////////////////
		// // Day 3: Batch 1 //
		// ////////////////////

		// next trade on next day i.e. day 2
		Timestamp::set_timestamp((init_timestamp + one_day * 2 + 1) * 1000);

		// Create orders
		let alice_order = Order::new(209.into(), alice_id)
			.set_price(FixedI128::from_float(1001.0))
			.sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(210.into(), bob_id)
			.set_price(FixedI128::from_float(1001.0))
			.set_direction(Direction::Short)
			.set_order_type(OrderType::Market)
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			// batch_id
			6.into(),
			// quantity_locked
			1.into(),
			// market_id
			market_id,
			// oracle_price
			1001.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			(init_timestamp + one_day) * 1000,
		));

		let bob_30day_master_volume_4 =
			TradingAccounts::get_30day_master_volume(charlie_account_address, market_id).unwrap();
		assert_eq!(bob_30day_master_volume_4, 4004.into(), "Error in 30 day volume");

		let charlie_30day_master_volume_4 =
			TradingAccounts::get_30day_master_volume(charlie_account_address, market_id).unwrap();
		assert_eq!(charlie_30day_master_volume_4, 4004.into(), "Error in 30 day volume");

		// Alice's current tier is 3
		// fee_rate = 0.00020 (0.02%)
		// fee_discount = 0.1 (10%)
		// effective_fee_rate = 0.00020 * (1-0.1)
		let alice_balance_5 = TradingAccounts::balances(alice_id, collateral_id);
		let alice_fee_5 = FixedI128::from_float(1001.0) * FixedI128::from_float(0.00018);
		assert!(
			alice_balance_5 == alice_balance_4 - alice_fee_5,
			"Invalid fee rate for Alice day 3 batch 1"
		);

		// Bob's current tier is 3
		// fee_rate = 0.0005 (0.05%)
		// fee_discount = 0.1 (10%)
		// effective_fee_rate = 0.0005 * (1-0.1)
		let bob_balance_5 = TradingAccounts::balances(bob_id, collateral_id);
		let bob_fee_5 = FixedI128::from_float(1001.0) * FixedI128::from_float(0.00045);
		assert!(
			bob_balance_5 == bob_balance_4 - bob_fee_5,
			"Invalid fee rate for Bob day 3 batch 1"
		);

		// Bob's master's current tier is 3
		// fee_share_rate = 0.08 (8%)
		let bob_master_fee_share_5 =
			TradingAccounts::master_account_fee_share(bob_account_address, collateral_id);
		let expected_bob_master_fee_share_5 = alice_fee_5 * FixedI128::from_float(0.08);
		assert!(
			bob_master_fee_share_5 ==
				(bob_master_fee_share_4 + expected_bob_master_fee_share_5).round_to_precision(6),
			"wrong master fee share"
		);

		// Charlie's master's current tier is 3
		// And level is 1
		let charlie_master_fee_share_5 =
			TradingAccounts::master_account_fee_share(charlie_account_address, collateral_id);
		let expected_charlie_master_fee_share_5 = bob_fee_5 * FixedI128::from_float(0.5);
		assert!(
			charlie_master_fee_share_5 ==
				(charlie_master_fee_share_4 + expected_charlie_master_fee_share_5)
					.round_to_precision(6),
			"wrong master fee share"
		);

		// Check MasterFeeShareUpdated event
		assert_has_events(vec![
			Event::MasterFeeShareUpdated {
				master_account_address: bob_account_address,
				referral_account_address: alice().account_address,
				order_volume: 1001.into(),
				collateral_id,
				fee_share: (alice_fee_5 * FixedI128::from_float(0.08)).round_to_precision(6),
			}
			.into(),
			Event::MasterFeeShareUpdated {
				master_account_address: charlie_account_address,
				referral_account_address: bob().account_address,
				order_volume: 1001.into(),
				collateral_id,
				fee_share: (bob_fee_5 * FixedI128::from_float(0.5)).round_to_precision(6),
			}
			.into(),
		]);

		// Emit FeeShareTransfer for Bob and Charlie
		assert_ok!(TradingAccounts::pay_fee_shares(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			vec![
				FeeSharesInput {
					master_account_address: bob_account_address,
					collateral_id,
					amount: bob_master_fee_share_5
				},
				FeeSharesInput {
					master_account_address: charlie_account_address,
					collateral_id,
					amount: charlie_master_fee_share_5
				},
			]
		));

		assert_has_events(vec![
			TradingAccountEvent::FeeShareTransfer {
				master_account_address: charlie_account_address,
				collateral_id,
				amount: charlie_master_fee_share_5,
				block_number: 1,
			}
			.into(),
			TradingAccountEvent::FeeShareTransfer {
				master_account_address: bob_account_address,
				collateral_id,
				amount: bob_master_fee_share_5,
				block_number: 1,
			}
			.into(),
		]);

		assert!(
			TradingAccounts::master_account_fee_share(charlie_account_address, collateral_id) ==
				FixedI128::zero(),
			"wrong master fee share"
		);

		assert!(
			TradingAccounts::master_account_fee_share(bob_account_address, collateral_id) ==
				FixedI128::zero(),
			"wrong master fee share"
		);
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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

		assert_ok!(Trading::perform_cleanup(RuntimeOrigin::signed(
			sp_core::sr25519::Public::from_raw([1u8; 32])
		)));

		// Check for portion executed
		let order1 = Trading::order_state(U256::from(201));
		assert_eq!(order1.0, FixedI128::zero());
		let order2 = Trading::order_state(U256::from(202));
		assert_eq!(order2.0, FixedI128::zero());
		let order3 = Trading::order_state(U256::from(203));
		assert_eq!(order3.0, FixedI128::one());
		let order4 = Trading::order_state(U256::from(204));
		assert_eq!(order4.0, FixedI128::one());

		// Check for order hash
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
		assert_eq!(1699940398, start_timestamp.unwrap());

		let timestamp1 = Trading::orders(1699940278);
		assert_eq!(false, timestamp1.is_some());
		let timestamp2 = Trading::orders(1702359500);
		assert_eq!(vec![U256::from(203)], timestamp2.unwrap());
		let timestamp3 = Trading::orders(1702359400);
		assert_eq!(vec![U256::from(204)], timestamp3.unwrap());

		let timestamp1 = Trading::batches(1699940360);
		assert_eq!(true, timestamp1.is_none());
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			Event::OrderError { order_id: U256::from(301), account_id: alice_id, error_code: 501 }
				.into(),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			Event::OrderError { order_id: U256::from(201), account_id: alice_id, error_code: 506 }
				.into(),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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
	});
}

#[test]
fn test_fee_rates() {
	// Get a test environment
	let mut env = setup();

	// User accounts
	// Generate account_ids
	let alice_id: U256 = get_trading_account_id(alice());
	let bob_id: U256 = get_trading_account_id(bob());

	// market id
	let market_id = btc_usdc().market.id;
	let collateral_id = usdc().asset.id;

	// Initial timestamp
	let init_timestamp = 1698796800;
	let one_day = 60 * 60 * 24;

	// Get the fees
	let (fee_details_maker, fee_details_taker) = setup_fee();

	env.execute_with(|| {
		// Set the init timestamp
		Timestamp::set_timestamp(init_timestamp * 1000);
		// Dispatch a signed extrinsic.
		assert_ok!(TradingFees::update_base_fees(
			RuntimeOrigin::root(),
			collateral_id,
			BaseFeeAggregate {
				maker_buy: fee_details_maker.clone(),
				maker_sell: fee_details_maker.clone(),
				taker_buy: fee_details_taker.clone(),
				taker_sell: fee_details_taker.clone(),
			}
		));

		// Check for get_fee function
		let common_fee_rates = FeeRates {
			maker_buy: FixedI128::from_inner(20000000000000000),
			maker_sell: FixedI128::from_inner(20000000000000000),
			taker_buy: FixedI128::from_inner(50000000000000000),
			taker_sell: FixedI128::from_inner(50000000000000000),
		};
		let alice_fee = Trading::get_fee(alice_id, market_id);
		let bob_fee = Trading::get_fee(bob_id, market_id);

		assert_eq!(alice_fee, (common_fee_rates, init_timestamp + one_day));
		assert_eq!(bob_fee, (common_fee_rates, init_timestamp + one_day));
	});
}

#[test]
fn test_discounted_fee_rate_for_referral() {
	// Get a test environment
	let mut env = setup();

	// User accounts
	// Generate account_ids
	let alice_id: U256 = get_trading_account_id(alice());
	let bob_id: U256 = get_trading_account_id(bob());

	// market id
	let market_id = btc_usdc().market.id;
	let collateral_id = usdc().asset.id;

	env.execute_with(|| {
		// Get the fees
		let (fee_details_maker, fee_details_taker) = setup_fee();
		assert_ok!(TradingFees::update_base_fees(
			RuntimeOrigin::root(),
			collateral_id,
			BaseFeeAggregate {
				maker_buy: fee_details_maker.clone(),
				maker_sell: vec![BaseFee { volume: FixedI128::zero(), fee: FixedI128::zero() }],
				taker_buy: fee_details_taker.clone(),
				taker_sell: vec![BaseFee { volume: FixedI128::zero(), fee: FixedI128::zero() }],
			}
		));

		// Get Trading Account details
		let alice_account_details = TradingAccounts::get_account(&alice_id).unwrap();
		let bob_account_details = TradingAccounts::get_account(&bob_id).unwrap();
		let referral_details = ReferralDetails {
			master_account_address: alice_account_details.account_address,
			fee_discount: FixedI128::from_inner(200000000000000000),
		};
		// Add referral to the system
		assert_ok!(TradingAccounts::add_referral(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
			bob_account_details.account_address,
			referral_details,
			U256::one()
		));

		let event_record = System::events();
		println!("Events: {:?}", event_record);

		let master_account =
			TradingAccounts::master_account(bob_account_details.account_address).unwrap();
		assert_eq!(master_account.master_account_address, referral_details.master_account_address);
		assert_eq!(master_account.fee_discount, FixedI128::from_inner(200000000000000000));

		let referral_monetary_address =
			TradingAccounts::referral_accounts((alice_account_details.account_address, 0));
		assert_eq!(referral_monetary_address, bob_account_details.account_address);

		let referral_count =
			TradingAccounts::referrals_count(alice_account_details.account_address);
		assert_eq!(referral_count, 1_u64);

		// Create open orders
		let alice_open_order =
			Order::new(U256::from(201), alice_id).sign_order(get_private_key(alice().pub_key));
		let bob_open_order = Order::new(U256::from(202), bob_id)
			.set_order_type(OrderType::Market)
			.set_direction(Direction::Short)
			.sign_order(get_private_key(bob().pub_key));

		// Execute the trade
		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(sp_core::sr25519::Public::from_raw([1u8; 32])),
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

		// Check for balances
		assert_eq!(TradingAccounts::balances(alice_id, collateral_id), 9998.into());
		assert_eq!(TradingAccounts::balances(bob_id, collateral_id), 9996.into());
	});
}
