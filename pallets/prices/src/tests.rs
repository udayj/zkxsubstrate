use crate::{mock::*, Event};
use frame_support::assert_ok;
use pallet_support::{
	helpers::{fixed_pow, ln, max},
	test_helpers::{
		asset_helper::{eth, link, usdc},
		market_helper::{eth_usdc, link_usdc},
	},
	types::{ExtendedMarket, MultiplePrices},
};
use sp_arithmetic::{
	fixed_point::FixedI128,
	traits::{One, Zero},
};
fn get_data() -> (Vec<FixedI128>, Vec<FixedI128>) {
	let eth_mark_f64 = vec![
		281.36, 281.57, 281.58, 281.67, 281.57, 281.57, 281.69, 281.69, 281.53, 281.69, 281.7,
		281.52, 281.4, 281.44, 281.32, 281.26, 281.42, 281.62, 281.87, 281.78, 281.67, 281.37,
		281.32, 281.3, 281.18, 281.24, 281.26, 281.44, 281.57, 281.54, 281.51, 281.47, 281.45,
		281.64, 281.84, 281.88, 281.92, 281.8, 281.86, 281.8, 281.78, 281.75, 281.66, 281.71,
		281.78, 281.43, 281.68, 281.78, 281.6, 281.53, 281.41, 281.37, 281.46, 281.64, 281.59,
		281.81, 281.82, 282.01, 282.06, 282.22, 282.08, 282.02, 282.01, 281.85, 281.95, 282.15,
		281.99, 282.08, 281.97, 282.14, 282.0, 281.96, 281.93, 281.83, 281.7, 281.67, 281.62,
		281.79, 282.05, 281.85, 281.79, 281.83, 281.81, 281.96, 282.02, 282.01, 281.94, 281.84,
		281.68, 281.53, 281.65, 281.66, 281.64, 281.51, 281.68, 281.74, 281.49, 281.47, 281.32,
		281.41, 281.31, 281.26, 281.29, 281.35, 281.43, 281.33, 281.54, 281.79, 281.73, 281.8,
		281.82, 281.85, 281.76, 281.73, 281.8, 281.94, 281.87, 281.91, 281.96, 282.27, 282.21,
		282.32, 282.09, 281.91, 281.82, 281.74, 281.91, 281.66, 281.69, 281.5, 281.63, 281.44,
		281.68, 281.8, 281.72, 282.02, 282.0, 281.87, 282.09, 281.82, 281.56, 281.88, 282.11,
		282.26, 282.05, 281.89, 281.65, 281.64, 281.71, 281.66, 281.7, 281.74, 281.63, 281.5,
		281.54, 281.5, 281.57, 281.24, 281.06, 281.12, 281.55, 281.59, 281.41, 281.33, 281.16,
		280.98, 280.99, 280.69, 280.66, 280.56, 280.35, 280.49, 280.55, 280.72, 280.99, 280.75,
		280.92, 281.14, 280.8, 280.9, 280.9, 280.88, 280.95, 280.93, 281.36, 281.3, 281.35, 281.2,
		281.44, 281.09, 280.9, 280.78, 280.61, 280.62, 280.64, 280.72, 280.79, 280.59, 280.51,
		280.36, 280.38, 280.67, 281.12, 281.18, 281.1, 281.07, 281.02, 280.98, 281.01, 280.86,
		281.23, 280.68, 281.25, 281.28, 281.37, 281.38, 281.07, 281.39, 281.26, 281.3, 281.34,
		281.72, 281.69, 281.66, 281.49, 281.75, 281.45, 281.7, 282.1, 282.28, 282.27, 282.31,
		282.24, 282.33, 282.37, 282.29, 282.43, 282.4, 282.61, 282.6, 283.05, 282.55, 282.34,
		282.22, 282.43, 282.47, 282.56, 282.55, 282.62, 282.59, 282.84, 282.89, 282.69, 282.71,
		282.62, 282.83, 282.85, 283.14, 283.6, 283.13, 282.55, 282.23, 282.26, 282.14, 282.08,
		281.97, 282.2, 282.16, 282.05, 281.67, 281.85, 281.67, 281.66, 281.57, 281.63, 281.36,
		281.43, 281.37, 281.36, 281.43, 281.42, 281.4, 281.2, 281.29, 281.08, 281.38, 281.16,
		281.11, 281.51, 281.93, 282.36, 282.35, 281.89, 281.87, 281.7, 281.97, 281.83, 281.54,
		281.51, 281.52, 281.52, 281.54, 281.14, 281.08, 280.95, 281.13, 281.16, 281.18, 280.86,
		281.16, 281.37, 281.51, 281.09, 281.1, 281.23, 281.41, 281.33, 281.32, 281.46, 281.43,
		281.44, 281.52, 281.18, 281.29, 281.55, 281.55, 281.78, 281.42, 281.19, 280.89, 280.94,
		281.09, 281.21, 281.3, 281.29, 281.23, 281.46, 281.51, 281.43, 281.28, 281.19, 281.1,
		281.19, 280.91, 280.99, 280.99, 281.15, 280.84, 280.99, 280.95, 281.25, 281.52, 281.28,
		281.49, 281.89, 281.42, 281.17, 281.11, 281.13, 281.15, 280.9, 281.13, 280.85, 281.04,
		280.96, 280.96, 280.81, 281.11, 281.21, 281.05, 281.02, 280.93, 280.62, 280.75, 280.71,
		280.17, 280.21, 280.25, 280.46, 280.33, 279.85, 279.63, 279.84, 279.92, 279.82, 279.7,
		279.9, 279.9, 280.0, 280.22, 279.97, 279.96, 279.91, 279.81, 279.43, 279.46, 279.31,
		279.36, 279.1, 279.42, 279.26, 279.06, 279.02, 278.86, 278.22, 278.09, 278.18, 277.88,
		278.76, 278.74, 279.03, 279.32, 279.4, 279.54, 279.75, 279.67, 279.5, 279.44, 279.65,
		279.69, 279.76, 279.5, 279.42, 279.52, 279.58, 279.44, 279.66, 279.6, 279.21, 279.84,
		280.13, 280.52, 280.82, 280.94, 280.96, 280.98, 280.85, 280.88, 280.92, 280.68, 280.83,
		280.8, 280.55, 280.54, 280.56, 280.39, 280.31, 280.24, 280.5, 280.91, 280.93, 281.05,
		281.07, 281.11, 281.56, 281.37, 281.27, 281.3, 281.15, 281.0, 281.07, 281.02, 280.96,
		280.61, 280.71, 280.92, 280.67, 280.4, 280.24, 280.37, 280.6, 280.49, 280.53, 280.45,
		280.4, 280.01, 279.88, 279.92, 280.36, 280.65,
	];

	let eth_spot_f64 = vec![
		281.35, 281.55, 281.55, 281.63, 281.55, 281.57, 281.66, 281.71, 281.5, 281.64, 281.68,
		281.51, 281.4, 281.43, 281.29, 281.24, 281.43, 281.61, 281.84, 281.75, 281.69, 281.36,
		281.31, 281.29, 281.2, 281.25, 281.24, 281.42, 281.56, 281.53, 281.47, 281.48, 281.45,
		281.63, 281.81, 281.86, 281.89, 281.79, 281.82, 281.75, 281.74, 281.69, 281.64, 281.69,
		281.76, 281.41, 281.65, 281.71, 281.57, 281.5, 281.37, 281.36, 281.43, 281.6, 281.52,
		281.76, 281.74, 281.95, 282.02, 282.17, 282.05, 281.98, 282.0, 281.84, 281.93, 282.17,
		281.97, 282.08, 281.94, 282.06, 281.94, 281.93, 281.93, 281.78, 281.65, 281.66, 281.58,
		281.8, 281.97, 281.79, 281.74, 281.79, 281.74, 281.9, 281.95, 281.92, 281.86, 281.78,
		281.63, 281.48, 281.58, 281.58, 281.58, 281.44, 281.62, 281.67, 281.41, 281.38, 281.26,
		281.35, 281.26, 281.22, 281.27, 281.32, 281.36, 281.28, 281.51, 281.75, 281.69, 281.75,
		281.76, 281.77, 281.67, 281.66, 281.7, 281.86, 281.82, 281.84, 281.87, 282.2, 282.15,
		282.24, 281.99, 281.83, 281.71, 281.64, 281.81, 281.53, 281.58, 281.41, 281.51, 281.37,
		281.57, 281.68, 281.7, 281.88, 281.87, 281.79, 281.96, 281.74, 281.62, 281.82, 282.06,
		282.14, 281.93, 281.75, 281.55, 281.56, 281.57, 281.57, 281.57, 281.61, 281.53, 281.4,
		281.44, 281.38, 281.45, 281.11, 280.98, 281.06, 281.48, 281.51, 281.31, 281.25, 281.03,
		280.89, 280.88, 280.64, 280.6, 280.52, 280.33, 280.48, 280.54, 280.72, 280.94, 280.71,
		280.88, 281.06, 280.77, 280.82, 280.86, 280.86, 280.93, 280.86, 281.29, 281.26, 281.29,
		281.14, 281.37, 281.06, 280.86, 280.74, 280.56, 280.56, 280.57, 280.65, 280.68, 280.5,
		280.45, 280.31, 280.3, 280.59, 281.04, 281.12, 281.02, 281.01, 280.91, 280.89, 280.93,
		280.8, 281.15, 280.6, 281.19, 281.21, 281.29, 281.3, 280.94, 281.28, 281.18, 281.23,
		281.23, 281.6, 281.6, 281.55, 281.41, 281.67, 281.31, 281.61, 281.99, 282.16, 282.15,
		282.17, 282.18, 282.3, 282.33, 282.26, 282.41, 282.39, 282.56, 282.57, 282.99, 282.54,
		282.32, 282.21, 282.42, 282.44, 282.55, 282.53, 282.6, 282.55, 282.84, 282.83, 282.62,
		282.64, 282.6, 282.76, 282.76, 283.03, 283.5, 283.05, 282.53, 282.16, 282.2, 282.12,
		282.03, 281.94, 282.14, 282.07, 281.97, 281.66, 281.78, 281.63, 281.63, 281.52, 281.59,
		281.35, 281.43, 281.36, 281.34, 281.4, 281.42, 281.38, 281.17, 281.26, 281.05, 281.36,
		281.08, 281.08, 281.51, 281.91, 282.26, 282.27, 281.84, 281.82, 281.61, 281.91, 281.77,
		281.48, 281.47, 281.48, 281.48, 281.46, 281.07, 281.01, 280.93, 281.07, 281.07, 281.11,
		280.8, 281.06, 281.27, 281.49, 281.09, 281.11, 281.21, 281.38, 281.31, 281.3, 281.43,
		281.4, 281.41, 281.49, 281.15, 281.26, 281.53, 281.51, 281.72, 281.39, 281.17, 280.84,
		280.93, 281.09, 281.23, 281.29, 281.28, 281.23, 281.46, 281.48, 281.42, 281.29, 281.13,
		281.05, 281.18, 280.9, 280.93, 280.95, 281.11, 280.78, 280.99, 280.95, 281.23, 281.51,
		281.3, 281.48, 281.85, 281.4, 281.13, 281.08, 281.09, 281.1, 280.85, 281.07, 280.82, 281.0,
		280.95, 280.96, 280.81, 281.07, 281.17, 281.01, 280.95, 280.89, 280.58, 280.67, 280.66,
		280.12, 280.16, 280.21, 280.41, 280.29, 279.86, 279.61, 279.83, 279.89, 279.75, 279.66,
		279.86, 279.88, 279.93, 280.18, 279.95, 279.94, 279.86, 279.76, 279.41, 279.44, 279.33,
		279.33, 279.06, 279.39, 279.18, 279.02, 278.97, 278.8, 278.16, 278.06, 278.14, 278.05,
		278.7, 278.75, 279.03, 279.31, 279.4, 279.47, 279.74, 279.6, 279.44, 279.36, 279.58,
		279.62, 279.69, 279.45, 279.35, 279.49, 279.52, 279.37, 279.59, 279.52, 279.23, 279.82,
		280.09, 280.52, 280.84, 280.93, 280.93, 280.96, 280.83, 280.86, 280.85, 280.65, 280.81,
		280.78, 280.53, 280.48, 280.54, 280.36, 280.29, 280.24, 280.48, 280.88, 280.96, 281.0,
		281.06, 281.09, 281.52, 281.36, 281.25, 281.26, 281.14, 280.9, 281.01, 280.92, 280.9,
		280.54, 280.67, 280.91, 280.6, 280.36, 280.21, 280.38, 280.57, 280.45, 280.5, 280.4,
		280.36, 279.96, 279.84, 279.87, 280.31, 280.59,
	];

	(convert_to_fixed(eth_mark_f64)[0..17].to_vec(), convert_to_fixed(eth_spot_f64)[0..17].to_vec())
}

