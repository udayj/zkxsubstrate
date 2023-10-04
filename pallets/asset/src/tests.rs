use crate::{mock::*, Error, Event};
use frame_support::assert_ok;
use zkx_support::types::Asset;

fn setup() -> (Asset, Asset, Asset, Asset) {
	let ETH_ID: u128 = 4543560;
	let USDC_ID: u128 = 1431520323;
	let LINK_ID: u128 = 1279872587;
	let BTC_ID: u128 = 4346947;

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
	(asset1, asset2, asset3, asset4)
}

#[test]
fn it_works_for_replace_assets() {
	new_test_ext().execute_with(|| {
		let ETH_ID: u128 = 4543560;
		let (asset1, _, _, _) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.
		let assets: Vec<Asset> = vec![asset1.clone()];
		assert_ok!(AssetModule::replace_all_assets(RuntimeOrigin::signed(1), assets));

		assert_eq!(AssetModule::assets_count(), 1);
		let asset_storage = AssetModule::assets(ETH_ID);
		assert_eq!(asset_storage.unwrap(), asset1);

		// Assert that the correct event was deposited
		System::assert_last_event(Event::AssetsCreated { length: 1 }.into());
	});
}

#[test]
fn it_works_for_replace_assets_multiple_assets() {
	new_test_ext().execute_with(|| {
		let ETH_ID: u128 = 4543560;
		let USDC_ID: u128 = 1431520323;
		let LINK_ID: u128 = 1279872587;
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
		let asset_storage1 = AssetModule::assets(ETH_ID);
		assert_eq!(asset_storage1.unwrap(), asset1);
		let asset_storage2 = AssetModule::assets(USDC_ID);
		assert_eq!(asset_storage2.unwrap(), asset2);
		let asset_storage3 = AssetModule::assets(LINK_ID);
		assert_eq!(asset_storage3.unwrap(), asset3);

		// Assert that the correct event was deposited
		System::assert_last_event(Event::AssetsCreated { length: 3 }.into());
	});
}

#[test]
#[should_panic(expected = "DuplicateAsset")]
fn it_does_not_work_for_replace_assets_duplicate() {
	new_test_ext().execute_with(|| {
		let ETH_ID: u128 = 4543560;
		let (asset1, _, _, _) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.
		let name: Vec<u8> = "USDC".into();
		let asset: Asset = Asset {
			id: ETH_ID,
			name: name.try_into().unwrap(),
			is_tradable: false,
			is_collateral: true,
			token_decimal: 6,
		};
		let assets: Vec<Asset> = vec![asset1.clone(), asset.clone()];
		assert_ok!(AssetModule::replace_all_assets(RuntimeOrigin::signed(1), assets));
	});
}

#[test]
#[should_panic(expected = "InvalidAsset")]
fn it_does_not_work_for_replace_assets_invalid_id() {
	new_test_ext().execute_with(|| {
		let ETH_ID: u128 = 4543560;
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.
		let name: Vec<u8> = "USDC".into();
		let asset: Asset = Asset {
			id: ETH_ID,
			name: name.try_into().unwrap(),
			is_tradable: false,
			is_collateral: true,
			token_decimal: 6,
		};
		let assets: Vec<Asset> = vec![asset.clone()];
		assert_ok!(AssetModule::replace_all_assets(RuntimeOrigin::signed(1), assets));
	});
}

#[test]
#[should_panic(expected = "InvalidAsset")]
fn it_does_not_work_for_replace_assets_invalid_decimal() {
	new_test_ext().execute_with(|| {
		let ETH_ID: u128 = 4543560;
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.
		let name: Vec<u8> = "ETH".into();
		let asset: Asset = Asset {
			id: ETH_ID,
			name: name.try_into().unwrap(),
			is_tradable: false,
			is_collateral: true,
			token_decimal: 19,
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
		assert_ok!(AssetModule::add_asset(RuntimeOrigin::signed(1), asset1));
		let count = AssetModule::assets_count();
		assert_eq!(count, 1);
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
		assert_ok!(AssetModule::add_asset(RuntimeOrigin::signed(1), asset1.clone()));
		// Add the same asset again
		assert_ok!(AssetModule::add_asset(RuntimeOrigin::signed(1), asset1));
	});
}

#[test]
#[should_panic(expected = "InvalidAsset")]
fn test_add_asset_with_invalid_decimal() {
	new_test_ext().execute_with(|| {
		let ETH_ID: u128 = 4543560;
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.
		let name: Vec<u8> = "ETH".into();
		let asset: Asset = Asset {
			id: ETH_ID,
			name: name.try_into().unwrap(),
			is_tradable: false,
			is_collateral: true,
			token_decimal: 19,
		};
		assert_ok!(AssetModule::add_asset(RuntimeOrigin::signed(1), asset));
	});
}

#[test]
fn test_remove_asset() {
	new_test_ext().execute_with(|| {
		let (asset1, _, _, _) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.
		assert_ok!(AssetModule::add_asset(RuntimeOrigin::signed(1), asset1.clone()));
		let count = AssetModule::assets_count();
		assert_eq!(count, 1);
		assert_ok!(AssetModule::remove_asset(RuntimeOrigin::signed(1), asset1));
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
		assert_ok!(AssetModule::add_asset(RuntimeOrigin::signed(1), asset1.clone()));
		let count = AssetModule::assets_count();
		assert_eq!(count, 1);
		assert_ok!(AssetModule::remove_asset(RuntimeOrigin::signed(1), asset1.clone()));
		let count = AssetModule::assets_count();
		assert_eq!(count, 0);
		assert_ok!(AssetModule::remove_asset(RuntimeOrigin::signed(1), asset1));
	});
}
