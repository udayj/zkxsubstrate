use crate::{mock::*, Event};
use frame_support::assert_ok;
use zkx_support::test_helpers::asset_helper::{eth, link, usdc};
use zkx_support::test_helpers::market_helper::{eth_usdc, link_usdc};
use zkx_support::types::Market;

fn setup() -> (sp_io::TestExternalities, Vec<Market>) {
	// Create a new test environment
	let mut env = new_test_ext();

	// Set the block number in the environment
	env.execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Assets::replace_all_assets(
			RuntimeOrigin::signed(1),
			vec![eth(), usdc(), link()]
		));
	});

	(env.into(), vec![eth_usdc(), link_usdc()])
}

#[test]
fn it_works_for_replace_markets() {
	let (mut env, markets) = setup();
	let eth_usdc_market = &markets[0];

	env.execute_with(|| {
		// Set eth_usdc as a market
		assert_ok!(MarketModule::replace_all_markets(
			RuntimeOrigin::signed(1),
			vec![eth_usdc_market.clone()]
		));

		// Check the state
		assert_eq!(MarketModule::markets_count(), 1);
		assert_eq!(MarketModule::markets(eth_usdc_market.id).unwrap(), eth_usdc_market.clone());

		// Assert that the correct event was deposited
		System::assert_last_event(Event::MarketsCreated { length: 1 }.into());
	});
}

#[test]
fn it_works_for_replace_markets_multiple_markets() {
	let (mut env, markets) = setup();
	let eth_usdc_market = &markets[0];
	let link_usdc_market = &markets[1];

	env.execute_with(|| {
		// Set eth_usdc and link_usdc as markets
		assert_ok!(MarketModule::replace_all_markets(
			RuntimeOrigin::signed(1),
			vec![eth_usdc_market.clone(), link_usdc_market.clone()]
		));

		// Check the state
		assert_eq!(MarketModule::markets_count(), 2);
		assert_eq!(MarketModule::markets(eth_usdc_market.id).unwrap(), eth_usdc_market.clone());
		assert_eq!(MarketModule::markets(link_usdc_market.id).unwrap(), link_usdc_market.clone());
	});
}

#[test]
#[should_panic(expected = "DuplicateMarket")]
fn it_does_not_work_for_replace_markets_duplicate() {
	let (mut env, markets) = setup();
	let eth_usdc_market = &markets[0];

	env.execute_with(|| {
		// Try to set eth_usdc as market, twice
		let markets: Vec<Market> = vec![eth_usdc_market.clone(), eth_usdc_market.clone()];
		assert_ok!(MarketModule::replace_all_markets(RuntimeOrigin::signed(1), markets));
	});
}

#[test]
#[should_panic(expected = "InvalidMarket")]
fn it_does_not_work_for_replace_markets_zero_id() {
	let (mut env, markets) = setup();
	let eth_usdc_market = &markets[0].clone().set_id(0);

	env.execute_with(|| {
		// Try to set eth_usdc with 0 id
		assert_ok!(MarketModule::replace_all_markets(
			RuntimeOrigin::signed(1),
			vec![eth_usdc_market.clone()]
		));
	});
}

#[test]
#[should_panic(expected = "AssetNotFound")]
fn it_does_not_work_for_replace_markets_invalid_asset() {
	let (mut env, markets) = setup();
	let eth_usdc_market = &markets[0].clone().set_asset(12345);

	env.execute_with(|| {
		// Try to set a market with invalid asset
		assert_ok!(MarketModule::replace_all_markets(
			RuntimeOrigin::signed(1),
			vec![eth_usdc_market.clone()]
		));
	});
}

#[test]
#[should_panic(expected = "AssetNotFound")]
fn it_does_not_work_for_replace_markets_invalid_asset_collateral() {
	let (mut env, markets) = setup();
	let eth_usdc_market = &markets[0].clone().set_asset_collateral(12345);

	env.execute_with(|| {
		// Try to set a market with invalid collateral
		assert_ok!(MarketModule::replace_all_markets(
			RuntimeOrigin::signed(1),
			vec![eth_usdc_market.clone()]
		));
	});
}

#[test]
#[should_panic(expected = "AssetNotCollateral")]
fn it_does_not_work_for_replace_markets_not_collateral() {
	let (mut env, markets) = setup();
	let eth_link_market = &markets[0].clone().set_asset_collateral(link().id);

	env.execute_with(|| {
		// Try to set a market with invalid collateral
		assert_ok!(MarketModule::replace_all_markets(
			RuntimeOrigin::signed(1),
			vec![eth_link_market.clone()]
		));
	});
}

#[test]
#[should_panic(expected = "InvalidLeverage")]
fn it_does_not_work_for_replace_markets_invalid_max_leverage() {
	let (mut env, markets) = setup();
	let eth_usdc_market =
		&markets[0].clone().set_maximum_leverage(4.into()).set_minimum_leverage(5.into());

	env.execute_with(|| {
		// Try to set a market with invalid collateral
		assert_ok!(MarketModule::replace_all_markets(
			RuntimeOrigin::signed(1),
			vec![eth_usdc_market.clone()]
		));
	});
}

#[test]
#[should_panic(expected = "InvalidLeverage")]
fn it_does_not_work_for_replace_markets_invalid_current_leverage() {
	let (mut env, markets) = setup();
	let eth_usdc_market = &markets[0].clone().set_currently_allowed_leverage(11.into());

	env.execute_with(|| {
		// Try to set a market with invalid current leverage
		assert_ok!(MarketModule::replace_all_markets(
			RuntimeOrigin::signed(1),
			vec![eth_usdc_market.clone()]
		));
	});
}