fn convert_to_fixed(arr: Vec<f64>) -> Vec<FixedI128> {
	let total_len = arr.len();
	let mut fixed_arr = Vec::<FixedI128>::with_capacity(total_len);
	for iterator in 0..total_len {
		fixed_arr.push(FixedI128::from_inner((arr[iterator] * 10u128.pow(18) as f64) as i128));
	}

	fixed_arr
}

fn setup() -> sp_io::TestExternalities {
	// Create a new test environment
	let mut test_env = new_test_ext();

	let assets = vec![eth(), usdc(), link()];

	// Set the signers using admin account
	test_env.execute_with(|| {
		assert_ok!(Timestamp::set(None.into(), 1699940367000));
		assert_ok!(AssetModule::replace_all_assets(RuntimeOrigin::signed(1), assets));
	});

	test_env.into()
}

// #[test]
// fn test_update_prices() {
// 	new_test_ext().execute_with(|| {
// 		let (market1, market2) = setup();
// 		// Go past genesis block so events get deposited
// 		System::set_block_number(1);
// 		// Dispatch a signed extrinsic.
// 		let markets = vec![market1.clone(), market2.clone()];
// 		assert_ok!(MarketModule::replace_all_markets(RuntimeOrigin::signed(1), markets));
// 		let mut prices: Vec<MultiplePrices> = Vec::new();
// 		let mark_price1 = MultiplePrices {
// 			market_id: market1.market.id,
// 			index_price: 102.into(),
// 			mark_price: 100.into(),
// 		};
// 		let mark_price2 = MultiplePrices {
// 			market_id: market2.market.id,
// 			index_price: 199.into(),
// 			mark_price: 200.into(),
// 		};
// 		prices.push(mark_price1);
// 		prices.push(mark_price2);
// 		assert_ok!(PricesModule::update_prices(
// 			RuntimeOrigin::signed(1),
// 			prices.clone(),
// 			1699940367000
// 		));

