use crate::{mock::*, Event};
use frame_support::assert_ok;
use sp_arithmetic::FixedI128;
use zkx_support::test_helpers::asset_helper::{eth, link, usdc};
use zkx_support::test_helpers::market_helper::{eth_usdc, link_usdc};
use zkx_support::types::{Asset, Market, MultiplePrices, Price};

fn setup() -> (Market, Market) {
	assert_ok!(Timestamp::set(None.into(), 100));
	let assets: Vec<Asset> = vec![eth(), usdc(), link()];
	assert_ok!(AssetModule::replace_all_assets(RuntimeOrigin::signed(1), assets));

	(eth_usdc(), link_usdc())
}

#[test]
#[should_panic(expected = "MarketNotFound")]
fn test_update_market_prices_with_invalid_market_id() {
	new_test_ext().execute_with(|| {
		let (market1, _) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.
		let markets: Vec<Market> = vec![market1.clone()];
		assert_ok!(MarketModule::replace_all_markets(RuntimeOrigin::signed(1), markets));
		let mut market_prices: Vec<MultiplePrices> = Vec::new();
		let market_price1 = MultiplePrices { market_id: 0, price: 1000.into() };
		market_prices.push(market_price1);

		assert_ok!(MarketPricesModule::update_market_prices(
			RuntimeOrigin::signed(1),
			market_prices
		));
	});
}

#[test]
#[should_panic(expected = "InvalidMarketPrice")]
fn test_update_market_prices_with_invalid_market_price() {
	new_test_ext().execute_with(|| {
		let (market1, _) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.
		let markets: Vec<Market> = vec![market1.clone()];
		assert_ok!(MarketModule::replace_all_markets(RuntimeOrigin::signed(1), markets));
		let mut market_prices: Vec<MultiplePrices> = Vec::new();
		let market_price1 = MultiplePrices { market_id: market1.id, price: (-100).into() };
		market_prices.push(market_price1);

		assert_ok!(MarketPricesModule::update_market_prices(
			RuntimeOrigin::signed(1),
			market_prices
		));
	});
}

#[test]
fn test_update_market_prices() {
	new_test_ext().execute_with(|| {
		let (market1, market2) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.
		let markets: Vec<Market> = vec![market1.clone(), market2.clone()];
		assert_ok!(MarketModule::replace_all_markets(RuntimeOrigin::signed(1), markets));
		let mut market_prices: Vec<MultiplePrices> = Vec::new();
		let market_price1 = MultiplePrices { market_id: market1.id, price: 1000.into() };
		let market_price2 = MultiplePrices { market_id: market2.id, price: 2000.into() };
		market_prices.push(market_price1);
		market_prices.push(market_price2);
		assert_ok!(MarketPricesModule::update_market_prices(
			RuntimeOrigin::signed(1),
			market_prices.clone()
		));

		let market_price: Price = MarketPricesModule::market_price(market1.id);
		let expected_price: FixedI128 = 1000.into();
		assert_eq!(expected_price, market_price.price);

		let market_price = MarketPricesModule::market_price(market2.id);
		let expected_price: FixedI128 = 2000.into();
		assert_eq!(expected_price, market_price.price);

		// Assert that the correct event was deposited
		System::assert_last_event(Event::MultipleMarketPricesUpdated { market_prices }.into());
	});
}

#[test]
fn test_update_index_prices() {
	new_test_ext().execute_with(|| {
		let (market1, market2) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.
		let markets: Vec<Market> = vec![market1.clone(), market2.clone()];
		assert_ok!(MarketModule::replace_all_markets(RuntimeOrigin::signed(1), markets));
		let mut index_prices: Vec<MultiplePrices> = Vec::new();
		let index_price1 = MultiplePrices { market_id: market1.id, price: 100.into() };
		let index_price2 = MultiplePrices { market_id: market2.id, price: 200.into() };
		index_prices.push(index_price1);
		index_prices.push(index_price2);
		assert_ok!(MarketPricesModule::update_index_prices(
			RuntimeOrigin::signed(1),
			index_prices.clone()
		));

		let index_price: Price = MarketPricesModule::index_price(market1.id);
		let expected_price: FixedI128 = 100.into();
		assert_eq!(expected_price, index_price.price);

		let index_price = MarketPricesModule::index_price(market2.id);
		let expected_price: FixedI128 = 200.into();
		assert_eq!(expected_price, index_price.price);

		// Assert that the correct event was deposited
		System::assert_last_event(Event::MultipleIndexPricesUpdated { index_prices }.into());
	});
}
