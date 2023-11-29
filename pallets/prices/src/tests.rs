use crate::mock::*;
use frame_support::assert_ok;
use pallet_support::{
	test_helpers::{
		asset_helper::{eth, link, usdc},
		market_helper::{eth_usdc, link_usdc},
	},
	types::MultiplePrices,
};
use sp_arithmetic::fixed_point::FixedI128;

// declare test_helper module
pub mod test_helper;
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
