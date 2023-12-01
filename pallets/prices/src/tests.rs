use crate::mock::*;
use frame_support::assert_ok;
use pallet_support::{
	test_helpers::{
		accounts_helper::{alice, bob, charlie, dave, get_private_key, get_trading_account_id},
		asset_helper::{btc, eth, link, usdc},
		market_helper::{btc_usdc, eth_usdc, link_usdc},
	},
	types::{Direction, MultiplePrices, Order, OrderType, Position, Side},
};
use primitive_types::U256;
use sp_arithmetic::fixed_point::FixedI128;

// declare test_helper module
pub mod test_helper;
use sp_io::transaction_index::index;
use test_helper::*;

fn setup() -> sp_io::TestExternalities {
	// Create a new test environment
	let mut test_env = new_test_ext();

	let assets = vec![eth(), usdc(), link()];

	// Set the signers using admin account
	test_env.execute_with(|| {
		assert_ok!(Timestamp::set(None.into(), 1699940367000));
		assert_ok!(AssetModule::replace_all_assets(RuntimeOrigin::signed(1), assets));

		// Go past genesis block so events get deposited
		System::set_block_number(1);
	});

	test_env.into()
}

fn setup_trading() -> sp_io::TestExternalities {
	// Create a new test environment
	let mut env = new_test_ext();

	// Set the block number in the environment
	env.execute_with(|| {
		// Set the block number
		System::set_block_number(1);
		assert_ok!(Timestamp::set(None.into(), 1699940278000));

		// Set the assets in the system
		assert_ok!(AssetModule::replace_all_assets(
			RuntimeOrigin::signed(1),
			vec![eth(), usdc(), link(), btc()]
		));
		assert_ok!(MarketModule::replace_all_markets(
			RuntimeOrigin::signed(1),
			vec![btc_usdc(), link_usdc()]
		));

		// Add accounts to the system
		assert_ok!(TradingAccounts::add_accounts(
			RuntimeOrigin::signed(1),
			vec![alice(), bob(), charlie(), dave()]
		));

		// Set ABR interval as 8 hours
		assert_ok!(PricesModule::set_abr_interval(RuntimeOrigin::signed(1), 28800));

		// Set Base ABR as 0.000025
		assert_ok!(PricesModule::set_base_abr(
			RuntimeOrigin::signed(1),
			FixedI128::from_inner(25000000000000)
		));

		// Set Bollinger width as 1.5
		assert_ok!(PricesModule::set_bollinger_width(
			RuntimeOrigin::signed(1),
			FixedI128::from_inner(1500000000000000000)
		));

		// Set no.of users per batch
		assert_ok!(PricesModule::set_no_of_users_per_batch(RuntimeOrigin::signed(1), 10));
	});
	env
}

fn set_prices(market_id: u128) {
	let (mark_prices, index_prices) = mock_prices::get_btc_usdc_prices_1();
	let mut interval: u64 = 1699940278000;
	for i in 0..mark_prices.len() {
		let mut prices: Vec<MultiplePrices> = Vec::new();
		let price: MultiplePrices =
			MultiplePrices { market_id, index_price: index_prices[i], mark_price: mark_prices[i] };
		prices.push(price);
		assert_ok!(PricesModule::update_prices(RuntimeOrigin::signed(1), prices, interval));
		interval += 60000;
	}
}

