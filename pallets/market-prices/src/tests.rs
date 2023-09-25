use crate::{mock::*, Event};
use frame_support::assert_ok;
use primitive_types::U256;
use sp_arithmetic::FixedI128;
use zkx_support::types::{Asset, Market, MarketPrice, MultipleMarketPrices};

fn setup() -> (Market, Market) {
	let eth_id: u128 = 4543560;
	let usdc_id: u128 = 1431520323;
	let link_id: u128 = 1279872587;
	let name1: Vec<u8> = "ETH".into();
	let asset1: Asset = Asset {
		id: eth_id,
		name: name1.try_into().unwrap(),
		is_tradable: true,
		is_collateral: false,
		token_decimal: 18,
	};
	let name2: Vec<u8> = "USDC".into();
	let asset2: Asset = Asset {
		id: usdc_id,
		name: name2.try_into().unwrap(),
		is_tradable: false,
		is_collateral: true,
		token_decimal: 6,
	};
	let name3: Vec<u8> = "LINK".into();
	let asset3: Asset = Asset {
		id: link_id,
		name: name3.try_into().unwrap(),
		is_tradable: true,
		is_collateral: false,
		token_decimal: 6,
	};
	let assets: Vec<Asset> = vec![asset1.clone(), asset2.clone(), asset3.clone()];
	assert_ok!(AssetModule::replace_all_assets(RuntimeOrigin::signed(1), assets));

	let market1: Market = Market {
		id: 1,
		asset: eth_id,
		asset_collateral: usdc_id,
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
		id: 2,
		asset: link_id,
		asset_collateral: usdc_id,
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
	(market1, market2)
}

#[test]
#[should_panic(expected = "MarketNotFound")]
fn test_update_multiple_market_prices_with_invalid_market_id() {
	new_test_ext().execute_with(|| {
		let (market1, _) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.
		let markets: Vec<Market> = vec![market1.clone()];
		assert_ok!(MarketModule::replace_all_markets(RuntimeOrigin::signed(1), markets));
		let mut market_prices: Vec<MultipleMarketPrices> = Vec::new();
		let market_price1 = MultipleMarketPrices { market_id: 0, price: 1000.into() };
		market_prices.push(market_price1);

		assert_ok!(MarketPricesModule::update_multiple_market_prices(
			RuntimeOrigin::signed(1),
			market_prices
		));
	});
}

#[test]
#[should_panic(expected = "InvalidMarketPrice")]
fn test_update_multiple_market_prices_with_invalid_market_price() {
	new_test_ext().execute_with(|| {
		let (market1, _) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.
		let markets: Vec<Market> = vec![market1.clone()];
		assert_ok!(MarketModule::replace_all_markets(RuntimeOrigin::signed(1), markets));
		let mut market_prices: Vec<MultipleMarketPrices> = Vec::new();
		let market_price1 = MultipleMarketPrices { market_id: market1.id, price: (-100).into() };
		market_prices.push(market_price1);

		assert_ok!(MarketPricesModule::update_multiple_market_prices(
			RuntimeOrigin::signed(1),
			market_prices
		));
	});
}

#[test]
fn test_update_multiple_market_prices() {
	new_test_ext().execute_with(|| {
		let (market1, market2) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.
		let markets: Vec<Market> = vec![market1.clone(), market2.clone()];
		assert_ok!(MarketModule::replace_all_markets(RuntimeOrigin::signed(1), markets));
		let mut market_prices: Vec<MultipleMarketPrices> = Vec::new();
		let market_price1 = MultipleMarketPrices { market_id: market1.id, price: 1000.into() };
		let market_price2 = MultipleMarketPrices { market_id: market2.id, price: 2000.into() };
		market_prices.push(market_price1);
		market_prices.push(market_price2);
		assert_ok!(MarketPricesModule::update_multiple_market_prices(
			RuntimeOrigin::signed(1),
			market_prices.clone()
		));

		let mut market_price: MarketPrice = MarketPricesModule::market_price(market1.id);
		let mut expected_price: FixedI128 = 1000.into();
		assert_eq!(expected_price, market_price.price);

		market_price = MarketPricesModule::market_price(market2.id);
		expected_price = 2000.into();
		assert_eq!(expected_price, market_price.price);

		// Assert that the correct event was deposited
		System::assert_last_event(Event::MultipleMarketPricesUpdated { market_prices }.into());
	});
}
