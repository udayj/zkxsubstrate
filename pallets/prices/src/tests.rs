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
	assert_ok!(Timestamp::set(None.into(), 100));
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
		assert_ok!(MarketPricesModule::update_prices(RuntimeOrigin::signed(1), prices.clone()));

		let price = MarketPricesModule::current_price(market1.market.id);
		assert_eq!(FixedI128::from_u32(100), price.mark_price);
		assert_eq!(FixedI128::from_u32(102), price.index_price);

		let price = MarketPricesModule::current_price(market2.market.id);
		assert_eq!(FixedI128::from_u32(200), price.mark_price);
		assert_eq!(FixedI128::from_u32(199), price.index_price);

		// Assert that the correct event was deposited
		System::assert_last_event(Event::MultiplePricesUpdated { prices }.into());
	});
}