#[test]
fn test_update_prices() {
	// Get a test environment
	let mut env = setup();

	// test variables
	let market1 = eth_usdc();
	let market2 = link_usdc();

	env.execute_with(|| {
		// Dispatch a signed extrinsic.
		let markets = vec![eth_usdc(), link_usdc()];
		assert_ok!(MarketModule::replace_all_markets(RuntimeOrigin::signed(1), markets));
		let mut prices: Vec<MultiplePrices> = Vec::new();
		let mark_price1 = MultiplePrices {
			market_id: market1.market.id,
			index_price: 102.into(),
			mark_price: 100.into(),
		};
		let mark_price2: MultiplePrices = MultiplePrices {
			market_id: market2.market.id,
			index_price: 199.into(),
			mark_price: 200.into(),
		};
		prices.push(mark_price1);
		prices.push(mark_price2);
		assert_ok!(PricesModule::update_prices(
			RuntimeOrigin::signed(1),
			prices.clone(),
			1699940367000
		));

		let price = PricesModule::current_price(market1.market.id);
		assert_eq!(FixedI128::from_u32(100), price.mark_price);
		assert_eq!(FixedI128::from_u32(102), price.index_price);

		let price = PricesModule::current_price(market2.market.id);
		assert_eq!(FixedI128::from_u32(200), price.mark_price);
		assert_eq!(FixedI128::from_u32(199), price.index_price);
	});
}

#[test]
fn test_historical_prices() {
	// Get a test environment
	let mut env = setup();

	// test variables
	let market1 = eth_usdc();
	let market2 = link_usdc();

	env.execute_with(|| {
		Timestamp::set_timestamp(1702359600000);

		let markets = vec![eth_usdc(), link_usdc()];
		assert_ok!(MarketModule::replace_all_markets(RuntimeOrigin::signed(1), markets));

		let mut prices: Vec<MultiplePrices> = Vec::new();
		let mark_price1 = MultiplePrices {
			market_id: market1.market.id,
			index_price: 102.into(),
			mark_price: 100.into(),
		};
		let mark_price2 = MultiplePrices {
			market_id: market2.market.id,
			index_price: 199.into(),
			mark_price: 200.into(),
		};
		prices.push(mark_price1);
		prices.push(mark_price2);
		assert_ok!(PricesModule::update_prices(
			RuntimeOrigin::signed(1),
			prices.clone(),
			1702359600000
		));

		let price = PricesModule::current_price(market1.market.id);
		assert_eq!(FixedI128::from_u32(100), price.mark_price);
		assert_eq!(FixedI128::from_u32(102), price.index_price);

		let price = PricesModule::current_price(market2.market.id);
		assert_eq!(FixedI128::from_u32(200), price.mark_price);
		assert_eq!(FixedI128::from_u32(199), price.index_price);

		let historical_price = PricesModule::historical_price(1702359600, market1.market.id);
		assert_eq!(FixedI128::from_u32(100), historical_price.mark_price);
		assert_eq!(FixedI128::from_u32(102), historical_price.index_price);

		let historical_price = PricesModule::historical_price(1702359600, market2.market.id);
		assert_eq!(FixedI128::from_u32(200), historical_price.mark_price);
		assert_eq!(FixedI128::from_u32(199), historical_price.index_price);

		let mut prices: Vec<MultiplePrices> = Vec::new();
		let mark_price1 = MultiplePrices {
			market_id: market1.market.id,
			index_price: 110.into(),
			mark_price: 109.into(),
		};
		let mark_price2 = MultiplePrices {
			market_id: market2.market.id,
			index_price: 190.into(),
			mark_price: 192.into(),
		};
		prices.push(mark_price1);
		prices.push(mark_price2);
		assert_ok!(PricesModule::update_prices(
			RuntimeOrigin::signed(1),
			prices.clone(),
			1702359620000
		));

		let price = PricesModule::current_price(market1.market.id);
		assert_eq!(FixedI128::from_u32(109), price.mark_price);
		assert_eq!(FixedI128::from_u32(110), price.index_price);

		let price = PricesModule::current_price(market2.market.id);
		assert_eq!(FixedI128::from_u32(192), price.mark_price);
		assert_eq!(FixedI128::from_u32(190), price.index_price);

		let historical_price = PricesModule::historical_price(1702359620, market1.market.id);
		assert_eq!(FixedI128::from_u32(109), historical_price.mark_price);
		assert_eq!(FixedI128::from_u32(110), historical_price.index_price);

		let historical_price = PricesModule::historical_price(1702359620, market2.market.id);
		assert_eq!(FixedI128::from_u32(192), historical_price.mark_price);
		assert_eq!(FixedI128::from_u32(190), historical_price.index_price);

		let mut prices: Vec<MultiplePrices> = Vec::new();
		let mark_price1 = MultiplePrices {
			market_id: market1.market.id,
			index_price: 150.into(),
			mark_price: 151.into(),
		};
		let mark_price2 = MultiplePrices {
			market_id: market2.market.id,
			index_price: 230.into(),
			mark_price: 229.into(),
		};
		prices.push(mark_price1);
		prices.push(mark_price2);
		assert_ok!(PricesModule::update_prices(
			RuntimeOrigin::signed(1),
			prices.clone(),
			1702359661000
		));

		let price = PricesModule::current_price(market1.market.id);
		assert_eq!(FixedI128::from_u32(151), price.mark_price);
		assert_eq!(FixedI128::from_u32(150), price.index_price);

		let price = PricesModule::current_price(market2.market.id);
		assert_eq!(FixedI128::from_u32(229), price.mark_price);
		assert_eq!(FixedI128::from_u32(230), price.index_price);

		let historical_price = PricesModule::historical_price(1702359661, market1.market.id);
		assert_eq!(FixedI128::from_u32(151), historical_price.mark_price);
		assert_eq!(FixedI128::from_u32(150), historical_price.index_price);

		let historical_price = PricesModule::historical_price(1702359661, market2.market.id);
		assert_eq!(FixedI128::from_u32(229), historical_price.mark_price);
		assert_eq!(FixedI128::from_u32(230), historical_price.index_price);
	});
}

