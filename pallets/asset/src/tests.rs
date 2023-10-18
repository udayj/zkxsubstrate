use crate::{mock::*, Event};
use frame_support::assert_ok;
use zkx_support::test_helpers::asset_helper::{btc, eth, link, usdc};
use zkx_support::types::ExtendedAsset;

fn setup() -> (sp_io::TestExternalities, Vec<ExtendedAsset>) {
	// Create a new test environment
	let mut env = new_test_ext();

	// Set the block number in the environment
	env.execute_with(|| {
		System::set_block_number(1);
	});

	(env.into(), vec![eth(), usdc(), link(), btc()])
}

#[test]
fn it_works_for_replace_assets() {
	let (mut env, assets) = setup();
	let eth_asset = &assets[0];

	env.execute_with(|| {
		// Set eth as an asset
		assert_ok!(AssetModule::replace_all_assets(
			RuntimeOrigin::signed(1),
			vec![eth_asset.clone()]
		));

		// Check the state
		assert_eq!(AssetModule::assets_count(), 1);
		assert_eq!(AssetModule::assets(eth_asset.asset.id).unwrap(), eth_asset.clone());

		// Assert that the correct event was deposited
		System::assert_last_event(Event::AssetsCreated { length: 1 }.into());
	});
}

#[test]
fn it_works_for_replace_assets_multiple_assets() {
	let (mut env, assets) = setup();
	let eth_asset = &assets[0];
	let link_asset = &assets[1];
	let usdc_asset = &assets[2];
	let btc_asset = &assets[3];

	env.execute_with(|| {
		// Set btc as an asset
		assert_ok!(AssetModule::replace_all_assets(
			RuntimeOrigin::signed(1),
			vec![btc_asset.clone()]
		));

		// Check the state
		assert_eq!(AssetModule::assets_count(), 1);

		// Set the rest of the assets
		assert_ok!(AssetModule::replace_all_assets(RuntimeOrigin::signed(1), assets[..3].to_vec()));

		// Check the state
		assert_eq!(AssetModule::assets_count(), 3);
		assert_eq!(AssetModule::assets(eth_asset.asset.id).unwrap(), eth_asset.clone());
		assert_eq!(AssetModule::assets(link_asset.asset.id).unwrap(), link_asset.clone());
		assert_eq!(AssetModule::assets(usdc_asset.asset.id).unwrap(), usdc_asset.clone());

		// Assert that the correct event was deposited
		System::assert_last_event(Event::AssetsCreated { length: 3 }.into());
	});
}

#[test]
#[should_panic(expected = "DuplicateAsset")]
fn it_does_not_work_for_replace_assets_duplicate() {
	let (mut env, assets) = setup();
	let eth_asset = &assets[0];

	env.execute_with(|| {
		// Set eth as asset twice
		assert_ok!(AssetModule::replace_all_assets(
			RuntimeOrigin::signed(1),
			vec![eth_asset.clone(), eth_asset.clone()]
		));
	});
}

#[test]
#[should_panic(expected = "InvalidAsset")]
fn it_does_not_work_for_replace_assets_invalid_decimal() {
	let (mut env, assets) = setup();
	let invalid_eth_asset = &assets[0].clone().set_decimals(19);

	env.execute_with(|| {
		// Try to set invalid eth asset
		assert_ok!(AssetModule::replace_all_assets(
			RuntimeOrigin::signed(1),
			vec![invalid_eth_asset.clone()]
		));
	});
}

#[test]
fn test_add_asset() {
	let (mut env, assets) = setup();
	let eth_asset = &assets[0];

	env.execute_with(|| {
		// Set eth as asset
		assert_ok!(AssetModule::add_asset(RuntimeOrigin::signed(1), eth_asset.clone()));

		// Check the state
		assert_eq!(AssetModule::assets_count(), 1);
		assert_eq!(AssetModule::assets(eth_asset.asset.id).unwrap(), eth_asset.clone());
	});
}

#[test]
#[should_panic(expected = "DuplicateAsset")]
fn test_add_duplicate_asset() {
	let (mut env, assets) = setup();
	let eth_asset = &assets[0];

	env.execute_with(|| {
		// Set eth as an asset
		assert_ok!(AssetModule::add_asset(RuntimeOrigin::signed(1), eth_asset.clone()));

		// Set eth as an asset again
		assert_ok!(AssetModule::add_asset(RuntimeOrigin::signed(1), eth_asset.clone()));
	});
}

#[test]
#[should_panic(expected = "InvalidAsset")]
fn test_add_asset_with_invalid_decimal() {
	let (mut env, assets) = setup();
	let invalid_eth_asset = &assets[0].clone().set_decimals(19);

	env.execute_with(|| {
		// Seth invalid eth as an asset
		assert_ok!(AssetModule::add_asset(RuntimeOrigin::signed(1), invalid_eth_asset.clone()));
	});
}

#[test]
fn test_update_asset() {
	let (mut env, assets) = setup();
	let eth_asset = &assets[0];
	let modified_eth_asset = &assets[0].clone().set_is_tradable(false);

	env.execute_with(|| {
		// Set eth as an asset
		assert_ok!(AssetModule::add_asset(RuntimeOrigin::signed(1), eth_asset.clone()));

		// Update the set eth asset
		assert_ok!(AssetModule::update_asset(RuntimeOrigin::signed(1), modified_eth_asset.clone()));

		// Check the state
		assert_eq!(AssetModule::assets(eth_asset.asset.id).unwrap(), modified_eth_asset.clone());
	});
}

#[test]
#[should_panic(expected = "InvalidAsset")]
fn test_update_asset_invalid_decimals() {
	let (mut env, assets) = setup();
	let eth_asset = &assets[0];
	let modified_invalid_eth_asset = &assets[0].clone().set_decimals(19);

	env.execute_with(|| {
		// Set eth as an asset
		assert_ok!(AssetModule::add_asset(RuntimeOrigin::signed(1), eth_asset.clone()));

		// Update the set eth asset
		assert_ok!(AssetModule::update_asset(
			RuntimeOrigin::signed(1),
			modified_invalid_eth_asset.clone()
		));
	});
}

#[test]
fn test_remove_asset() {
	let (mut env, assets) = setup();
	let eth_asset = &assets[0];

	env.execute_with(|| {
		// Set eth as an asset
		assert_ok!(AssetModule::add_asset(RuntimeOrigin::signed(1), eth_asset.clone()));

		// Check the state
		assert_eq!(AssetModule::assets_count(), 1);

		// Remove the eth aseet
		assert_ok!(AssetModule::remove_asset(RuntimeOrigin::signed(1), eth_asset.asset.id));

		// Check the state again
		assert_eq!(AssetModule::assets_count(), 0);
	});
}

#[test]
#[should_panic(expected = "InvalidAsset")]
fn test_remove_already_removed_asset() {
	let (mut env, assets) = setup();
	let eth_asset = &assets[0];

	env.execute_with(|| {
		// Set eth as asset
		assert_ok!(AssetModule::add_asset(RuntimeOrigin::signed(1), eth_asset.clone()));

		// Check state
		assert_eq!(AssetModule::assets_count(), 1);

		// Remove eth asset
		assert_ok!(AssetModule::remove_asset(RuntimeOrigin::signed(1), eth_asset.asset.id));

		// Check state again
		assert_eq!(AssetModule::assets_count(), 0);

		// Try to remove eth asset again
		assert_ok!(AssetModule::remove_asset(RuntimeOrigin::signed(1), eth_asset.asset.id));
	});
}
