use crate::{mock::*, Event};
use frame_support::assert_ok;
use primitive_types::U256;
use zkx_support::test_helpers::asset_helper::{eth, usdc, link, btc};
use zkx_support::types::Asset;

fn setup() -> (Asset, Asset, Asset, Asset) {
	(eth(), usdc(), link(), btc())
}

#[test]
fn it_works_for_replace_assets() {
	new_test_ext().execute_with(|| {
		let (asset1, _, _, _) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.
		let assets: Vec<Asset> = vec![asset1.clone()];
		assert_ok!(AssetModule::replace_all_assets(RuntimeOrigin::signed(1), assets));

		assert_eq!(AssetModule::assets_count(), 1);
		let asset_storage = AssetModule::assets(eth().id);
		assert_eq!(asset_storage.unwrap(), asset1);

		// Assert that the correct event was deposited
		System::assert_last_event(Event::AssetsCreated { length: 1 }.into());
	});
}

#[test]
fn it_works_for_replace_assets_multiple_assets() {
	new_test_ext().execute_with(|| {
		let (asset1, asset2, asset3, asset4) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.
		let assets: Vec<Asset> = vec![asset4.clone()];
		assert_ok!(AssetModule::replace_all_assets(RuntimeOrigin::signed(1), assets));
		assert_eq!(AssetModule::assets_count(), 1);

		// Perform replace assets for the second time
		let assets: Vec<Asset> = vec![asset1.clone(), asset2.clone(), asset3.clone()];
		assert_ok!(AssetModule::replace_all_assets(RuntimeOrigin::signed(1), assets));

		assert_eq!(AssetModule::assets_count(), 3);
		let asset_storage1 = AssetModule::assets(eth().id);
		assert_eq!(asset_storage1.unwrap(), asset1);
		let asset_storage2 = AssetModule::assets(usdc().id);
		assert_eq!(asset_storage2.unwrap(), asset2);
		let asset_storage3 = AssetModule::assets(link().id);
		assert_eq!(asset_storage3.unwrap(), asset3);

		// Assert that the correct event was deposited
		System::assert_last_event(Event::AssetsCreated { length: 3 }.into());
	});
}

#[test]
#[should_panic(expected = "DuplicateAsset")]
fn it_does_not_work_for_replace_assets_duplicate() {
	new_test_ext().execute_with(|| {
		let (asset1, _, _, _) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.
		let asset: Asset = eth();
		let assets: Vec<Asset> = vec![asset1.clone(), asset.clone()];
		assert_ok!(AssetModule::replace_all_assets(RuntimeOrigin::signed(1), assets));
	});
}


#[test]
#[should_panic(expected = "InvalidAsset")]
fn it_does_not_work_for_replace_assets_invalid_decimal() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.
		let asset: Asset = Asset {
			id: eth().id,
			version: 1,
			short_name: eth().short_name,
			is_tradable: false,
			is_collateral: true,
			l2_address: U256::from(100),
			decimals: 19,
		};
		let assets: Vec<Asset> = vec![asset.clone()];
		assert_ok!(AssetModule::replace_all_assets(RuntimeOrigin::signed(1), assets));
	});
}
