use crate::{mock::*, Event};
use frame_support::assert_ok;
use pallet_support::{
	test_helpers::{
		accounts_helper::{alice, bob, charlie, dave, get_private_key, get_trading_account_id},
		asset_helper::{btc, eth, link, usdc},
		market_helper::{btc_usdc, eth_usdc, link_usdc},
	},
	types::{ABRState, Direction, MultiplePrices, Order, OrderType},
};
use primitive_types::U256;
use sp_arithmetic::{fixed_point::FixedI128, traits::One};

// declare test_helper module
pub mod test_helper;
use sp_runtime::traits::Zero;
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
			vec![btc_usdc(), link_usdc(), eth_usdc()]
		));

		// Add accounts to the system
		assert_ok!(TradingAccounts::add_accounts(
			RuntimeOrigin::signed(1),
			vec![alice(), bob(), charlie(), dave()]
		));

		// Set ABR interval as 8 hours
		assert_ok!(PricesModule::set_abr_interval(RuntimeOrigin::root(), 28800));

		// Set Base ABR as 0.000025
		assert_ok!(PricesModule::set_base_abr(
			RuntimeOrigin::root(),
			FixedI128::from_inner(25000000000000)
		));

		// Set Bollinger width as 1.5
		assert_ok!(PricesModule::set_bollinger_width(
			RuntimeOrigin::root(),
			FixedI128::from_inner(1500000000000000000)
		));

		// Set no.of users per batch
		assert_ok!(PricesModule::set_no_of_users_per_batch(RuntimeOrigin::root(), 10));
	});
	env
}

fn set_prices(market_id: u128, mark_prices: Vec<FixedI128>, index_prices: Vec<FixedI128>) {
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

		System::assert_has_event(Event::PricesUpdated { timestamp: 1699940367, prices }.into());
	});
}

#[test]
#[should_panic(expected = "FutureTimestampPriceUpdate")]
fn test_update_prices_with_future_timestamp() {
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
		PricesModule::update_prices(RuntimeOrigin::signed(1), prices.clone(), 1702359660000)
			.expect("Update price request for the future timestamp");
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

		Timestamp::set_timestamp(1702359620000);

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

		Timestamp::set_timestamp(1702359661000);

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
fn test_historical_prices_cleanup_after_timelimit() {
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
			index_price: 300.into(),
			mark_price: 301.into(),
		};
		let mark_price2 = MultiplePrices {
			market_id: market2.market.id,
			index_price: 400.into(),
			mark_price: 401.into(),
		};
		prices.push(mark_price1);
		prices.push(mark_price2);
		assert_ok!(PricesModule::update_prices(
			RuntimeOrigin::signed(1),
			prices.clone(),
			1702359500000
		));

		// Timestamp should be updated to this current timestamp,
		// as start_timestamp is None at the moment
		let start_timestamp = PricesModule::prices_start_timestamp().unwrap();
		assert_eq!(start_timestamp, 1702359500);

		let mut prices: Vec<MultiplePrices> = Vec::new();
		let mark_price1 = MultiplePrices {
			market_id: market1.market.id,
			index_price: 100.into(),
			mark_price: 101.into(),
		};
		let mark_price2 = MultiplePrices {
			market_id: market2.market.id,
			index_price: 200.into(),
			mark_price: 201.into(),
		};
		prices.push(mark_price1);
		prices.push(mark_price2);
		assert_ok!(PricesModule::update_prices(
			RuntimeOrigin::signed(1),
			prices.clone(),
			1702359400000
		));

		// Timestamp should be updated to this current timestamp,
		// as it is less than previous timestamp
		let start_timestamp = PricesModule::prices_start_timestamp().unwrap();
		assert_eq!(start_timestamp, 1702359400);

		let mut prices: Vec<MultiplePrices> = Vec::new();
		let mark_price1 = MultiplePrices {
			market_id: market1.market.id,
			index_price: 500.into(),
			mark_price: 501.into(),
		};
		let mark_price2 = MultiplePrices {
			market_id: market2.market.id,
			index_price: 600.into(),
			mark_price: 601.into(),
		};
		prices.push(mark_price1);
		prices.push(mark_price2);
		assert_ok!(PricesModule::update_prices(
			RuntimeOrigin::signed(1),
			prices.clone(),
			1702359601000
		));

		// Timestamp shouldn't be updated to this current timestamp,
		// as it is more than previous timestamp
		let start_timestamp = PricesModule::prices_start_timestamp().unwrap();
		assert_eq!(start_timestamp, 1702359400);

		// Increment the blocktimestamp by 4 weeks
		Timestamp::set_timestamp(1704779800000);

		// Perform cleanup of historical price data
		assert_ok!(PricesModule::perform_prices_cleanup(RuntimeOrigin::signed(1)));

		// Read historical prices after cleanup, every price should show as zero
		let historical_price = PricesModule::historical_price(1702359500, market1.market.id);
		assert_eq!(FixedI128::zero(), historical_price.mark_price);
		assert_eq!(FixedI128::zero(), historical_price.index_price);

		let historical_price = PricesModule::historical_price(1702359500, market2.market.id);
		assert_eq!(FixedI128::zero(), historical_price.mark_price);
		assert_eq!(FixedI128::zero(), historical_price.index_price);

		let historical_price = PricesModule::historical_price(1702359400, market1.market.id);
		assert_eq!(FixedI128::zero(), historical_price.mark_price);
		assert_eq!(FixedI128::zero(), historical_price.index_price);

		let historical_price = PricesModule::historical_price(1702359400, market2.market.id);
		assert_eq!(FixedI128::zero(), historical_price.mark_price);
		assert_eq!(FixedI128::zero(), historical_price.index_price);

		let historical_price = PricesModule::historical_price(1702359601, market1.market.id);
		assert_eq!(FixedI128::zero(), historical_price.mark_price);
		assert_eq!(FixedI128::zero(), historical_price.index_price);

		let historical_price = PricesModule::historical_price(1702359601, market2.market.id);
		assert_eq!(FixedI128::zero(), historical_price.mark_price);
		assert_eq!(FixedI128::zero(), historical_price.index_price);
	});
}

