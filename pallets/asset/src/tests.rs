use crate::{mock::*, Event};
use frame_support::{assert_err, assert_ok};
use primitive_types::U256;
use zkx_support::test_helpers::asset_helper::{btc, eth, link, usdc};
use zkx_support::traits::AssetInterface;
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

#[test]
fn test_add_asset() {
	new_test_ext().execute_with(|| {
		let (asset1, _, _, _) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		// Dispatch a signed extrinsic.
		assert_ok!(AssetModule::add_asset_admin(RuntimeOrigin::root(), asset1));
		let count = AssetModule::assets_count();
		assert_eq!(count, 1);
	});
}

#[test]
// #[should_panic(expected = "NotAdmin")]
fn test_add_asset_unauthorized() {
	new_test_ext().execute_with(|| {
		let (asset1, _, _, _) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		// Dispatch a signed extrinsic.
		let _res = AssetModule::add_asset_admin(RuntimeOrigin::signed(1), asset1);
	});
}

#[test]
#[should_panic(expected = "DuplicateAsset")]
fn test_add_duplicate_asset() {
	new_test_ext().execute_with(|| {
		let (asset1, _, _, _) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.
		assert_ok!(AssetModule::add_asset_admin(RuntimeOrigin::root(), asset1.clone()));
		// Add the same asset again
		assert_ok!(AssetModule::add_asset_admin(RuntimeOrigin::root(), asset1));
	});
}

#[test]
#[should_panic(expected = "InvalidAsset")]
fn test_add_asset_with_invalid_decimal() {
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
			l2_address: U256::from(104),
			decimals: 19,
		};
		assert_ok!(AssetModule::add_asset_admin(RuntimeOrigin::root(), asset));
	});
}

#[test]
fn test_update_asset() {
	new_test_ext().execute_with(|| {
		let (asset1, _, _, _) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		// Dispatch a signed extrinsic.
		assert_ok!(AssetModule::add_asset_admin(RuntimeOrigin::root(), asset1.clone()));

		let updated_asset = Asset {
			id: asset1.id,
			version: asset1.version,
			short_name: asset1.short_name,
			is_tradable: false,
			is_collateral: asset1.is_collateral,
			l2_address: asset1.l2_address,
			decimals: asset1.decimals,
		};

		// Update the asset
		assert_ok!(AssetModule::update_asset_admin(RuntimeOrigin::root(), updated_asset.clone()));
		assert_eq!(AssetModule::get_asset(updated_asset.id).unwrap(), updated_asset);
	});
}

#[test]
#[should_panic(expected = "NotAdmin")]
fn test_update_asset_unauthorized() {
	new_test_ext().execute_with(|| {
		let (asset1, _, _, _) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		// Dispatch a signed extrinsic.
		assert_ok!(AssetModule::add_asset_admin(RuntimeOrigin::root(), asset1.clone()));

		let updated_asset = Asset {
			id: asset1.id,
			version: asset1.version,
			short_name: asset1.short_name,
			is_tradable: false,
			is_collateral: asset1.is_collateral,
			l2_address: asset1.l2_address,
			decimals: asset1.decimals,
		};

		// Update the asset
		assert_ok!(AssetModule::update_asset_admin(RuntimeOrigin::signed(1), updated_asset.clone()));
	});
}

#[test]
#[should_panic(expected = "InvalidAsset")]
fn test_update_asset_invalid_decimals() {
	new_test_ext().execute_with(|| {
		let (asset1, _, _, _) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		// Dispatch a signed extrinsic.
		assert_ok!(AssetModule::add_asset_admin(RuntimeOrigin::root(), asset1.clone()));

		let updated_asset = Asset {
			id: asset1.id,
			version: asset1.version,
			short_name: asset1.short_name,
			is_tradable: asset1.is_tradable,
			is_collateral: asset1.is_collateral,
			l2_address: asset1.l2_address,
			decimals: 19,
		};

		// Update the asset
		assert_ok!(AssetModule::update_asset_admin(RuntimeOrigin::root(), updated_asset.clone()));
	});
}


#[test]
#[should_panic(expected = "NotAdmin")]
fn test_remove_asset_unauthorized() {
	new_test_ext().execute_with(|| {
		let (asset1, _, _, _) = setup();

		// Dispatch a signed extrinsic.
		assert_ok!(AssetModule::add_asset_admin(RuntimeOrigin::root(), asset1.clone()));
		assert_ok!(AssetModule::remove_asset_admin(RuntimeOrigin::signed(1), asset1.id));
	});
}

#[test]
fn test_remove_asset() {
	new_test_ext().execute_with(|| {
		let (asset1, _, _, _) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.
		assert_ok!(AssetModule::add_asset_admin(RuntimeOrigin::root(), asset1.clone()));
		let count = AssetModule::assets_count();
		assert_eq!(count, 1);
		assert_ok!(AssetModule::remove_asset_admin(RuntimeOrigin::root(), asset1.id));
		let count = AssetModule::assets_count();
		assert_eq!(count, 0);
	});
}

#[test]
#[should_panic(expected = "InvalidAsset")]
fn test_remove_already_removed_asset() {
	new_test_ext().execute_with(|| {
		let (asset1, _, _, _) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.
		assert_ok!(AssetModule::add_asset_admin(RuntimeOrigin::root(), asset1.clone()));
		let count = AssetModule::assets_count();
		assert_eq!(count, 1);
		assert_ok!(AssetModule::remove_asset_admin(RuntimeOrigin::root(), asset1.id));
		let count = AssetModule::assets_count();
		assert_eq!(count, 0);
		assert_ok!(AssetModule::remove_asset_admin(RuntimeOrigin::root(), asset1.id));
	});
}