// 		let price = PricesModule::current_price(market1.market.id);
// 		assert_eq!(FixedI128::from_u32(100), price.mark_price);
// 		assert_eq!(FixedI128::from_u32(102), price.index_price);

// 		let price = PricesModule::current_price(market2.market.id);
// 		assert_eq!(FixedI128::from_u32(200), price.mark_price);
// 		assert_eq!(FixedI128::from_u32(199), price.index_price);
// 	});
// }

// #[test]
// fn test_historical_prices() {
// 	new_test_ext().execute_with(|| {
// 		let (market1, market2) = setup();
// 		// Go past genesis block so events get deposited
// 		System::set_block_number(1);
// 		Timestamp::set_timestamp(1702359600000);

// 		let markets = vec![market1.clone(), market2.clone()];
// 		assert_ok!(MarketModule::replace_all_markets(RuntimeOrigin::signed(1), markets));

// 		assert_ok!(PricesModule::update_price_interval(RuntimeOrigin::signed(1), 60000));

// 		let mut prices: Vec<MultiplePrices> = Vec::new();
// 		let mark_price1 = MultiplePrices {
// 			market_id: market1.market.id,
// 			index_price: 102.into(),
// 			mark_price: 100.into(),
// 		};
// 		let mark_price2 = MultiplePrices {
// 			market_id: market2.market.id,
// 			index_price: 199.into(),
// 			mark_price: 200.into(),
// 		};
// 		prices.push(mark_price1);
// 		prices.push(mark_price2);
// 		assert_ok!(PricesModule::update_prices(
// 			RuntimeOrigin::signed(1),
// 			prices.clone(),
// 			1702359600000
// 		));