#[test]
fn test_historical_prices_cleanup_before_timelimit() {
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
			index_price: 300.into(),
			mark_price: 301.into(),
		};
		let mark_price2 = MultiplePrices {
			market_id: market2.market.id,
			index_price: 400.into(),
			mark_price: 401.into(),
		};
		prices.push(mark_price1);
		prices.push(mark_price2);
		assert_ok!(PricesModule::update_prices(
			RuntimeOrigin::signed(1),
			prices.clone(),
			1702359620000
		));

		// Timestamp should be updated to this current timestamp,
		// as start_timestamp is None at the moment
		let start_timestamp = PricesModule::prices_start_timestamp().unwrap();
		assert_eq!(start_timestamp, 1702359620);

		let mut prices: Vec<MultiplePrices> = Vec::new();
		let mark_price1 = MultiplePrices {
			market_id: market1.market.id,
			index_price: 100.into(),
			mark_price: 101.into(),
		};
		let mark_price2 = MultiplePrices {
			market_id: market2.market.id,
			index_price: 200.into(),
			mark_price: 201.into(),
		};
		prices.push(mark_price1);
		prices.push(mark_price2);
		assert_ok!(PricesModule::update_prices(
			RuntimeOrigin::signed(1),
			prices.clone(),
			1702359600000
		));

		// Timestamp should be updated to this current timestamp,
		// as it is less than previous timestamp
		let start_timestamp = PricesModule::prices_start_timestamp().unwrap();
		assert_eq!(start_timestamp, 1702359600);

		let mut prices: Vec<MultiplePrices> = Vec::new();
		let mark_price1 = MultiplePrices {
			market_id: market1.market.id,
			index_price: 500.into(),
			mark_price: 501.into(),
		};
		let mark_price2 = MultiplePrices {
			market_id: market2.market.id,
			index_price: 600.into(),
			mark_price: 601.into(),
		};
		prices.push(mark_price1);
		prices.push(mark_price2);
		assert_ok!(PricesModule::update_prices(
			RuntimeOrigin::signed(1),
			prices.clone(),
			1702359661000
		));

		// Timestamp shouldn't be updated to this current timestamp,
		// as it is more than previous timestamp
		let start_timestamp = PricesModule::prices_start_timestamp().unwrap();
		assert_eq!(start_timestamp, 1702359600);

		// Increment the blocktimestamp by 2 weeks
		Timestamp::set_timestamp(1703569200000);

		// Perform cleanup of historical price data
		assert_ok!(PricesModule::perform_prices_cleanup(RuntimeOrigin::signed(1)));

		// Prices remian intact becuase we called cleanup before the timelimit
		let historical_price = PricesModule::historical_price(1702359661, market1.market.id);
		assert_eq!(FixedI128::from_u32(501), historical_price.mark_price);
		assert_eq!(FixedI128::from_u32(500), historical_price.index_price);

		let historical_price = PricesModule::historical_price(1702359661, market2.market.id);
		assert_eq!(FixedI128::from_u32(601), historical_price.mark_price);
		assert_eq!(FixedI128::from_u32(600), historical_price.index_price);

		let historical_price = PricesModule::historical_price(1702359620, market1.market.id);
		assert_eq!(FixedI128::from_u32(301), historical_price.mark_price);
		assert_eq!(FixedI128::from_u32(300), historical_price.index_price);

		let historical_price = PricesModule::historical_price(1702359620, market2.market.id);
		assert_eq!(FixedI128::from_u32(401), historical_price.mark_price);
		assert_eq!(FixedI128::from_u32(400), historical_price.index_price);

		let historical_price = PricesModule::historical_price(1702359600, market1.market.id);
		assert_eq!(FixedI128::from_u32(101), historical_price.mark_price);
		assert_eq!(FixedI128::from_u32(100), historical_price.index_price);

		let historical_price = PricesModule::historical_price(1702359600, market2.market.id);
		assert_eq!(FixedI128::from_u32(201), historical_price.mark_price);
		assert_eq!(FixedI128::from_u32(200), historical_price.index_price);
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
fn test_abr_calculation_btc_usdc_debug() {
	// Get a test environment
	let mut env = setup();

	env.execute_with(|| {
		let (mark_prices, index_prices) = mock_prices::get_btc_usdc_debug();
		let result = PricesModule::calculate_abr(
			mark_prices,
			index_prices,
			convert_to_fixed(0.000025_f64),
			convert_to_fixed(1.5),
			8_usize,
		);
		compare_with_threshold(
			result.0,
			convert_to_fixed(2.5124864797511748e-05),
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
#[should_panic(expected = "Error while setting ABR interval: BadOrigin")]
fn test_unauthorized_set_abr_interval() {
	let mut env = setup_trading();

	env.execute_with(|| {
		PricesModule::set_abr_interval(RuntimeOrigin::signed(1), 100)
			.expect("Error while setting ABR interval");
	});
}

#[test]
#[should_panic(expected = "InvalidAbrInterval")]
fn test_invalid_abr_interval() {
	let mut env = setup_trading();

	env.execute_with(|| {
		PricesModule::set_abr_interval(RuntimeOrigin::root(), 100)
			.expect("Error while setting ABR interval");
	});
}

#[test]
#[should_panic(expected = "Error while setting base ABR: BadOrigin")]
fn test_unauthorized_set_base_abr() {
	let mut env = setup_trading();

	env.execute_with(|| {
		PricesModule::set_base_abr(RuntimeOrigin::signed(1), FixedI128::from_inner(12000000000000))
			.expect("Error while setting base ABR");
	});
}

#[test]
#[should_panic(expected = "InvalidBaseAbr")]
fn test_invalid_base_abr() {
	let mut env = setup_trading();

	env.execute_with(|| {
		PricesModule::set_base_abr(RuntimeOrigin::root(), FixedI128::from_inner(12000000000000))
			.expect("Error while setting base ABR");
	});
}

#[test]
#[should_panic(expected = "InvalidUsersPerBatch")]
fn test_invalid_no_of_users() {
	let mut env = setup_trading();

	env.execute_with(|| {
		PricesModule::set_no_of_users_per_batch(RuntimeOrigin::root(), 0)
			.expect("Error while setting No.of users per batch");
	});
}

#[test]
#[should_panic(expected = "MarketNotTradable")]
fn test_set_abr_untradable_market() {
	let mut env = setup_trading();

	env.execute_with(|| {
		assert_ok!(PricesModule::set_initialisation_timestamp(
			RuntimeOrigin::root(),
			1699940278000
		));

		// market id
		let link_market_id = link_usdc().market.id;

		// Change block timestamp
		Timestamp::set_timestamp(1699979078000);

		// link_usdc is not tradable
		PricesModule::set_abr_value(RuntimeOrigin::signed(1), link_market_id)
			.expect("Error while setting abr value");
	});
}

#[test]
#[should_panic(expected = "AbrValueAlreadySet")]
fn test_set_abr_value_for_already_set_market() {
	let mut env = setup_trading();

	env.execute_with(|| {
		assert_ok!(PricesModule::set_initialisation_timestamp(
			RuntimeOrigin::root(),
			1699940278000
		));

		// market id
		let btc_market_id = btc_usdc().market.id;

		// Change block timestamp
		Timestamp::set_timestamp(1699979078000);

		// Set mark and index prices
		let (mark_prices, index_prices) = mock_prices::get_btc_usdc_prices_1();
		set_prices(btc_market_id, mark_prices, index_prices);

		assert_ok!(PricesModule::set_abr_value(RuntimeOrigin::signed(1), btc_market_id));
		// setting abr value for already set market
		PricesModule::set_abr_value(RuntimeOrigin::signed(1), btc_market_id)
			.expect("Error while setting abr value");
	});
}

#[test]
fn test_set_abr_value_when_prices_array_is_empty() {
	let mut env = setup_trading();

	env.execute_with(|| {
		assert_ok!(PricesModule::set_initialisation_timestamp(
			RuntimeOrigin::root(),
			1699940278000
		));

		// market id
		let btc_market_id = btc_usdc().market.id;

		// Change block timestamp
		Timestamp::set_timestamp(1699979078000);

		// setting abr value when prices array is empty
		assert_ok!(PricesModule::set_abr_value(RuntimeOrigin::signed(1), btc_market_id));

		let epoch_to_abr_value = PricesModule::epoch_market_to_abr_value(1, btc_market_id);
		assert_eq!(epoch_to_abr_value, FixedI128::zero());

		let epoch_market_to_last_price = PricesModule::epoch_market_to_last_price(1, btc_market_id);
		assert_eq!(epoch_market_to_last_price, FixedI128::zero());
	});
}

#[test]
#[should_panic(expected = "InvalidState")]
fn test_set_abr_value_with_invalid_state() {
	let mut env = setup_trading();

	env.execute_with(|| {
		assert_ok!(PricesModule::set_initialisation_timestamp(
			RuntimeOrigin::root(),
			1699940278000
		));

		// market id
		let btc_market_id = btc_usdc().market.id;
		let eth_market_id = eth_usdc().market.id;

		// Change block timestamp
		Timestamp::set_timestamp(1699979078000);

		// Set mark and index prices
		let (mark_prices_btc, index_prices_btc) = mock_prices::get_btc_usdc_prices_1();
		let (mark_prices_eth, index_prices_eth) = mock_prices::get_btc_usdt_prices_1();
		set_prices(btc_market_id, mark_prices_btc, index_prices_btc);
		set_prices(eth_market_id, mark_prices_eth, index_prices_eth);

		assert_ok!(PricesModule::set_abr_value(RuntimeOrigin::signed(1), btc_market_id));
		assert_ok!(PricesModule::set_abr_value(RuntimeOrigin::signed(1), eth_market_id));
		// calling set_abr_value again with state changed to 2
		PricesModule::set_abr_value(RuntimeOrigin::signed(1), btc_market_id)
			.expect("Error while setting abr value");
	});
}

#[test]
#[should_panic(expected = "InvalidState")]
fn test_pay_abr_with_invalid_state() {
	let mut env = setup_trading();

	env.execute_with(|| {
		PricesModule::make_abr_payments(RuntimeOrigin::signed(1))
			.expect("Error while making abr payments");
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

		let abr_interval = PricesModule::abr_interval();
		assert_eq!(abr_interval, 28800); // should be 8 hours

		assert_ok!(PricesModule::set_initialisation_timestamp(
			RuntimeOrigin::root(),
			1699940278000
		));

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
			1699940278000
		));

		// Change block timestamp
		Timestamp::set_timestamp(1699969078000);

		let abr_state = PricesModule::abr_state();
		assert_eq!(abr_state, ABRState::State0);

		let epoch_to_timestamp = PricesModule::epoch_to_timestamp(1);
		assert_eq!(epoch_to_timestamp, 0);

		// Set mark and index prices
		let (mark_prices_btc, index_prices_btc) = mock_prices::get_btc_usdc_prices_1();
		set_prices(market_id, mark_prices_btc.clone(), index_prices_btc.clone());
		set_prices(eth_usdc().market.id, mark_prices_btc, index_prices_btc);

		// Compute ABR value
		assert_ok!(PricesModule::set_abr_value(RuntimeOrigin::signed(1), market_id));
		assert_ok!(PricesModule::set_abr_value(RuntimeOrigin::signed(1), eth_usdc().market.id));
		let abr_state = PricesModule::abr_state();
		assert_eq!(abr_state, ABRState::State2);

		let epoch_to_timestamp = PricesModule::epoch_to_timestamp(1);
		assert_eq!(epoch_to_timestamp, 1699940278 + 28800);

		let epoch_to_abr_value = PricesModule::epoch_market_to_abr_value(1, market_id);

		let epoch_market_to_last_price = PricesModule::epoch_market_to_last_price(1, market_id);

		let alice_before_balance =
			TradingAccounts::balances(alice_id, btc_usdc().market.asset_collateral);
		let bob_before_balance =
			TradingAccounts::balances(bob_id, btc_usdc().market.asset_collateral);

		// Pay ABR
		assert_ok!(PricesModule::make_abr_payments(RuntimeOrigin::signed(1)));

		let alice_after_balance =
			TradingAccounts::balances(alice_id, btc_usdc().market.asset_collateral);

		let bob_after_balance =
			TradingAccounts::balances(bob_id, btc_usdc().market.asset_collateral);

		let abr_payment = epoch_to_abr_value * epoch_market_to_last_price * FixedI128::one();
		compare_with_threshold(
			alice_after_balance,
			alice_before_balance - abr_payment,
			convert_to_fixed(1e-6),
		); // abr is +ve, long pays short

		compare_with_threshold(
			bob_after_balance,
			bob_before_balance + abr_payment,
			convert_to_fixed(1e-6),
		); // abr is +ve, short receives from long

		let event_record = System::events();
		println!("Events: {:?}", event_record);
	});
}

#[test]
fn test_abr_flow_for_btc_and_eth_orders() {
	let mut env = setup_trading();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());

		// market id
		let btc_market_id = btc_usdc().market.id;
		let eth_market_id = eth_usdc().market.id;

		let abr_interval = PricesModule::abr_interval();
		assert_eq!(abr_interval, 28800); // should be 8 hours

		assert_ok!(PricesModule::set_initialisation_timestamp(
			RuntimeOrigin::root(),
			1699940278000
		));

		// Create btc orders
		let alice_order =
			Order::new(U256::from(101), alice_id).sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(U256::from(102), bob_id)
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
			btc_market_id,
			// oracle_price
			100.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			1699940278000
		));

		// Create eth orders
		let alice_order = Order::new(U256::from(201), alice_id)
			.set_direction(Direction::Short)
			.set_market_id(3)
			.set_order_type(OrderType::Market)
			.sign_order(get_private_key(alice().pub_key));
		let bob_order = Order::new(U256::from(202), bob_id)
			.set_market_id(3)
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch_id
			U256::from(2_u8),
			// quantity_locked
			1.into(),
			// market_id
			eth_market_id,
			// oracle_price
			100.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()],
			// batch_timestamp
			1699940278000
		));

		// Change block timestamp
		Timestamp::set_timestamp(1699969078000);

		let abr_state = PricesModule::abr_state();
		assert_eq!(abr_state, ABRState::State0);

		let epoch_to_timestamp = PricesModule::epoch_to_timestamp(1);
		assert_eq!(epoch_to_timestamp, 0);

		// Set mark and index prices
		let (mark_prices_btc, index_prices_btc) = mock_prices::get_btc_usdc_prices_1();
		set_prices(btc_market_id, mark_prices_btc.clone(), index_prices_btc.clone());
		set_prices(eth_market_id, mark_prices_btc, index_prices_btc);

		// Compute ABR value
		assert_ok!(PricesModule::set_abr_value(RuntimeOrigin::signed(1), btc_market_id));
		let abr_state = PricesModule::abr_state();
		assert_eq!(abr_state, ABRState::State1);
		assert_ok!(PricesModule::set_abr_value(RuntimeOrigin::signed(1), eth_market_id));
		let abr_state = PricesModule::abr_state();
		assert_eq!(abr_state, ABRState::State2);

		let epoch_to_timestamp = PricesModule::epoch_to_timestamp(1);
		assert_eq!(epoch_to_timestamp, 1699940278 + 28800);

		let epoch_to_abr_value = PricesModule::epoch_market_to_abr_value(1, btc_market_id);
		println!("btc epoch_to_abr_value: {:?}", epoch_to_abr_value);

		let epoch_market_to_last_price = PricesModule::epoch_market_to_last_price(1, btc_market_id);
		println!("btc epoch_market_to_last_price: {:?}", epoch_market_to_last_price);

		let epoch_to_abr_value = PricesModule::epoch_market_to_abr_value(1, eth_market_id);
		println!("eth epoch_to_abr_value: {:?}", epoch_to_abr_value);

		let epoch_market_to_last_price = PricesModule::epoch_market_to_last_price(1, eth_market_id);
		println!("eth epoch_market_to_last_price: {:?}", epoch_market_to_last_price);

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