#[test]
fn test_abr_calculation_eth_usdc_1() {
	// Get a test environment
	let mut env = setup();

	env.execute_with(|| {
		let (mark_prices, index_prices) = mock_prices::get_eth_usdc_prices_1();
		let result = PricesModule::calculate_abr(
			mark_prices,
			index_prices,
			convert_to_fixed(0.000025_f64),
			convert_to_fixed(1.5),
			8_usize,
		);
		compare_with_threshold(
			result.0,
			convert_to_fixed(4.577354961709272e-05),
			convert_to_fixed(1e-10),
		);
	});
}

#[test]
fn test_abr_calculation_eth_usdc_2() {
	// Get a test environment
	let mut env = setup();

	env.execute_with(|| {
		let (mark_prices, index_prices) = mock_prices::get_eth_usdc_prices_2();
		let result = PricesModule::calculate_abr(
			mark_prices,
			index_prices,
			convert_to_fixed(0.000025_f64),
			convert_to_fixed(1.5),
			8_usize,
		);
		compare_with_threshold(
			result.0,
			convert_to_fixed(4.492383850355448e-05),
			convert_to_fixed(1e-10),
		);
	});
}

#[test]
fn test_abr_calculation_btc_usdc_1() {
	// Get a test environment
	let mut env = setup();

	env.execute_with(|| {
		let (mark_prices, index_prices) = mock_prices::get_btc_usdc_prices_1();
		let result = PricesModule::calculate_abr(
			mark_prices,
			index_prices,
			convert_to_fixed(0.000025_f64),
			convert_to_fixed(1.5),
			8_usize,
		);
		compare_with_threshold(
			result.0,
			convert_to_fixed(8.83808701975073e-05),
			convert_to_fixed(1e-10),
		);
	});
}

#[test]
fn test_abr_calculation_btc_usdc_2() {
	// Get a test environment
	let mut env = setup();

	env.execute_with(|| {
		let (mark_prices, index_prices) = mock_prices::get_btc_usdc_prices_2();
		let result = PricesModule::calculate_abr(
			mark_prices,
			index_prices,
			convert_to_fixed(0.000025_f64),
			convert_to_fixed(1.5),
			8_usize,
		);
		compare_with_threshold(
			result.0,
			convert_to_fixed(0.0011603379908277198),
			convert_to_fixed(1e-10),
		);
	});
}