// 		let price = PricesModule::current_price(market1.market.id);
// 		assert_eq!(FixedI128::from_u32(100), price.mark_price);
// 		assert_eq!(FixedI128::from_u32(102), price.index_price);

// 		let price = PricesModule::current_price(market2.market.id);
// 		assert_eq!(FixedI128::from_u32(200), price.mark_price);
// 		assert_eq!(FixedI128::from_u32(199), price.index_price);

// 		let historical_price = PricesModule::historical_price(1702359600, market1.market.id);
// 		assert_eq!(FixedI128::from_u32(100), historical_price.mark_price);
// 		assert_eq!(FixedI128::from_u32(102), historical_price.index_price);

// 		let historical_price = PricesModule::historical_price(1702359600, market2.market.id);
// 		assert_eq!(FixedI128::from_u32(200), historical_price.mark_price);
// 		assert_eq!(FixedI128::from_u32(199), historical_price.index_price);

// 		let price_interval = PricesModule::price_interval();
// 		assert_eq!(60, price_interval);

// 		let timestamps = PricesModule::price_timestamps();
// 		assert_eq!(vec![1702359600], timestamps);

// 		let last_timestamp = PricesModule::last_timestamp();
// 		assert_eq!(1702359600, last_timestamp);

// 		// Set timestamp such that historical price will not get updated
// 		Timestamp::set_timestamp(1702359620000);

