use crate::{mock::*, Event};
use frame_support::assert_ok;
use primitive_types::U256;
use zkx_support::types::Asset;
use zkx_support::types::Market;

fn setup() -> (Market, Market) {
	let ETH_ID: U256 = 4543560.into();
	let USDC_ID: U256 = 1431520323.into();
	let LINK_ID: U256 = 1279872587.into();
	let BTC_ID: U256 = 4346947.into();
	let name1: Vec<u8> = "ETH".into();
	let asset1: Asset = Asset {
		id: ETH_ID,
		name: name1.try_into().unwrap(),
		is_tradable: true,
		is_collateral: false,
		token_decimal: 18,
	};
	let name2: Vec<u8> = "USDC".into();
	let asset2: Asset = Asset {
		id: USDC_ID,
		name: name2.try_into().unwrap(),
		is_tradable: false,
		is_collateral: true,
		token_decimal: 6,
	};
	let name3: Vec<u8> = "LINK".into();
	let asset3: Asset = Asset {
		id: LINK_ID,
		name: name3.try_into().unwrap(),
		is_tradable: true,
		is_collateral: false,
		token_decimal: 6,
	};
	let name3: Vec<u8> = "BTC".into();
	let asset4: Asset = Asset {
		id: BTC_ID,
		name: name3.try_into().unwrap(),
		is_tradable: true,
		is_collateral: false,
		token_decimal: 6,
	};

	let assets: Vec<Asset> = vec![asset1.clone(), asset2.clone(), asset3.clone()];
	assert_ok!(Assets::replace_all_assets(RuntimeOrigin::signed(1), assets));

	let market1: Market = Market {
		id: 1.into(),
		asset: ETH_ID,
		asset_collateral: USDC_ID,
		is_tradable: 1,
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
		id: 2.into(),
		asset: LINK_ID,
		asset_collateral: USDC_ID,
		is_tradable: 0,
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
fn it_works_for_replace_markets() {
	new_test_ext().execute_with(|| {
		let (market1, _) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.
		let markets: Vec<Market> = vec![market1.clone()];
		assert_ok!(MarketModule::replace_all_markets(RuntimeOrigin::signed(1), markets));

		assert_eq!(MarketModule::markets_count(), 1);
		let market_storage = MarketModule::markets(U256::from(1_u8));
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
		let market_storage1 = MarketModule::markets(U256::from(1_u8));
		assert_eq!(market_storage1.unwrap(), market1);
		let market_storage2 = MarketModule::markets(U256::from(2_u8));
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
		let market: Market = Market { id: 0.into(), ..market1 };
		// Dispatch a signed extrinsic.
		let markets: Vec<Market> = vec![market.clone()];
		assert_ok!(MarketModule::replace_all_markets(RuntimeOrigin::signed(1), markets));
	});
}

#[test]
#[should_panic(expected = "InvalidTradableFlag")]
fn it_does_not_work_for_replace_markets_invalid_tradable_flag() {
	new_test_ext().execute_with(|| {
		let (market1, _) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		let market: Market = Market { is_tradable: 3, ..market1 };
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
		let market: Market = Market { asset: 12345678.into(), ..market1 };
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
		let market: Market = Market { asset_collateral: 12345678.into(), ..market1 };
		// Dispatch a signed extrinsic.
		let markets: Vec<Market> = vec![market.clone()];
		assert_ok!(MarketModule::replace_all_markets(RuntimeOrigin::signed(1), markets));
	});
}

#[test]
#[should_panic(expected = "AssetNotCollateral")]
fn it_does_not_work_for_replace_markets_not_collateral() {
	new_test_ext().execute_with(|| {
		let LINK_ID: U256 = 1279872587.into();
		let (market1, _) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		let market: Market = Market { asset_collateral: LINK_ID, ..market1 };
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