#[test]
#[should_panic(expected = "Error while setting max abr: Bad Origin")]
fn test_set_max_abr_non_admin() {
	let mut env = setup_trading();

	env.execute_with(|| {
		PricesModule::set_max_abr(
			RuntimeOrigin::signed(1),
			btc_usdc().market.id,
			FixedI128::from_float(0.0001),
		)
		.expect("Error while setting max abr: Bad Origin");
	});
}

#[test]
#[should_panic(expected = "Error while setting max abr: Bad Origin")]
fn test_set_max_default_abr_non_admin() {
	let mut env = setup_trading();

	env.execute_with(|| {
		PricesModule::set_default_max_abr(RuntimeOrigin::signed(1), FixedI128::from_float(0.0001))
			.expect("Error while setting max abr: Bad Origin");
	});
}

#[test]
#[should_panic(expected = "MarketNotFound")]
fn test_set_max_abr_invalid_market() {
	let mut env = setup_trading();

	env.execute_with(|| {
		PricesModule::set_max_abr(RuntimeOrigin::root(), 8213_u128, FixedI128::from_float(0.0001))
			.expect("Error while setting max abr");
	});
}

#[test]
#[should_panic(expected = "MarketNotTradable")]
fn test_set_max_abr_non_tradable_market() {
	let mut env = setup_trading();

	env.execute_with(|| {
		PricesModule::set_max_abr(
			RuntimeOrigin::root(),
			link_usdc().market.id,
			FixedI128::from_float(0.0001),
		)
		.expect("Error while setting max abr");
	});
}