// 		let mut prices: Vec<MultiplePrices> = Vec::new();
// 		let mark_price1 = MultiplePrices {
// 			market_id: market1.market.id,
// 			index_price: 110.into(),
// 			mark_price: 109.into(),
// 		};
// 		let mark_price2 = MultiplePrices {
// 			market_id: market2.market.id,
// 			index_price: 190.into(),
// 			mark_price: 192.into(),
// 		};
// 		prices.push(mark_price1);
// 		prices.push(mark_price2);
// 		assert_ok!(PricesModule::update_prices(
// 			RuntimeOrigin::signed(1),
// 			prices.clone(),
// 			1702359620000
// 		));

// 		let price = PricesModule::current_price(market1.market.id);
// 		assert_eq!(FixedI128::from_u32(109), price.mark_price);
// 		assert_eq!(FixedI128::from_u32(110), price.index_price);

// 		let price = PricesModule::current_price(market2.market.id);
// 		assert_eq!(FixedI128::from_u32(192), price.mark_price);
// 		assert_eq!(FixedI128::from_u32(190), price.index_price);

// 		let historical_price = PricesModule::historical_price(1702359620, market1.market.id);
// 		assert_eq!(FixedI128::from_u32(0), historical_price.mark_price);
// 		assert_eq!(FixedI128::from_u32(0), historical_price.index_price);

// 		let historical_price = PricesModule::historical_price(1702359620, market2.market.id);
// 		assert_eq!(FixedI128::from_u32(0), historical_price.mark_price);
// 		assert_eq!(FixedI128::from_u32(0), historical_price.index_price);

// 		let timestamps = PricesModule::price_timestamps();
// 		assert_eq!(vec![1702359600], timestamps);

// 		let last_timestamp = PricesModule::last_timestamp();
// 		assert_eq!(1702359600, last_timestamp);

// 		// Set timestamp such that historical price will get updated
// 		Timestamp::set_timestamp(1702359661000);

// 		let mut prices: Vec<MultiplePrices> = Vec::new();
// 		let mark_price1 = MultiplePrices {
// 			market_id: market1.market.id,
// 			index_price: 150.into(),
// 			mark_price: 151.into(),
// 		};
// 		let mark_price2 = MultiplePrices {
// 			market_id: market2.market.id,
// 			index_price: 230.into(),
// 			mark_price: 229.into(),
// 		};
// 		prices.push(mark_price1);
// 		prices.push(mark_price2);
// 		assert_ok!(PricesModule::update_prices(
// 			RuntimeOrigin::signed(1),
// 			prices.clone(),
// 			1702359661000
// 		));

// 		let price = PricesModule::current_price(market1.market.id);
// 		assert_eq!(FixedI128::from_u32(151), price.mark_price);
// 		assert_eq!(FixedI128::from_u32(150), price.index_price);

// 		let price = PricesModule::current_price(market2.market.id);
// 		assert_eq!(FixedI128::from_u32(229), price.mark_price);
// 		assert_eq!(FixedI128::from_u32(230), price.index_price);

// 		let historical_price = PricesModule::historical_price(1702359661, market1.market.id);
// 		assert_eq!(FixedI128::from_u32(151), historical_price.mark_price);
// 		assert_eq!(FixedI128::from_u32(150), historical_price.index_price);

// 		let historical_price = PricesModule::historical_price(1702359661, market2.market.id);
// 		assert_eq!(FixedI128::from_u32(229), historical_price.mark_price);
// 		assert_eq!(FixedI128::from_u32(230), historical_price.index_price);

// 		let timestamps = PricesModule::price_timestamps();
// 		assert_eq!(vec![1702359600, 1702359661], timestamps);

// 		let last_timestamp = PricesModule::last_timestamp();
// 		assert_eq!(1702359661, last_timestamp);
// 	});

#[test]
fn test_abr_calculation() {
	// Get a test environment
	let mut env = setup();

	env.execute_with(|| {
		let (mark_prices, index_prices) = get_data();
		let result = PricesModule::calculate_abr(
			mark_prices,
			index_prices,
			convert_to_fixed(vec![0.000025_f64])[0],
			convert_to_fixed(vec![1.5])[0],
			8_usize,
		);
		print!("the result is {:?}", result);
	});
}
