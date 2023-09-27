use crate::{mock::*, Event};
use frame_support::assert_ok;
use zkx_support::types::Market;
use zkx_support::test_helpers::asset_helper::{eth, usdc, link};
use zkx_support::test_helpers::market_helper::{eth_usdc, link_usdc};


fn setup() -> (Market, Market) {
	assert_ok!(Assets::replace_all_assets(RuntimeOrigin::signed(1), vec![eth(), usdc(), link()]));
	(eth_usdc(), link_usdc())
}

#[test]
fn it_works_for_replace_markets() {
	new_test_ext().execute_with(|| {
		let (market1, _) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.
		let markets: Vec<Market> = vec![market1.clone()];
		assert_ok!(MarketModule::replace_all_markets(RuntimeOrigin::signed(1), markets));

		assert_eq!(MarketModule::markets_count(), 1);
		let market_storage = MarketModule::markets(1);
		assert_eq!(market_storage.unwrap(), market1);

		// Assert that the correct event was deposited
		System::assert_last_event(Event::MarketsCreated { length: 1 }.into());
	});
}

#[test]
fn it_works_for_replace_markets_multiple_markets() {
	new_test_ext().execute_with(|| {
		let (market1, market2) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.
		let markets: Vec<Market> = vec![market1.clone(), market2.clone()];
		assert_ok!(MarketModule::replace_all_markets(RuntimeOrigin::signed(1), markets));

		assert_eq!(MarketModule::markets_count(), 2);
		let market_storage1 = MarketModule::markets(1);
		assert_eq!(market_storage1.unwrap(), market1);
		let market_storage2 = MarketModule::markets(2);
		assert_eq!(market_storage2.unwrap(), market2);
	});
}

#[test]
#[should_panic(expected = "DuplicateMarket")]
fn it_does_not_work_for_replace_markets_duplicate() {
	new_test_ext().execute_with(|| {
		let (market1, _) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.
		let markets: Vec<Market> = vec![market1.clone(), market1.clone()];
		assert_ok!(MarketModule::replace_all_markets(RuntimeOrigin::signed(1), markets));
	});
}

#[test]
#[should_panic(expected = "InvalidMarketId")]
fn it_does_not_work_for_replace_markets_zero_id() {
	new_test_ext().execute_with(|| {
		let (market1, _) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		let market: Market = Market { id: 0, ..market1 };
		// Dispatch a signed extrinsic.
		let markets: Vec<Market> = vec![market.clone()];
		assert_ok!(MarketModule::replace_all_markets(RuntimeOrigin::signed(1), markets));
	});
}

#[test]
#[should_panic(expected = "AssetNotFound")]
fn it_does_not_work_for_replace_markets_invalid_asset() {
	new_test_ext().execute_with(|| {
		let (market1, _) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		let market: Market = Market { asset: 12345678, ..market1 };
		// Dispatch a signed extrinsic.
		let markets: Vec<Market> = vec![market.clone()];
		assert_ok!(MarketModule::replace_all_markets(RuntimeOrigin::signed(1), markets));
	});
}

#[test]
#[should_panic(expected = "AssetNotFound")]
fn it_does_not_work_for_replace_markets_invalid_asset_collateral() {
	new_test_ext().execute_with(|| {
		let (market1, _) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		let market: Market = Market { asset_collateral: 12345678, ..market1 };
		// Dispatch a signed extrinsic.
		let markets: Vec<Market> = vec![market.clone()];
		assert_ok!(MarketModule::replace_all_markets(RuntimeOrigin::signed(1), markets));
	});
}

#[test]
#[should_panic(expected = "AssetNotCollateral")]
fn it_does_not_work_for_replace_markets_not_collateral() {
	new_test_ext().execute_with(|| {
		let (market1, _) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		let market: Market = Market { asset_collateral: link().id, ..market1 };
		// Dispatch a signed extrinsic.
		let markets: Vec<Market> = vec![market.clone()];
		assert_ok!(MarketModule::replace_all_markets(RuntimeOrigin::signed(1), markets));
	});
}

#[test]
#[should_panic(expected = "InvalidLeverage")]
fn it_does_not_work_for_replace_markets_invalid_max_leverage() {
	new_test_ext().execute_with(|| {
		let (market1, _) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		let market: Market =
			Market { minimum_leverage: 5.into(), maximum_leverage: 4.into(), ..market1 };
		// Dispatch a signed extrinsic.
		let markets: Vec<Market> = vec![market.clone()];
		assert_ok!(MarketModule::replace_all_markets(RuntimeOrigin::signed(1), markets));
	});
}

#[test]
#[should_panic(expected = "InvalidLeverage")]
fn it_does_not_work_for_replace_markets_invalid_current_leverage() {
	new_test_ext().execute_with(|| {
		let (market1, _) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		let market: Market = Market { currently_allowed_leverage: 11.into(), ..market1 };
		// Dispatch a signed extrinsic.
		let markets: Vec<Market> = vec![market.clone()];
		assert_ok!(MarketModule::replace_all_markets(RuntimeOrigin::signed(1), markets));
	});
}