#[test]
#[should_panic(expected = "NegativeMaxValue")]
fn test_set_default_max_abr_negative_value() {
	let mut env = setup_trading();

	env.execute_with(|| {
		PricesModule::set_default_max_abr(RuntimeOrigin::root(), FixedI128::from_float(-0.0001))
			.expect("Error while setting max abr");
	});
}

#[test]
#[should_panic(expected = "NegativeMaxValue")]
fn test_set_max_abr_negative_value() {
	let mut env = setup_trading();

	env.execute_with(|| {
		PricesModule::set_max_abr(
			RuntimeOrigin::root(),
			btc_usdc().market.id,
			FixedI128::from_float(-0.0001),
		)
		.expect("Error while setting max abr");
	});
}

#[test]
fn test_set_max_abr_admin() {
	let mut env = setup_trading();

	env.execute_with(|| {
		assert_ok!(PricesModule::set_max_abr(
			RuntimeOrigin::root(),
			btc_usdc().market.id,
			FixedI128::from_float(0.0001),
		));
		assert_eq!(PricesModule::max_abr(btc_usdc().market.id), FixedI128::from_float(0.0001));
	});
}

#[test]
fn test_max_abr_flow() {
	let mut env = setup_trading();

	env.execute_with(|| {
		// Market_ids
		let btc_market_id = btc_usdc().market.id;
		let eth_market_id = eth_usdc().market.id;

		// Set init time
		assert_ok!(PricesModule::set_initialisation_timestamp(
			RuntimeOrigin::root(),
			1699940278000
		));

		// Change block timestamp
		Timestamp::set_timestamp(1699969078000);

		// Set max abr values
		assert_ok!(PricesModule::set_max_abr(
			RuntimeOrigin::root(),
			btc_market_id,
			FixedI128::from_float(4.1e-05),
		));

		assert_ok!(PricesModule::set_max_abr(
			RuntimeOrigin::root(),
			eth_market_id,
			FixedI128::from_float(1.1e-04),
		));

		// Set prices
		let (mark_prices_btc, index_prices_btc) = mock_prices::get_btc_usdc_prices_1();
		let (mark_prices_eth, index_prices_eth) = mock_prices::get_btc_usdt_prices_1();
		set_prices(btc_market_id, mark_prices_btc, index_prices_btc);
		set_prices(eth_market_id, mark_prices_eth, index_prices_eth);

		// Set the abr value
		assert_ok!(PricesModule::set_abr_value(RuntimeOrigin::signed(1), btc_market_id));
		assert_ok!(PricesModule::set_abr_value(RuntimeOrigin::signed(1), eth_market_id));

		// Compare the abr values
		// Actual value is 8.83808701975073e-05
		// It gets reduced to max value for btc market which is 4.1e-05
		assert_eq!(
			PricesModule::epoch_market_to_abr_value(1, btc_market_id),
			FixedI128::from_float(4.1e-05)
		);
		// Actual value is -2.730150595400045e-04
		// It gets reduced to max abs value for eth market which is -1.1e-04
		assert_eq!(
			PricesModule::epoch_market_to_abr_value(1, eth_market_id),
			FixedI128::from_float(-1.1e-04)
		);
	});
}