#[test]
fn test_add_market() {
	let (mut env, markets) = setup();
	let eth_usdc_market = &markets[0];

	env.execute_with(|| {
		// Set eth usd as market
		assert_ok!(MarketModule::add_market(RuntimeOrigin::signed(1), eth_usdc_market.clone()));
		assert_eq!(MarketModule::markets_count(), 1);
	});
}

#[test]
#[should_panic(expected = "DuplicateMarket")]
fn test_add_market_with_duplicate_market() {
	let (mut env, markets) = setup();
	let eth_usdc_market = &markets[0];

	env.execute_with(|| {
		// Try to set the market twice
		assert_ok!(MarketModule::add_market(RuntimeOrigin::signed(1), eth_usdc_market.clone()));
		assert_ok!(MarketModule::add_market(RuntimeOrigin::signed(1), eth_usdc_market.clone()));
	});
}

#[test]
#[should_panic(expected = "InvalidMarket")]
fn test_add_market_with_zero_id() {
	let (mut env, markets) = setup();
	let eth_usdc_market = &markets[0].clone().set_id(0);

	env.execute_with(|| {
		// Try to set a market with 0 id
		assert_ok!(MarketModule::add_market(RuntimeOrigin::signed(1), eth_usdc_market.clone()));
	});
}

#[test]
#[should_panic(expected = "AssetNotFound")]
fn test_add_market_with_invalid_asset() {
	let (mut env, markets) = setup();
	let eth_usdc_market = &markets[0].clone().set_asset(12345);

	env.execute_with(|| {
		// Try to set a market with invalid asset
		assert_ok!(MarketModule::add_market(RuntimeOrigin::signed(1), eth_usdc_market.clone()));
	});
}

#[test]
#[should_panic(expected = "AssetNotCollateral")]
fn test_add_market_with_asset_not_collateral() {
	let (mut env, markets) = setup();
	let eth_link_market = &markets[0].clone().set_asset_collateral(link().id);

	env.execute_with(|| {
		// Try to set an invalid collateral
		assert_ok!(MarketModule::add_market(RuntimeOrigin::signed(1), eth_link_market.clone()));
	});
}

#[test]
#[should_panic(expected = "AssetNotFound")]
fn test_add_market_with_invalid_asset_collateral() {
	let (mut env, markets) = setup();
	let eth_usdc_market = &markets[0].clone().set_asset_collateral(12345);

	env.execute_with(|| {
		// Try to set a market with invalid collateral
		assert_ok!(MarketModule::add_market(RuntimeOrigin::signed(1), eth_usdc_market.clone()));
	});
}

#[test]
#[should_panic(expected = "InvalidLeverage")]
fn test_add_market_with_invalid_max_leverage() {
	let (mut env, markets) = setup();
	let eth_usdc_market =
		&markets[0].clone().set_maximum_leverage(4.into()).set_minimum_leverage(5.into());

	env.execute_with(|| {
		// Try to set a market with invalid collateral
		assert_ok!(MarketModule::add_market(RuntimeOrigin::signed(1), eth_usdc_market.clone()));
	});
}

#[test]
#[should_panic(expected = "InvalidLeverage")]
fn test_add_market_with_invalid_current_leverage() {
	let (mut env, markets) = setup();
	let eth_usdc_market = &markets[0].clone().set_currently_allowed_leverage(11.into());

	env.execute_with(|| {
		// Try to set a market with invalid current leverage
		assert_ok!(MarketModule::add_market(RuntimeOrigin::signed(1), eth_usdc_market.clone()));
	});
}

#[test]
fn test_update_market() {
	let (mut env, markets) = setup();
	let eth_usdc_market = &markets[0];
	let eth_usdc_market_updated = &markets[0].clone().set_is_tradable(false);

	env.execute_with(|| {
		// Set the eth_usdc market
		assert_ok!(MarketModule::add_market(RuntimeOrigin::signed(1), eth_usdc_market.clone()));

		// Update the eth_usdc market
		assert_ok!(MarketModule::update_market(
			RuntimeOrigin::signed(1),
			eth_usdc_market_updated.clone()
		));
		assert_eq!(
			MarketModule::markets(eth_usdc_market_updated.id).unwrap(),
			eth_usdc_market_updated.clone()
		);
	});
}

#[test]
fn test_remove_market() {
	let (mut env, markets) = setup();
	let eth_usdc_market = &markets[0];

	env.execute_with(|| {
		// Add eth_usdc market
		assert_ok!(MarketModule::add_market(RuntimeOrigin::signed(1), eth_usdc_market.clone()));

		// Check the state
		assert_eq!(MarketModule::markets_count(), 1);

		// Remove the market
		assert_ok!(MarketModule::remove_market(RuntimeOrigin::signed(1), eth_usdc_market.id));

		// Check the state
		assert_eq!(MarketModule::markets_count(), 0);
	});
}

#[test]
#[should_panic(expected = "InvalidMarket")]
fn test_remove_market_with_already_removed_market_id() {
	let (mut env, markets) = setup();
	let eth_usdc_market = &markets[0];

	env.execute_with(|| {
		// Add market and remove it twice
		assert_ok!(MarketModule::add_market(RuntimeOrigin::signed(1), eth_usdc_market.clone()));
		assert_ok!(MarketModule::remove_market(RuntimeOrigin::signed(1), eth_usdc_market.id));
		assert_ok!(MarketModule::remove_market(RuntimeOrigin::signed(1), eth_usdc_market.id));
	});
}
