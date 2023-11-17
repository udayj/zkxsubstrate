use crate::{mock::*, Event};
use frame_support::assert_ok;
use pallet_support::{
	test_helpers::{
		asset_helper::{eth, link, usdc},
		market_helper::{eth_usdc, link_usdc},
	},
	types::{ExtendedMarket, MultiplePrices},
};
use sp_arithmetic::FixedI128;

fn setup() -> (ExtendedMarket, ExtendedMarket) {
	assert_ok!(Timestamp::set(None.into(), 1699940367000));
	let assets = vec![eth(), usdc(), link()];
	assert_ok!(AssetModule::replace_all_assets(RuntimeOrigin::signed(1), assets));

	(eth_usdc(), link_usdc())
}

#[test]
fn test_update_prices() {
	new_test_ext().execute_with(|| {
		let (market1, market2) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.
		let markets = vec![market1.clone(), market2.clone()];
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
		assert_ok!(PricesModule::update_prices(RuntimeOrigin::signed(1), prices.clone()));

		let price = PricesModule::current_price(market1.market.id);
		assert_eq!(FixedI128::from_u32(100), price.mark_price);
		assert_eq!(FixedI128::from_u32(102), price.index_price);

		let price = PricesModule::current_price(market2.market.id);
		assert_eq!(FixedI128::from_u32(200), price.mark_price);
		assert_eq!(FixedI128::from_u32(199), price.index_price);

		// Assert that the correct event was deposited
		System::assert_last_event(Event::MultiplePricesUpdated { prices }.into());
	});
}

#[test]
fn test_historical_prices() {
	new_test_ext().execute_with(|| {
		let (market1, market2) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		Timestamp::set_timestamp(1702359600000);

		let markets = vec![market1.clone(), market2.clone()];
		assert_ok!(MarketModule::replace_all_markets(RuntimeOrigin::signed(1), markets));

		assert_ok!(PricesModule::update_price_interval(RuntimeOrigin::signed(1), 60000));

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
		assert_ok!(PricesModule::update_prices(RuntimeOrigin::signed(1), prices.clone()));

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

		let price_interval = PricesModule::price_interval();
		assert_eq!(60, price_interval);

		let timestamps = PricesModule::price_timestamps();
		assert_eq!(vec![1702359600], timestamps);

		let last_timestamp = PricesModule::last_timestamp();
		assert_eq!(1702359600, last_timestamp);

		// Set timestamp such that historical price will not get updated
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
		assert_ok!(PricesModule::update_prices(RuntimeOrigin::signed(1), prices.clone()));

		let price = PricesModule::current_price(market1.market.id);
		assert_eq!(FixedI128::from_u32(109), price.mark_price);
		assert_eq!(FixedI128::from_u32(110), price.index_price);

		let price = PricesModule::current_price(market2.market.id);
		assert_eq!(FixedI128::from_u32(192), price.mark_price);
		assert_eq!(FixedI128::from_u32(190), price.index_price);

		let historical_price = PricesModule::historical_price(1702359620, market1.market.id);
		assert_eq!(FixedI128::from_u32(0), historical_price.mark_price);
		assert_eq!(FixedI128::from_u32(0), historical_price.index_price);

		let historical_price = PricesModule::historical_price(1702359620, market2.market.id);
		assert_eq!(FixedI128::from_u32(0), historical_price.mark_price);
		assert_eq!(FixedI128::from_u32(0), historical_price.index_price);

		let timestamps = PricesModule::price_timestamps();
		assert_eq!(vec![1702359600], timestamps);

		let last_timestamp = PricesModule::last_timestamp();
		assert_eq!(1702359600, last_timestamp);

		// Set timestamp such that historical price will get updated
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
		assert_ok!(PricesModule::update_prices(RuntimeOrigin::signed(1), prices.clone()));

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

		let timestamps = PricesModule::price_timestamps();
		assert_eq!(vec![1702359600, 1702359661], timestamps);

		let last_timestamp = PricesModule::last_timestamp();
		assert_eq!(1702359661, last_timestamp);
	});
}