#[test]
fn test_default_max_abr_flow() {
	let mut env = setup_trading();

	env.execute_with(|| {
		// Market_ids
		let btc_market_id = btc_usdc().market.id;
		let eth_market_id = eth_usdc().market.id;

		// Set init time
		assert_ok!(PricesModule::set_initialisation_timestamp(
			RuntimeOrigin::root(),
			1699940278000
		));

		// Change block timestamp
		Timestamp::set_timestamp(1699969078000);

		// Set max abr values
		assert_ok!(PricesModule::set_default_max_abr(
			RuntimeOrigin::root(),
			FixedI128::from_float(2.5e-05),
		));
		assert_ok!(PricesModule::set_max_abr(
			RuntimeOrigin::root(),
			eth_market_id,
			FixedI128::from_float(1.1e-04),
		));

		// Set prices
		let (mark_prices_btc, index_prices_btc) = mock_prices::get_btc_usdc_prices_1();
		let (mark_prices_eth, index_prices_eth) = mock_prices::get_btc_usdt_prices_1();
		set_prices(btc_market_id, mark_prices_btc, index_prices_btc);
		set_prices(eth_market_id, mark_prices_eth, index_prices_eth);

		// Set the abr value
		assert_ok!(PricesModule::set_abr_value(RuntimeOrigin::signed(1), btc_market_id));
		assert_ok!(PricesModule::set_abr_value(RuntimeOrigin::signed(1), eth_market_id));

		// Compare the abr values
		// Actual value is 8.83808701975073e-05
		// It gets reduced to max value for btc market which is 4.1e-05
		assert_eq!(
			PricesModule::epoch_market_to_abr_value(1, btc_market_id),
			FixedI128::from_float(2.5e-05)
		);
		// Actual value is -2.730150595400045e-04
		// It gets reduced to max abs value for eth market which is -1.1e-04
		assert_eq!(
			PricesModule::epoch_market_to_abr_value(1, eth_market_id),
			FixedI128::from_float(-1.1e-04)
		);
	});
}