#[test]
fn test_abr_calculation_btc_usdt_1() {
	// Get a test environment
	let mut env = setup();

	env.execute_with(|| {
		let (mark_prices, index_prices) = mock_prices::get_btc_usdt_prices_1();
		let result = PricesModule::calculate_abr(
			mark_prices,
			index_prices,
			convert_to_fixed(0.000025_f64),
			convert_to_fixed(1.5),
			8_usize,
		);
		compare_with_threshold(
			result.0,
			convert_to_fixed(-0.0002730150595400045),
			convert_to_fixed(1e-10),
		);
	});
}

#[test]
fn test_abr_calculation_btc_usdt_2() {
	// Get a test environment
	let mut env = setup();

	env.execute_with(|| {
		let (mark_prices, index_prices) = mock_prices::get_btc_usdt_prices_2();
		let result = PricesModule::calculate_abr(
			mark_prices,
			index_prices,
			convert_to_fixed(0.000025_f64),
			convert_to_fixed(1.5),
			8_usize,
		);
		compare_with_threshold(
			result.0,
			convert_to_fixed(-0.0009117240376668166),
			convert_to_fixed(1e-10),
		);
	});
}

#[test]
fn test_abr_different_length() {
	// Get a test environment
	let mut env = setup();

	env.execute_with(|| {
		let (mark_prices, index_prices) = mock_prices::get_eth_usdc_prices_1();
		let lengths: Vec<usize> = (0..16).map(|x| 30 + x * 30).collect();
		let expected_results = mock_prices::expected_prices_eth_usdc_1();

		for iterator in 0..16 {
			let result = PricesModule::calculate_abr(
				mark_prices[0..lengths[iterator]].to_vec(),
				index_prices[0..lengths[iterator]].to_vec(),
				convert_to_fixed(0.000025_f64),
				convert_to_fixed(1.5),
				8_usize,
			);

			compare_with_threshold(result.0, expected_results[iterator], convert_to_fixed(1e-10));
		}
	});
}

#[test]
fn test_abr_flow_for_btc_orders() {
	let mut env = setup_trading();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());

		// market id
		let market_id = btc_usdc().market.id;

		// Create orders
		let alice_order =
			Order::new(201_u128, alice_id).sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(202_u128, bob_id)
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
			1699940278000
		));

		// Change block timestamp
		Timestamp::set_timestamp(1699969078000);

		// Set ABR timestamp
		assert_ok!(PricesModule::set_abr_timestamp(RuntimeOrigin::signed(1), 1699969078));

		let abr_state = PricesModule::abr_state();
		println!("abr_state {:?}", abr_state);

		let epoch_to_timestamp = PricesModule::epoch_to_timestamp(1);
		println!("epoch_to_timestamp {:?}", epoch_to_timestamp);

		// Set mark and index prices
		set_prices(market_id);

		// Compute ABR value
		assert_ok!(PricesModule::set_abr_value(RuntimeOrigin::signed(1), market_id));

		let epoch_to_abr_value = PricesModule::epoch_market_to_abr_value(1, market_id);
		println!("epoch_to_abr_value: {:?}", epoch_to_abr_value);

		let epoch_market_to_last_price = PricesModule::epoch_market_to_last_price(1, market_id);
		println!("epoch_market_to_last_price: {:?}", epoch_market_to_last_price);

		// Pay ABR
		assert_ok!(PricesModule::make_abr_payments(RuntimeOrigin::signed(1)));

		let balance = TradingAccounts::balances(alice_id, btc_usdc().market.asset_collateral);
		println!("Alice balance: {:?}", balance);

		let balance = TradingAccounts::balances(bob_id, btc_usdc().market.asset_collateral);
		println!("Bob balance: {:?}", balance);

		let event_record = System::events();
		println!("Events: {:?}", event_record);
	});
}
