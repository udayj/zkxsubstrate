use crate::{mock::*, Event};
use frame_support::assert_ok;
use sp_arithmetic::FixedI128;
use zkx_support::{
	test_helpers::{
		asset_helper::{eth, link, usdc},
		market_helper::{eth_usdc, link_usdc},
	},
	types::{ExtendedMarket, MultiplePrices, Price},
};

fn setup() -> (ExtendedMarket, ExtendedMarket) {
	assert_ok!(Timestamp::set(None.into(), 100));
	let assets = vec![eth(), usdc(), link()];
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
		let markets = vec![market1.clone()];
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
		let markets = vec![market1.clone()];
		assert_ok!(MarketModule::replace_all_markets(RuntimeOrigin::signed(1), markets));
		let mut market_prices: Vec<MultiplePrices> = Vec::new();
		let market_price1 = MultiplePrices { market_id: market1.market.id, price: (-100).into() };
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
		let markets = vec![market1.clone(), market2.clone()];
		assert_ok!(MarketModule::replace_all_markets(RuntimeOrigin::signed(1), markets));
		let mut market_prices: Vec<MultiplePrices> = Vec::new();
		let market_price1 = MultiplePrices { market_id: market1.market.id, price: 1000.into() };
		let market_price2 = MultiplePrices { market_id: market2.market.id, price: 2000.into() };
		market_prices.push(market_price1);
		market_prices.push(market_price2);
		assert_ok!(MarketPricesModule::update_market_prices(
			RuntimeOrigin::signed(1),
			market_prices.clone()
		));

		let market_price: Price = MarketPricesModule::market_price(market1.market.id);
		let expected_price: FixedI128 = 1000.into();
		assert_eq!(expected_price, market_price.price);

		let market_price = MarketPricesModule::market_price(market2.market.id);
		let expected_price: FixedI128 = 2000.into();
		assert_eq!(expected_price, market_price.price);

		// Assert that the correct event was deposited
		System::assert_last_event(Event::MultipleMarketPricesUpdated { market_prices }.into());
	});
}

#[test]
fn test_update_mark_prices() {
	new_test_ext().execute_with(|| {
		let (market1, market2) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.
		let markets = vec![market1.clone(), market2.clone()];
		assert_ok!(MarketModule::replace_all_markets(RuntimeOrigin::signed(1), markets));
		let mut mark_prices: Vec<MultiplePrices> = Vec::new();
		let mark_price1 = MultiplePrices { market_id: market1.market.id, price: 100.into() };
		let mark_price2 = MultiplePrices { market_id: market2.market.id, price: 200.into() };
		mark_prices.push(mark_price1);
		mark_prices.push(mark_price2);
		assert_ok!(MarketPricesModule::update_mark_prices(
			RuntimeOrigin::signed(1),
			mark_prices.clone()
		));

		let mark_price: Price = MarketPricesModule::mark_price(market1.market.id);
		let expected_price: FixedI128 = 100.into();
		assert_eq!(expected_price, mark_price.price);

		let mark_price = MarketPricesModule::mark_price(market2.market.id);
		let expected_price: FixedI128 = 200.into();
		assert_eq!(expected_price, mark_price.price);

		// Assert that the correct event was deposited
		System::assert_last_event(Event::MultipleMarkPricesUpdated { mark_prices }.into());
	});
}
