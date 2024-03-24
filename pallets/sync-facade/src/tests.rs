use crate::{mock::*, Event};
use frame_support::{assert_ok, dispatch::Vec};
use pallet_support::{
	test_helpers::{
		asset_helper::{btc, eth, usdc, usdt},
		market_helper::{btc_usdc, eth_usdc},
	},
	traits::{FieldElementExt, TradingFeesInterface, TradingInterface},
	types::{
		Asset, AssetRemoved, AssetUpdated, BaseFeeAggregate, ExtendedAsset, ExtendedMarket,
		FeeSettingsType, MarketRemoved, MarketUpdated, OrderSide, QuorumSet, SettingsAdded, Side,
		SignerAdded, SignerRemoved, SyncSignature, TradingAccountMinimal, UniversalEvent,
		UserDeposit,
	},
	FieldElement,
};
use primitive_types::U256;
use sp_arithmetic::fixed_point::FixedI128;
use sp_io::hashing::blake2_256;
use sp_runtime::{traits::ConstU32, BoundedVec};

// declare test_helper module
pub mod test_helper;
use test_helper::*;

fn get_trading_account_id(trading_account: TradingAccountMinimal) -> U256 {
	let account_address = U256::from(trading_account.account_address);
	let mut account_array: [u8; 32] = [0; 32];
	account_address.to_little_endian(&mut account_array);

	let mut concatenated_bytes: Vec<u8> = account_array.to_vec();
	concatenated_bytes.push(trading_account.index);
	let result: [u8; 33] = concatenated_bytes.try_into().unwrap();

	let trading_account_id: U256 = blake2_256(&result).into();
	trading_account_id
}

fn get_collaterals() -> Vec<ExtendedAsset> {
	vec![usdc(), usdt(), btc(), eth()]
}

fn get_markets() -> Vec<ExtendedMarket> {
	vec![btc_usdc(), eth_usdc()]
}

fn get_signers() -> Vec<U256> {
	vec![
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133c"),
		U256::from("0x60bb8c260ea369b4a1a7a7e593282171e2285470ce11704390d3b0faf2dc677"),
		U256::from("0x56ed355d24ed92801233aeb55a7929a52b65c99c640afd23b9fc054fdbbfdbd"),
		U256::from("0xa81ba5d1b269c7de0a41e84ef33ad200b961dccb98903d6590146af52ac440"),
		U256::from("0x45ccf172479eb092964e292477217b9efbb4f393706948b55ab8801eeef5752"),
	]
}

fn compare_base_fees(id: u128, expected_value: BaseFeeAggregate) {
	let actual_fees = TradingFees::base_fees_all(id);

	assert!(actual_fees == expected_value, "Mismatch fees");
}

fn check_fees_storage_empty(ids: Vec<u128>) {
	for id in ids {
		assert!(SyncFacade::get_temp_assets(id) == None, "Id is not removed");
		assert!(
			SyncFacade::get_temp_fees(id, FeeSettingsType::MakerVols) == None,
			"Maker volumes not removed"
		);
		assert!(
			SyncFacade::get_temp_fees(id, FeeSettingsType::TakerVols) == None,
			"Taker volumes not removed"
		);
		assert!(
			SyncFacade::get_temp_fees(id, FeeSettingsType::MakerOpen) == None,
			"Maker Open values not removed"
		);
		assert!(
			SyncFacade::get_temp_fees(id, FeeSettingsType::MakerClose) == None,
			"Maker Close values not removed"
		);
		assert!(
			SyncFacade::get_temp_fees(id, FeeSettingsType::TakerOpen) == None,
			"Taker Open values not removed"
		);
		assert!(
			SyncFacade::get_temp_fees(id, FeeSettingsType::TakerClose) == None,
			"Taker Close values not removed"
		);
	}
}

fn setup() -> sp_io::TestExternalities {
	// Create a new test environment
	let mut test_env = new_test_ext();

	// Set the signers using admin account
	test_env.execute_with(|| {
		SyncFacade::add_signer(RuntimeOrigin::signed(1), get_signers()[0])
			.expect("error while adding signer");
		SyncFacade::set_signers_quorum(RuntimeOrigin::signed(1), 1_u8)
			.expect("error while setting quorum");
		Assets::replace_all_assets(RuntimeOrigin::signed(1), get_collaterals())
			.expect("error while adding assets");
		Markets::replace_all_markets(RuntimeOrigin::signed(1), get_markets())
			.expect("error while adding markets");
		System::set_block_number(1336);
	});

	test_env.into()
}

#[test]
fn add_signer_authorized() {
	// Get a test environment
	let mut env = setup();

	env.execute_with(|| {
		// Add a signer
		SyncFacade::add_signer(RuntimeOrigin::signed(1), get_signers()[1])
			.expect("error while adding signer");
		assert_eq!(SyncFacade::signers().len(), 2);
		assert_eq!(SyncFacade::signers(), get_signers()[0..2]);
		assert_eq!(SyncFacade::is_signer_valid(get_signers()[0]), true);
		assert_eq!(SyncFacade::is_signer_valid(get_signers()[1]), true);
	});
}

#[test]
#[should_panic(expected = "ZeroSigner")]
fn add_signer_authorized_0_pub_key() {
	// Get a test environment
	let mut env = setup();

	env.execute_with(|| {
		// Add signer
		SyncFacade::add_signer(RuntimeOrigin::signed(1), U256::from(0)).expect("Error in code");
	});
}

#[test]
#[should_panic(expected = "DuplicateSigner")]
fn add_signer_authorized_duplicate_pub_key() {
	// Get a test environment
	let mut env = setup();

	env.execute_with(|| {
		// Add signer; error
		SyncFacade::add_signer(RuntimeOrigin::signed(1), get_signers()[0]).expect("Error in code");
	});
}

#[test]
#[should_panic(expected = "InsufficientSigners")]
fn remove_signer_authorized_insufficient_signer() {
	// Get a test environment
	let mut env = setup();

	env.execute_with(|| {
		// Remove signer; error
		SyncFacade::remove_signer(RuntimeOrigin::signed(1), get_signers()[0])
			.expect("Error in code");
	});
}

#[test]
#[should_panic(expected = "SignerNotWhitelisted")]
fn remove_signer_authorized_invalid_signer() {
	// Get a test environment
	let mut env = setup();

	env.execute_with(|| {
		// Remove signer; error
		SyncFacade::remove_signer(RuntimeOrigin::signed(1), U256::from(0)).expect("Error in code");
	});
}

#[test]
fn remove_signer_unauthorized() {
	// Get a test environment
	let mut env = setup();

	env.execute_with(|| {
		// Add signer
		SyncFacade::add_signer(RuntimeOrigin::signed(1), get_signers()[1])
			.expect("error while adding signer");
		// Add signer
		SyncFacade::add_signer(RuntimeOrigin::signed(1), get_signers()[2])
			.expect("error while adding signer");
		// Remove signer
		SyncFacade::remove_signer(RuntimeOrigin::signed(1), get_signers()[0])
			.expect("error while removing signer");
		assert_eq!(SyncFacade::signers().len(), 2);
		assert_eq!(SyncFacade::signers(), get_signers()[1..3]);
		assert_eq!(SyncFacade::is_signer_valid(get_signers()[0]), false);
		assert_eq!(SyncFacade::is_signer_valid(get_signers()[1]), true);
		assert_eq!(SyncFacade::is_signer_valid(get_signers()[2]), true);

		// Remove signer
		SyncFacade::remove_signer(RuntimeOrigin::signed(1), get_signers()[1])
			.expect("error while removing signer");
		assert_eq!(SyncFacade::signers().len(), 1);
		assert_eq!(SyncFacade::signers(), vec![get_signers()[2]]);
		assert_eq!(SyncFacade::is_signer_valid(get_signers()[1]), false);
	});
}

#[test]
#[should_panic(expected = "InsufficientSigners")]
fn set_quorum_authorized_insufficient_signers() {
	// Get a test environment
	let mut env = setup();

	env.execute_with(|| {
		SyncFacade::add_signer(RuntimeOrigin::signed(1), get_signers()[1])
			.expect("error while adding signer");
		// Set quorum; error
		SyncFacade::set_signers_quorum(RuntimeOrigin::signed(1), 3_u8)
			.expect("error while setting quorum");
	});
}

#[test]
fn set_quorum_authorized() {
	// Get a test environment
	let mut env = setup();

	env.execute_with(|| {
		SyncFacade::add_signer(RuntimeOrigin::signed(1), get_signers()[1])
			.expect("error while adding signer");
		SyncFacade::add_signer(RuntimeOrigin::signed(1), get_signers()[2])
			.expect("error while adding signer");
		// Set quorum; error
		SyncFacade::set_signers_quorum(RuntimeOrigin::signed(1), 3_u8)
			.expect("error while setting quorum");
		let quorum = SyncFacade::get_signers_quorum();
		assert_eq!(quorum, 3_u8);
	});
}

#[test]
fn sync_add_signer_events() {
	// Get a test environment
	let mut env = setup();

	let add_signer_event_1 = <SignerAdded as SignerAddedTrait>::new(1, get_signers()[1], 1337);

	let mut events_batch = <Vec<UniversalEvent> as UniversalEventArray>::new();
	events_batch.add_signer_added_event(add_signer_event_1);

	let events_batch_hash = events_batch.compute_hash();

	let mut signature_array = <Vec<SyncSignature> as SyncSignatureArray>::new();
	signature_array.add_new_signature(
		events_batch_hash,
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133c"),
		FieldElement::from(12345_u16),
	);

	env.execute_with(|| {
		// synchronize the events
		SyncFacade::synchronize_events(RuntimeOrigin::signed(1), events_batch, signature_array)
			.expect("error while adding signer");

		assert_eq!(SyncFacade::signers().len(), 2);
		assert_eq!(SyncFacade::signers(), get_signers()[0..2]);
		assert_eq!(SyncFacade::is_signer_valid(get_signers()[0]), true);
		assert_eq!(SyncFacade::is_signer_valid(get_signers()[1]), true);
	});
}

#[test]
fn sync_add_multiple_signer_events() {
	// Get a test environment
	let mut env = setup();

	let add_signer_event_1 = <SignerAdded as SignerAddedTrait>::new(1, get_signers()[1], 1337);
	let add_signer_event_2 = <SignerAdded as SignerAddedTrait>::new(2, get_signers()[2], 1337);
	let add_signer_event_3 = <SignerAdded as SignerAddedTrait>::new(2, get_signers()[3], 1337);
	let mut events_batch = <Vec<UniversalEvent> as UniversalEventArray>::new();
	events_batch.add_signer_added_event(add_signer_event_1);
	events_batch.add_signer_added_event(add_signer_event_2);
	events_batch.add_signer_added_event(add_signer_event_3);

	let events_batch_hash = events_batch.compute_hash();

	let mut signature_array = <Vec<SyncSignature> as SyncSignatureArray>::new();
	signature_array.add_new_signature(
		events_batch_hash,
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133c"),
		FieldElement::from(12345_u16),
	);

	env.execute_with(|| {
		// synchronize the events
		SyncFacade::synchronize_events(RuntimeOrigin::signed(1), events_batch, signature_array)
			.expect("error while adding signer");

		assert_eq!(SyncFacade::signers().len(), 4);
		assert_eq!(SyncFacade::signers(), get_signers()[0..4]);
		assert_eq!(SyncFacade::is_signer_valid(get_signers()[0]), true);
		assert_eq!(SyncFacade::is_signer_valid(get_signers()[1]), true);
		assert_eq!(SyncFacade::is_signer_valid(get_signers()[2]), true);
		assert_eq!(SyncFacade::is_signer_valid(get_signers()[3]), true);
	});
}

#[test]
fn sync_add_duplicate_signer_events() {
	// Get a test environment
	let mut env = setup();

	let add_signer_event_1 = <SignerAdded as SignerAddedTrait>::new(1, get_signers()[0], 1337);

	let mut events_batch = <Vec<UniversalEvent> as UniversalEventArray>::new();
	events_batch.add_signer_added_event(add_signer_event_1);

	let events_batch_hash = events_batch.compute_hash();

	let mut signature_array = <Vec<SyncSignature> as SyncSignatureArray>::new();
	signature_array.add_new_signature(
		events_batch_hash,
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133c"),
		FieldElement::from(12345_u16),
	);

	env.execute_with(|| {
		// synchronize the events
		SyncFacade::synchronize_events(RuntimeOrigin::signed(1), events_batch, signature_array)
			.expect("error while adding signer");

		assert_eq!(SyncFacade::signers().len(), 1);
		assert_eq!(SyncFacade::signers(), get_signers()[0..1]);
		assert_eq!(SyncFacade::is_signer_valid(get_signers()[0]), true);

		System::assert_has_event(Event::SignerAddedError { pub_key: get_signers()[0] }.into());
	});
}

#[test]
fn sync_update_asset_event_add_asset() {
	// Get a test environment
	let mut env = setup();

	let update_asset_event_1 = <AssetUpdated as AssetUpdatedTrait>::new(
		1,
		btc().asset.id,
		btc().asset,
		btc().asset_addresses,
		BoundedVec::<u8, ConstU32<256>>::new(),
		1337,
	);

	let mut events_batch = <Vec<UniversalEvent> as UniversalEventArray>::new();
	events_batch.add_asset_updated_event(update_asset_event_1);

	let events_batch_hash = events_batch.compute_hash();

	let mut signature_array = <Vec<SyncSignature> as SyncSignatureArray>::new();
	signature_array.add_new_signature(
		events_batch_hash,
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133c"),
		FieldElement::from(12345_u16),
	);

	env.execute_with(|| {
		// synchronize the events
		SyncFacade::synchronize_events(RuntimeOrigin::signed(1), events_batch, signature_array)
			.expect("error while updating asset");

		assert_eq!(Assets::assets_count(), 4);
		assert_eq!(Assets::assets(usdc().asset.id).unwrap(), usdc());
	});
}

#[test]
fn sync_update_asset_event_multiple_add_asset() {
	// Get a test environment
	let mut env = setup();

	let update_asset_event_1 = <AssetUpdated as AssetUpdatedTrait>::new(
		1,
		btc().asset.id,
		btc().asset,
		btc().asset_addresses,
		btc().metadata_url,
		1337,
	);

	let update_asset_event_2 = <AssetUpdated as AssetUpdatedTrait>::new(
		2,
		eth().asset.id,
		eth().asset,
		eth().asset_addresses,
		eth().metadata_url,
		1337,
	);

	let mut events_batch = <Vec<UniversalEvent> as UniversalEventArray>::new();
	events_batch.add_asset_updated_event(update_asset_event_1);
	events_batch.add_asset_updated_event(update_asset_event_2);

	let events_batch_hash = events_batch.compute_hash();

	let mut signature_array = <Vec<SyncSignature> as SyncSignatureArray>::new();
	signature_array.add_new_signature(
		events_batch_hash,
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133c"),
		FieldElement::from(12345_u16),
	);

	env.execute_with(|| {
		// synchronize the events
		SyncFacade::synchronize_events(RuntimeOrigin::signed(1), events_batch, signature_array)
			.expect("error while updating asset");

		assert_eq!(Assets::assets_count(), 4);
		assert_eq!(Assets::assets(usdt().asset.id).unwrap(), usdt());
		assert_eq!(Assets::assets(usdc().asset.id).unwrap(), usdc());
		assert_eq!(Assets::assets(btc().asset.id).unwrap(), btc());
		assert_eq!(Assets::assets(eth().asset.id).unwrap(), eth());
	});
}

#[test]
fn sync_asset_event_add_asset_remove_asset() {
	// Get a test environment
	let mut env = setup();

	let update_asset_event_1 = <AssetUpdated as AssetUpdatedTrait>::new(
		1,
		btc().asset.id,
		btc().asset,
		btc().asset_addresses,
		BoundedVec::<u8, ConstU32<256>>::new(),
		1337,
	);

	let remove_asset_event_1 = <AssetRemoved as AssetRemovedTrait>::new(2, btc().asset.id, 1337);

	let mut events_batch = <Vec<UniversalEvent> as UniversalEventArray>::new();
	events_batch.add_asset_updated_event(update_asset_event_1);
	events_batch.add_asset_removed_event(remove_asset_event_1);

	let events_batch_hash = events_batch.compute_hash();

	let mut signature_array = <Vec<SyncSignature> as SyncSignatureArray>::new();
	signature_array.add_new_signature(
		events_batch_hash,
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133c"),
		FieldElement::from(12345_u16),
	);

	env.execute_with(|| {
		// synchronize the events
		SyncFacade::synchronize_events(RuntimeOrigin::signed(1), events_batch, signature_array)
			.expect("error while updating asset");

		assert_eq!(Assets::assets_count(), 3);
	});
}

#[test]
fn sync_update_market_event_add_market() {
	// Get a test environment
	let mut env = setup();

	let update_market_event_1 = <MarketUpdated as MarketUpdatedTrait>::new(
		1,
		eth_usdc().market.id,
		eth_usdc().market,
		eth_usdc().metadata_url.clone(),
		1337,
	);

	let mut events_batch: Vec<UniversalEvent> = <Vec<UniversalEvent> as UniversalEventArray>::new();
	events_batch.add_market_updated_event(update_market_event_1);

	let events_batch_hash = events_batch.compute_hash();

	let mut signature_array = <Vec<SyncSignature> as SyncSignatureArray>::new();
	signature_array.add_new_signature(
		events_batch_hash,
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133c"),
		FieldElement::from(12345_u16),
	);

	env.execute_with(|| {
		// add assets
		assert_ok!(Assets::replace_all_assets(RuntimeOrigin::signed(1), vec![usdc(), eth()]));

		// synchronize the events
		SyncFacade::synchronize_events(RuntimeOrigin::signed(1), events_batch, signature_array)
			.expect("error while updating market");

		assert_eq!(Markets::markets_count(), 2);
		assert_eq!(Markets::markets(eth_usdc().market.id).unwrap(), eth_usdc());
	});
}

#[test]
fn sync_update_market_event_multiple_add_market() {
	// Get a test environment
	let mut env = setup();

	let update_market_event_1 = <MarketUpdated as MarketUpdatedTrait>::new(
		1,
		eth_usdc().market.id,
		eth_usdc().market,
		eth_usdc().metadata_url.clone(),
		1337,
	);

	let update_market_event_2 = <MarketUpdated as MarketUpdatedTrait>::new(
		2,
		btc_usdc().market.id,
		btc_usdc().market,
		btc_usdc().metadata_url.clone(),
		1337,
	);

	let mut events_batch: Vec<UniversalEvent> = <Vec<UniversalEvent> as UniversalEventArray>::new();
	events_batch.add_market_updated_event(update_market_event_1);
	events_batch.add_market_updated_event(update_market_event_2);

	let events_batch_hash = events_batch.compute_hash();

	let mut signature_array = <Vec<SyncSignature> as SyncSignatureArray>::new();
	signature_array.add_new_signature(
		events_batch_hash,
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133c"),
		FieldElement::from(12345_u16),
	);

	env.execute_with(|| {
		assert_ok!(Assets::replace_all_assets(
			RuntimeOrigin::signed(1),
			vec![usdc(), eth(), btc()]
		));

		// synchronize the events
		SyncFacade::synchronize_events(RuntimeOrigin::signed(1), events_batch, signature_array)
			.expect("error while updating market");

		assert_eq!(Markets::markets_count(), 2);
		assert_eq!(Markets::markets(eth_usdc().market.id).unwrap(), eth_usdc());
		assert_eq!(Markets::markets(btc_usdc().market.id).unwrap(), btc_usdc());
	});
}

#[test]
fn sync_update_market_event_update_market() {
	// Get a test environment
	let mut env = setup();

	let mut updated_market = eth_usdc();
	updated_market.market.is_archived = true;

	let update_market_event_1 = <MarketUpdated as MarketUpdatedTrait>::new(
		1,
		updated_market.market.id,
		updated_market.market.clone(),
		updated_market.metadata_url.clone(),
		1337,
	);

	let mut events_batch: Vec<UniversalEvent> = <Vec<UniversalEvent> as UniversalEventArray>::new();
	events_batch.add_market_updated_event(update_market_event_1);

	let events_batch_hash = events_batch.compute_hash();

	let mut signature_array = <Vec<SyncSignature> as SyncSignatureArray>::new();
	signature_array.add_new_signature(
		events_batch_hash,
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133c"),
		FieldElement::from(12345_u16),
	);

	env.execute_with(|| {
		// add assets
		assert_ok!(Assets::replace_all_assets(RuntimeOrigin::signed(1), vec![usdc(), eth()]));
		// add markets
		assert_ok!(Markets::replace_all_markets(RuntimeOrigin::signed(1), vec![eth_usdc()]));
		// synchronize the events
		SyncFacade::synchronize_events(RuntimeOrigin::signed(1), events_batch, signature_array)
			.expect("error while updating market");

		assert_eq!(Markets::markets_count(), 1);
		assert_eq!(Markets::markets(updated_market.market.id).unwrap(), updated_market);
	});
}

#[test]
fn sync_remove_market_event() {
	// Get a test environment
	let mut env = setup();

	let removed_market_event_1 =
		<MarketRemoved as MarketRemovedTrait>::new(1, eth_usdc().market.id, 1337);

	let mut events_batch: Vec<UniversalEvent> = <Vec<UniversalEvent> as UniversalEventArray>::new();
	events_batch.add_market_removed_event(removed_market_event_1);

	let events_batch_hash = events_batch.compute_hash();

	let mut signature_array = <Vec<SyncSignature> as SyncSignatureArray>::new();
	signature_array.add_new_signature(
		events_batch_hash,
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133c"),
		FieldElement::from(12345_u16),
	);

	env.execute_with(|| {
		// add assets
		assert_ok!(Assets::replace_all_assets(RuntimeOrigin::signed(1), vec![usdc(), eth()]));
		// add markets
		assert_ok!(Markets::replace_all_markets(RuntimeOrigin::signed(1), vec![eth_usdc()]));
		// synchronize the events
		SyncFacade::synchronize_events(RuntimeOrigin::signed(1), events_batch, signature_array)
			.expect("error while updating market");

		assert_eq!(Markets::markets_count(), 0);
	});
}

#[test]
fn sync_quorum_set_event() {
	// Get a test environment
	let mut env = setup();

	let quroum_set_event_1 = <QuorumSet as QuorumSetTrait>::new(1, 2_u8, 1337);

	let mut events_batch: Vec<UniversalEvent> = <Vec<UniversalEvent> as UniversalEventArray>::new();
	events_batch.add_quorum_set_event(quroum_set_event_1);

	let events_batch_hash = events_batch.compute_hash();

	let mut signature_array = <Vec<SyncSignature> as SyncSignatureArray>::new();
	signature_array.add_new_signature(
		events_batch_hash,
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133c"),
		FieldElement::from(12345_u16),
	);

	env.execute_with(|| {
		// add a signer
		SyncFacade::add_signer(RuntimeOrigin::signed(1), get_signers()[1])
			.expect("error while adding signer");
		// add assets
		assert_ok!(Assets::replace_all_assets(RuntimeOrigin::signed(1), vec![usdc(), eth()]));
		// add markets
		assert_ok!(Markets::replace_all_markets(RuntimeOrigin::signed(1), vec![eth_usdc()]));
		// synchronize the events
		SyncFacade::synchronize_events(RuntimeOrigin::signed(1), events_batch, signature_array)
			.expect("error while updating market");

		assert_eq!(SyncFacade::get_signers_quorum(), 2_u8);
	});
}

#[test]
fn sync_quorum_set_event_insufficient_signers() {
	// Get a test environment
	let mut env = setup();

	let quroum_set_event_1 = <QuorumSet as QuorumSetTrait>::new(1, 2_u8, 1337);

	let mut events_batch: Vec<UniversalEvent> = <Vec<UniversalEvent> as UniversalEventArray>::new();
	events_batch.add_quorum_set_event(quroum_set_event_1);

	let events_batch_hash = events_batch.compute_hash();

	let mut signature_array = <Vec<SyncSignature> as SyncSignatureArray>::new();
	signature_array.add_new_signature(
		events_batch_hash,
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133c"),
		FieldElement::from(12345_u16),
	);

	env.execute_with(|| {
		// add assets
		assert_ok!(Assets::replace_all_assets(RuntimeOrigin::signed(1), vec![usdc(), eth()]));
		// add markets
		assert_ok!(Markets::replace_all_markets(RuntimeOrigin::signed(1), vec![eth_usdc()]));
		// synchronize the events
		SyncFacade::synchronize_events(RuntimeOrigin::signed(1), events_batch, signature_array)
			.expect("error while updating market");

		assert_eq!(SyncFacade::get_signers_quorum(), 1_u8);
		System::assert_has_event(Event::QuorumSetError { quorum: 2_u8 }.into());
	});
}

#[test]
fn sync_remove_non_existent_market_event() {
	// Get a test environment
	let mut env = setup();

	let removed_market_event_1 = <MarketRemoved as MarketRemovedTrait>::new(1, 42_u128, 1337);

	let mut events_batch: Vec<UniversalEvent> = <Vec<UniversalEvent> as UniversalEventArray>::new();
	events_batch.add_market_removed_event(removed_market_event_1);

	let events_batch_hash = events_batch.compute_hash();

	let mut signature_array = <Vec<SyncSignature> as SyncSignatureArray>::new();
	signature_array.add_new_signature(
		events_batch_hash,
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133c"),
		FieldElement::from(12345_u16),
	);

	env.execute_with(|| {
		// synchronize the events
		SyncFacade::synchronize_events(RuntimeOrigin::signed(1), events_batch, signature_array)
			.expect("error while updating market");

		// Assert error debugging event has been emitted
		System::assert_has_event(Event::MarketRemovedError { id: 42_u128 }.into());
	});
}

#[test]
fn sync_update_asset_event_bump_asset() {
	// Get a test environment
	let mut env = setup();

	let usdc_asset = usdc();
	let modified_usdc_asset = ExtendedAsset {
		asset: Asset { is_collateral: false, version: 2, ..usdc_asset.asset },
		asset_addresses: usdc_asset.asset_addresses.clone(),
		metadata_url: usdc_asset.metadata_url.clone(),
	};

	let update_asset_event_1 = <AssetUpdated as AssetUpdatedTrait>::new(
		1,
		modified_usdc_asset.asset.id,
		modified_usdc_asset.asset.clone(),
		usdc_asset.asset_addresses.clone(),
		usdc_asset.metadata_url.clone(),
		1337,
	);

	let mut events_batch = <Vec<UniversalEvent> as UniversalEventArray>::new();
	events_batch.add_asset_updated_event(update_asset_event_1);

	let events_batch_hash = events_batch.compute_hash();

	let mut signature_array = <Vec<SyncSignature> as SyncSignatureArray>::new();
	signature_array.add_new_signature(
		events_batch_hash,
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133c"),
		FieldElement::from(12345_u16),
	);

	env.execute_with(|| {
		// synchronize the events
		SyncFacade::synchronize_events(RuntimeOrigin::signed(1), events_batch, signature_array)
			.expect("error while updating asset");

		assert_eq!(Assets::assets_count(), 4);
		assert_eq!(Assets::assets(modified_usdc_asset.asset.id).unwrap(), modified_usdc_asset);
	});
}

#[test]
fn sync_update_remove_asset() {
	// Get a test environment
	let mut env = setup();

	let remove_asset_event_1 = <AssetRemoved as AssetRemovedTrait>::new(1, usdc().asset.id, 1337);

	let mut events_batch = <Vec<UniversalEvent> as UniversalEventArray>::new();
	events_batch.add_asset_removed_event(remove_asset_event_1);

	let events_batch_hash = events_batch.compute_hash();

	let mut signature_array = <Vec<SyncSignature> as SyncSignatureArray>::new();
	signature_array.add_new_signature(
		events_batch_hash,
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133c"),
		FieldElement::from(12345_u16),
	);

	env.execute_with(|| {
		// synchronize the events
		SyncFacade::synchronize_events(RuntimeOrigin::signed(1), events_batch, signature_array)
			.expect("error while updating asset");

		assert_eq!(Assets::assets_count(), 3);
	});
}

#[test]
fn sync_update_remove_non_existent_asset() {
	// Get a test environment
	let mut env = setup();

	let remove_asset_event_1 = <AssetRemoved as AssetRemovedTrait>::new(1, 42_u128, 1337);

	let mut events_batch = <Vec<UniversalEvent> as UniversalEventArray>::new();
	events_batch.add_asset_removed_event(remove_asset_event_1);

	let events_batch_hash = events_batch.compute_hash();

	let mut signature_array = <Vec<SyncSignature> as SyncSignatureArray>::new();
	signature_array.add_new_signature(
		events_batch_hash,
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133c"),
		FieldElement::from(12345_u16),
	);

	env.execute_with(|| {
		// synchronize the events
		SyncFacade::synchronize_events(RuntimeOrigin::signed(1), events_batch, signature_array)
			.expect("error while updating asset");

		// Assert error event has been emitted
		System::assert_has_event(Event::AssetRemovedError { id: 42_u128 }.into());
	});
}

#[test]
#[should_panic(expected = "DuplicateBatch")]
fn sync_add_signer_events_duplicate_batch() {
	// Get a test environment
	let mut env = setup();

	let add_signer_event_1 = <SignerAdded as SignerAddedTrait>::new(1, get_signers()[1], 1337);

	let mut events_batch = <Vec<UniversalEvent> as UniversalEventArray>::new();
	events_batch.add_signer_added_event(add_signer_event_1);

	let events_batch_hash = events_batch.compute_hash();

	let mut signature_array = <Vec<SyncSignature> as SyncSignatureArray>::new();
	signature_array.add_new_signature(
		events_batch_hash,
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133c"),
		FieldElement::from(12345_u16),
	);

	env.execute_with(|| {
		// synchronize the events
		SyncFacade::synchronize_events(
			RuntimeOrigin::signed(1),
			events_batch.clone(),
			signature_array.clone(),
		)
		.expect("error while adding signer");

		// synchronize the events; error
		SyncFacade::synchronize_events(RuntimeOrigin::signed(1), events_batch, signature_array)
			.expect("error while adding signer");
	});
}

#[test]
#[should_panic(expected = "OldBatch")]
fn sync_batch_old_blocks() {
	// Get a test environment
	let mut env = setup();

	let add_signer_event_1 = <SignerAdded as SignerAddedTrait>::new(1, get_signers()[1], 1337);
	let mut events_batch = <Vec<UniversalEvent> as UniversalEventArray>::new();
	events_batch.add_signer_added_event(add_signer_event_1);

	let events_batch_hash = events_batch.compute_hash();

	let mut signature_array = <Vec<SyncSignature> as SyncSignatureArray>::new();
	signature_array.add_new_signature(
		events_batch_hash,
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133c"),
		FieldElement::from(12345_u16),
	);

	let add_signer_event_2 = <SignerAdded as SignerAddedTrait>::new(1, get_signers()[2], 1336);
	let mut events_batch_1 = <Vec<UniversalEvent> as UniversalEventArray>::new();
	events_batch_1.add_signer_added_event(add_signer_event_2);

	let events_batch_hash_1 = events_batch.compute_hash();

	let mut signature_array_1 = <Vec<SyncSignature> as SyncSignatureArray>::new();
	signature_array_1.add_new_signature(
		events_batch_hash_1,
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133c"),
		FieldElement::from(12345_u16),
	);

	env.execute_with(|| {
		// synchronize the events
		SyncFacade::synchronize_events(RuntimeOrigin::signed(1), events_batch, signature_array)
			.expect("error while adding signer");

		// synchronize the events; error
		SyncFacade::synchronize_events(RuntimeOrigin::signed(1), events_batch_1, signature_array_1)
			.expect("error while adding signer");
	});
}

#[test]
#[should_panic(expected = "InsufficientSignatures")]
fn sync_batch_insufficient_signatures() {
	// Get a test environment
	let mut env = setup();

	let add_signer_event_1 = <SignerAdded as SignerAddedTrait>::new(1, get_signers()[1], 1337);
	let mut events_batch = <Vec<UniversalEvent> as UniversalEventArray>::new();
	events_batch.add_signer_added_event(add_signer_event_1);

	let events_batch_hash = events_batch.compute_hash();

	let mut signature_array = <Vec<SyncSignature> as SyncSignatureArray>::new();
	signature_array.add_new_signature(
		events_batch_hash,
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133a"),
		FieldElement::from(12346_u16),
	);

	env.execute_with(|| {
		// synchronize the events
		SyncFacade::synchronize_events(RuntimeOrigin::signed(1), events_batch, signature_array)
			.expect("error while adding signer");
	});
}

#[test]
fn sync_remove_signer_events() {
	// Get a test environment
	let mut env = setup();

	// Add a signer that can be removed using sync events
	env.execute_with(|| {
		// Add a signer
		SyncFacade::add_signer(RuntimeOrigin::signed(1), get_signers()[1]).expect("Error in code");
	});

	let remove_signer_event_1 =
		<SignerRemoved as SignerRemovedTrait>::new(1, get_signers()[1], 1337);

	let mut events_batch = <Vec<UniversalEvent> as UniversalEventArray>::new();
	events_batch.add_signer_removed_event(remove_signer_event_1);

	let events_batch_hash = events_batch.compute_hash();

	let mut signature_array = <Vec<SyncSignature> as SyncSignatureArray>::new();
	signature_array.add_new_signature(
		events_batch_hash,
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133c"),
		FieldElement::from(12345_u16),
	);

	env.execute_with(|| {
		// synchronize the events
		SyncFacade::synchronize_events(RuntimeOrigin::signed(1), events_batch, signature_array)
			.expect("error while removing signer");

		assert_eq!(SyncFacade::signers().len(), 1);
		assert_eq!(SyncFacade::signers(), vec![get_signers()[0]]);
		assert_eq!(SyncFacade::is_signer_valid(get_signers()[0]), true);
		assert_eq!(SyncFacade::is_signer_valid(get_signers()[1]), false);
	});
}

#[test]
fn sync_remove_signer_insufficient_quorum_events() {
	// Get a test environment
	let mut env = setup();

	let remove_signer_event_1 =
		<SignerRemoved as SignerRemovedTrait>::new(1, get_signers()[0], 1337);

	let mut events_batch = <Vec<UniversalEvent> as UniversalEventArray>::new();
	events_batch.add_signer_removed_event(remove_signer_event_1);

	let events_batch_hash = events_batch.compute_hash();

	let mut signature_array = <Vec<SyncSignature> as SyncSignatureArray>::new();
	signature_array.add_new_signature(
		events_batch_hash,
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133c"),
		FieldElement::from(12345_u16),
	);

	env.execute_with(|| {
		// synchronize the events
		SyncFacade::synchronize_events(RuntimeOrigin::signed(1), events_batch, signature_array)
			.expect("error while removing signer");

		assert_eq!(SyncFacade::signers().len(), 1);
		assert_eq!(SyncFacade::signers(), vec![get_signers()[0]]);
		assert_eq!(SyncFacade::is_signer_valid(get_signers()[0]), true);

		System::assert_has_event(Event::SignerRemovedQuorumError { quorum: 1 }.into());
	});
}

#[test]
fn sync_remove_multiple_signer_events() {
	// Get a test environment
	let mut env = setup();

	// Add a signer that can be removed using sync events
	env.execute_with(|| {
		// Add a signer
		SyncFacade::add_signer(RuntimeOrigin::signed(1), get_signers()[1]).expect("Error in code");
		SyncFacade::add_signer(RuntimeOrigin::signed(1), get_signers()[2]).expect("Error in code");
	});

	let remove_signer_event_1 =
		<SignerRemoved as SignerRemovedTrait>::new(1, get_signers()[0], 1337);
	let remove_signer_event_2 =
		<SignerRemoved as SignerRemovedTrait>::new(1, get_signers()[1], 1337);
	let remove_signer_event_3 =
		<SignerRemoved as SignerRemovedTrait>::new(1, get_signers()[2], 1337);

	let mut events_batch = <Vec<UniversalEvent> as UniversalEventArray>::new();
	events_batch.add_signer_removed_event(remove_signer_event_1);
	events_batch.add_signer_removed_event(remove_signer_event_2);
	events_batch.add_signer_removed_event(remove_signer_event_3);

	let events_batch_hash = events_batch.compute_hash();

	let mut signature_array = <Vec<SyncSignature> as SyncSignatureArray>::new();
	signature_array.add_new_signature(
		events_batch_hash,
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133c"),
		FieldElement::from(12345_u16),
	);

	env.execute_with(|| {
		// synchronize the events
		SyncFacade::synchronize_events(RuntimeOrigin::signed(1), events_batch, signature_array)
			.expect("error while removing signer");

		assert_eq!(SyncFacade::signers().len(), 1);
		assert_eq!(SyncFacade::signers(), vec![get_signers()[2]]);
		assert_eq!(SyncFacade::is_signer_valid(get_signers()[0]), false);
		assert_eq!(SyncFacade::is_signer_valid(get_signers()[1]), false);
		assert_eq!(SyncFacade::is_signer_valid(get_signers()[2]), true);

		System::assert_has_event(Event::SignerRemovedQuorumError { quorum: 1 }.into());
	});
}

#[test]
fn sync_remove_non_existent_signer_events() {
	// Get a test environment
	let mut env = setup();

	// Add a signer that can be removed using sync events
	env.execute_with(|| {
		// Add a signer
		SyncFacade::add_signer(RuntimeOrigin::signed(1), get_signers()[1]).expect("Error in code");
	});

	let remove_signer_event_1 = <SignerRemoved as SignerRemovedTrait>::new(1, 42_u128.into(), 1337);

	let mut events_batch = <Vec<UniversalEvent> as UniversalEventArray>::new();
	events_batch.add_signer_removed_event(remove_signer_event_1);

	let events_batch_hash = events_batch.compute_hash();

	let mut signature_array = <Vec<SyncSignature> as SyncSignatureArray>::new();
	signature_array.add_new_signature(
		events_batch_hash,
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133c"),
		FieldElement::from(12345_u16),
	);

	env.execute_with(|| {
		// synchronize the events
		SyncFacade::synchronize_events(RuntimeOrigin::signed(1), events_batch, signature_array)
			.expect("error while adding signer");

		assert_eq!(SyncFacade::signers().len(), 2);
		assert_eq!(SyncFacade::signers(), get_signers()[0..2].to_vec());
		assert_eq!(SyncFacade::is_signer_valid(get_signers()[0]), true);
		assert_eq!(SyncFacade::is_signer_valid(get_signers()[1]), true);

		// Assert error event has been emitted
		System::assert_has_event(Event::SignerRemovedError { pub_key: 42_u128.into() }.into());
	});
}

#[test]
fn sync_deposit_events() {
	// Get a test environment
	let mut env = setup();

	let alice_account = TradingAccountMinimal {
		account_address: U256::from(100),
		pub_key: U256::from(1000),
		index: 1,
	};
	let alice_account_id = get_trading_account_id(alice_account);

	let bob_account = TradingAccountMinimal {
		account_address: U256::from(101),
		pub_key: U256::from(1001),
		index: 2,
	};
	let bob_account_id = get_trading_account_id(bob_account);

	let deposit_event_1 = <UserDeposit as UserDepositTrait>::new(
		1,
		alice_account,
		usdc().asset.id,
		U256::from(1),
		FixedI128::from(123),
		1337,
	);
	let deposit_event_2 = <UserDeposit as UserDepositTrait>::new(
		2,
		bob_account,
		usdc().asset.id,
		U256::from(2),
		FixedI128::from(154),
		1337,
	);

	let mut events_batch = <Vec<UniversalEvent> as UniversalEventArray>::new();
	events_batch.add_user_deposit_event(deposit_event_1);
	events_batch.add_user_deposit_event(deposit_event_2);

	let events_batch_hash = events_batch.compute_hash();

	let mut signature_array = <Vec<SyncSignature> as SyncSignatureArray>::new();
	signature_array.add_new_signature(
		events_batch_hash,
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133c"),
		FieldElement::from(12345_u16),
	);

	env.execute_with(|| {
		// synchronize the events
		SyncFacade::synchronize_events(RuntimeOrigin::signed(1), events_batch, signature_array)
			.expect("error while adding signer");

		let alice_balance = TradingAccounts::balances(alice_account_id, usdc().asset.id);
		let bob_balance = TradingAccounts::balances(bob_account_id, usdc().asset.id);

		assert_eq!(alice_balance, deposit_event_1.amount);
		assert_eq!(bob_balance, deposit_event_2.amount);
		assert_eq!(SyncFacade::get_sync_state(), (1337, 2, events_batch_hash.to_u256()));
	});
}

#[test]
fn sync_settings_event_usdc() {
	// Get a test environment
	let mut env = setup();

	let mut events_batch = <Vec<UniversalEvent> as UniversalEventArray>::new();
	events_batch
		.add_settings_event(<SettingsAdded as SettingsAddedTrait>::get_usdc_fees_settings());

	let events_batch_hash = events_batch.compute_hash();

	let mut signature_array = <Vec<SyncSignature> as SyncSignatureArray>::new();
	signature_array.add_new_signature(
		events_batch_hash,
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133c"),
		FieldElement::from(12345_u16),
	);

	env.execute_with(|| {
		// synchronize the events
		SyncFacade::synchronize_events(RuntimeOrigin::signed(1), events_batch, signature_array)
			.expect("error while adding settings");

		// Check if the fees were set successfully
		compare_base_fees(usdc().asset.id, get_usdc_aggregate_fees());

		// The storage should be empty
		check_fees_storage_empty(vec![usdc().asset.id]);
	});
}

#[test]
fn sync_settings_event_btc_usdc() {
	// Get a test environment
	let mut env = setup();

	let mut events_batch = <Vec<UniversalEvent> as UniversalEventArray>::new();
	events_batch
		.add_settings_event(<SettingsAdded as SettingsAddedTrait>::get_btc_usdc_fees_settings());

	let events_batch_hash = events_batch.compute_hash();

	let mut signature_array = <Vec<SyncSignature> as SyncSignatureArray>::new();
	signature_array.add_new_signature(
		events_batch_hash,
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133c"),
		FieldElement::from(12345_u16),
	);

	env.execute_with(|| {
		// synchronize the events
		SyncFacade::synchronize_events(RuntimeOrigin::signed(1), events_batch, signature_array)
			.expect("error while adding settings");

		// Check if the fees were set successfully
		compare_base_fees(btc_usdc().market.id, get_btc_usdc_aggregate_fees());

		// The storage should be empty
		check_fees_storage_empty(vec![btc_usdc().market.id]);

		// Get the aggregate fee structure stored in TradingFees
		let fee_details = TradingFees::get_all_fees(btc_usdc().market.id, usdc().asset.id);

		// Check fees for maker
		let fees_1 = Trading::get_base_fee_rate(
			&fee_details,
			Side::Buy,
			OrderSide::Maker,
			FixedI128::from_u32(9999),
		);
		let fees_2 = Trading::get_base_fee_rate(
			&fee_details,
			Side::Buy,
			OrderSide::Maker,
			FixedI128::from_u32(999999),
		);
		let fees_3 = Trading::get_base_fee_rate(
			&fee_details,
			Side::Buy,
			OrderSide::Maker,
			FixedI128::from_u32(1000001),
		);

		// Check if we get correct values from get_fee_rate
		assert!(fees_1 == (FixedI128::from_float(0.002), 1), "Invalid fees for tier 1");
		assert!(fees_2 == (FixedI128::from_float(0.001), 2), "Invalid fees for tier 2");
		assert!(fees_3 == (FixedI128::from_float(0.0), 3), "Invalid fees for tier 2");

		// Check fees for taker
		let fees_1 = Trading::get_base_fee_rate(
			&fee_details,
			Side::Buy,
			OrderSide::Taker,
			FixedI128::from_u32(9999),
		);
		let fees_2 = Trading::get_base_fee_rate(
			&fee_details,
			Side::Buy,
			OrderSide::Taker,
			FixedI128::from_u32(999999),
		);
		let fees_3 = Trading::get_base_fee_rate(
			&fee_details,
			Side::Buy,
			OrderSide::Taker,
			FixedI128::from_u32(1000001),
		);
		let fees_4 = Trading::get_base_fee_rate(
			&fee_details,
			Side::Buy,
			OrderSide::Taker,
			FixedI128::from_u32(5000001),
		);

		// Check if we get correct values from get_fee_rate
		assert!(fees_1 == (FixedI128::from_float(0.005), 1), "Invalid fees for tier 1");
		assert!(fees_2 == (FixedI128::from_float(0.0045), 2), "Invalid fees for tier 2");
		assert!(fees_3 == (FixedI128::from_float(0.004), 3), "Invalid fees for tier 3");
		assert!(fees_4 == (FixedI128::from_float(0.002), 4), "Invalid fees for tier 4");
	});
}

#[test]
fn sync_settings_event_usdt() {
	// Get a test environment
	let mut env = setup();

	let mut events_batch = <Vec<UniversalEvent> as UniversalEventArray>::new();
	events_batch
		.add_settings_event(<SettingsAdded as SettingsAddedTrait>::get_usdt_fees_settings());

	let events_batch_hash = events_batch.compute_hash();

	let mut signature_array = <Vec<SyncSignature> as SyncSignatureArray>::new();
	signature_array.add_new_signature(
		events_batch_hash,
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133c"),
		FieldElement::from(12345_u16),
	);

	env.execute_with(|| {
		// synchronize the events
		SyncFacade::synchronize_events(RuntimeOrigin::signed(1), events_batch, signature_array)
			.expect("error while adding settings");

		// Check if the fees were set successfully
		compare_base_fees(usdt().asset.id, get_usdt_aggregate_fees());

		// The storage should be empty
		check_fees_storage_empty(vec![usdt().asset.id]);
	});
}

#[test]
fn sync_settings_event_multiple_collaterals_markets() {
	// Get a test environment
	let mut env = setup();

	let mut events_batch = <Vec<UniversalEvent> as UniversalEventArray>::new();
	let mut usdc_fees = <SettingsAdded as SettingsAddedTrait>::get_usdc_fees_settings();
	let usdt_fees = <SettingsAdded as SettingsAddedTrait>::get_usdt_fees_settings();
	let btc_usdc_fees = <SettingsAdded as SettingsAddedTrait>::get_btc_usdc_fees_settings();

	for setting in usdt_fees.settings {
		usdc_fees.settings.force_push(setting);
	}

	for setting in btc_usdc_fees.settings {
		usdc_fees.settings.force_push(setting);
	}

	events_batch.add_settings_event(usdc_fees);

	let events_batch_hash = events_batch.compute_hash();

	let mut signature_array = <Vec<SyncSignature> as SyncSignatureArray>::new();
	signature_array.add_new_signature(
		events_batch_hash,
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133c"),
		FieldElement::from(12345_u16),
	);

	env.execute_with(|| {
		// synchronize the events
		SyncFacade::synchronize_events(RuntimeOrigin::signed(1), events_batch, signature_array)
			.expect("error while adding settings");

		// Check if the fees were set successfully
		// USDT
		compare_base_fees(usdt().asset.id, get_usdt_aggregate_fees());

		// USDC
		compare_base_fees(usdc().asset.id, get_usdc_aggregate_fees());

		// BTC USDC
		compare_base_fees(btc_usdc().market.id, get_btc_usdc_aggregate_fees());

		// The storage should be empty
		check_fees_storage_empty(vec![usdc().asset.id, usdt().asset.id, btc_usdc().market.id]);

		let fee_details = TradingFees::get_all_fees(btc_usdc().market.id, usdc().asset.id);

		// Check fees for maker
		let fees_1 = Trading::get_base_fee_rate(
			&fee_details,
			Side::Buy,
			OrderSide::Maker,
			FixedI128::from_u32(9999),
		);
		let fees_2 = Trading::get_base_fee_rate(
			&fee_details,
			Side::Buy,
			OrderSide::Maker,
			FixedI128::from_u32(999999),
		);
		let fees_3 = Trading::get_base_fee_rate(
			&fee_details,
			Side::Buy,
			OrderSide::Maker,
			FixedI128::from_u32(1000001),
		);

		// Check if we get correct values from get_fee_rate
		assert!(fees_1 == (FixedI128::from_float(0.002), 1), "Invalid fees for tier 1");
		assert!(fees_2 == (FixedI128::from_float(0.001), 2), "Invalid fees for tier 2");
		assert!(fees_3 == (FixedI128::from_float(0.0), 3), "Invalid fees for tier 2");

		// Check fees for taker
		let fees_1 = Trading::get_base_fee_rate(
			&fee_details,
			Side::Buy,
			OrderSide::Taker,
			FixedI128::from_u32(9999),
		);
		let fees_2 = Trading::get_base_fee_rate(
			&fee_details,
			Side::Buy,
			OrderSide::Taker,
			FixedI128::from_u32(999999),
		);
		let fees_3 = Trading::get_base_fee_rate(
			&fee_details,
			Side::Buy,
			OrderSide::Taker,
			FixedI128::from_u32(1000001),
		);
		let fees_4 = Trading::get_base_fee_rate(
			&fee_details,
			Side::Buy,
			OrderSide::Taker,
			FixedI128::from_u32(5000001),
		);

		// Check if we get correct values from get_fee_rate
		assert!(fees_1 == (FixedI128::from_float(0.005), 1), "Invalid fees for tier 1");
		assert!(fees_2 == (FixedI128::from_float(0.0045), 2), "Invalid fees for tier 2");
		assert!(fees_3 == (FixedI128::from_float(0.004), 3), "Invalid fees for tier 3");
		assert!(fees_4 == (FixedI128::from_float(0.002), 4), "Invalid fees for tier 4");
	});
}

#[test]
fn sync_settings_event_abr_default() {
	// Get a test environment
	let mut env = setup();

	let mut events_batch = <Vec<UniversalEvent> as UniversalEventArray>::new();
	events_batch
		.add_settings_event(<SettingsAdded as SettingsAddedTrait>::get_max_default_settings());

	let events_batch_hash = events_batch.compute_hash();

	let mut signature_array = <Vec<SyncSignature> as SyncSignatureArray>::new();
	signature_array.add_new_signature(
		events_batch_hash,
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133c"),
		FieldElement::from(12345_u16),
	);

	env.execute_with(|| {
		// synchronize the events
		SyncFacade::synchronize_events(RuntimeOrigin::signed(1), events_batch, signature_array)
			.expect("error while adding settings");

		// Check if the max abr value is set
		assert!(Prices::default_max() == FixedI128::from_float(0.0012), "Wrong max default value");
	});
}

#[test]
fn sync_settings_event_abr_default_empty_values() {
	// Get a test environment
	let mut env = setup();

	let mut events_batch = <Vec<UniversalEvent> as UniversalEventArray>::new();
	let mut default_max_abr = <SettingsAdded as SettingsAddedTrait>::get_max_default_settings();
	default_max_abr.settings[0].values = BoundedVec::new();
	events_batch.add_settings_event(default_max_abr);

	let events_batch_hash = events_batch.compute_hash();

	let mut signature_array = <Vec<SyncSignature> as SyncSignatureArray>::new();
	signature_array.add_new_signature(
		events_batch_hash,
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133c"),
		FieldElement::from(12345_u16),
	);

	env.execute_with(|| {
		// synchronize the events
		SyncFacade::synchronize_events(RuntimeOrigin::signed(1), events_batch, signature_array)
			.expect("error while adding settings");

		// Check if the max abr value is set
		assert!(Prices::default_max() == FixedI128::from_float(0.0), "Wrong max default value");

		// Check for the empty event
		System::assert_has_event(Event::EmptyValuesError { id: 45 }.into());
	});
}

#[test]
fn sync_settings_event_abr_btc_usd_value() {
	// Get a test environment
	let mut env = setup();

	let mut events_batch = <Vec<UniversalEvent> as UniversalEventArray>::new();
	events_batch
		.add_settings_event(<SettingsAdded as SettingsAddedTrait>::get_max_btc_usdc_settings());

	let events_batch_hash = events_batch.compute_hash();

	let mut signature_array = <Vec<SyncSignature> as SyncSignatureArray>::new();
	signature_array.add_new_signature(
		events_batch_hash,
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133c"),
		FieldElement::from(12345_u16),
	);

	env.execute_with(|| {
		// synchronize the events
		SyncFacade::synchronize_events(RuntimeOrigin::signed(1), events_batch, signature_array)
			.expect("error while adding settings");

		// Check if the max abr value is set
		assert!(
			Prices::max_abr(btc_usdc().market.id) == FixedI128::from_float(0.01),
			"Wrong max value for btc_usdc"
		);
	});
}

#[test]
fn sync_settings_event_abr_invalid_market() {
	// Get a test environment
	let mut env = setup();

	let mut events_batch = <Vec<UniversalEvent> as UniversalEventArray>::new();
	let mut max_btc_usdc = <SettingsAdded as SettingsAddedTrait>::get_max_btc_usdc_settings();
	max_btc_usdc.settings[0].key = U256::from(1325909088870421414631324406079277_i128);
	events_batch.add_settings_event(max_btc_usdc);

	let events_batch_hash = events_batch.compute_hash();

	let mut signature_array = <Vec<SyncSignature> as SyncSignatureArray>::new();
	signature_array.add_new_signature(
		events_batch_hash,
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133c"),
		FieldElement::from(12345_u16),
	);

	env.execute_with(|| {
		// synchronize the events
		SyncFacade::synchronize_events(RuntimeOrigin::signed(1), events_batch, signature_array)
			.expect("error while adding settings");

		// Check if the max abr value is set
		assert!(
			Prices::max_abr(btc_usdc().market.id) == FixedI128::from_float(0.0),
			"Wrong max value for btc_usdc"
		);

		// Check for the empty event
		System::assert_has_event(Event::InvalidMarket { id: 6004514686699258947 }.into());
	});
}

#[test]
fn sync_settings_event_abr_multiple_markets() {
	// Get a test environment
	let mut env = setup();

	let mut events_batch = <Vec<UniversalEvent> as UniversalEventArray>::new();
	let mut max_btc_usdc = <SettingsAdded as SettingsAddedTrait>::get_max_btc_usdc_settings();
	let max_eth_usdc = <SettingsAdded as SettingsAddedTrait>::get_max_eth_usdc_settings();
	let max_abr = <SettingsAdded as SettingsAddedTrait>::get_max_default_settings();

	// Add the eth and default values to the same array
	for setting in max_eth_usdc.settings {
		max_btc_usdc.settings.force_push(setting);
	}

	for setting in max_abr.settings {
		max_btc_usdc.settings.force_push(setting);
	}

	events_batch.add_settings_event(max_btc_usdc);

	let events_batch_hash = events_batch.compute_hash();

	let mut signature_array = <Vec<SyncSignature> as SyncSignatureArray>::new();
	signature_array.add_new_signature(
		events_batch_hash,
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133c"),
		FieldElement::from(12345_u16),
	);

	env.execute_with(|| {
		// synchronize the events
		SyncFacade::synchronize_events(RuntimeOrigin::signed(1), events_batch, signature_array)
			.expect("error while adding settings");

		// Check if the max abr value is set for btc_usdc
		assert!(
			Prices::max_abr(btc_usdc().market.id) == FixedI128::from_float(0.01),
			"Wrong max default value"
		);

		// Check if the max abr value is set for eth_usdc
		assert!(
			Prices::max_abr(eth_usdc().market.id) == FixedI128::from_float(0.05),
			"Wrong max value for eth_usdc"
		);

		// Check if the max abr value is set
		assert!(
			Prices::default_max() == FixedI128::from_float(0.0012),
			"Wrong max value for eth_usdc"
		);
	});
}

#[test]
fn sync_settings_invalid_key_general_settings_type() {
	// Get a test environment
	let mut env = setup();

	let mut events_batch = <Vec<UniversalEvent> as UniversalEventArray>::new();
	let mut usdc_fees = <SettingsAdded as SettingsAddedTrait>::get_usdc_fees_settings();
	usdc_fees.settings[0].key = U256::from(337046609303792675741519_i128);
	events_batch.add_settings_event(usdc_fees);

	let events_batch_hash = events_batch.compute_hash();

	let mut signature_array = <Vec<SyncSignature> as SyncSignatureArray>::new();
	signature_array.add_new_signature(
		events_batch_hash,
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133c"),
		FieldElement::from(12345_u16),
	);

	env.execute_with(|| {
		// synchronize the events
		SyncFacade::synchronize_events(RuntimeOrigin::signed(1), events_batch, signature_array)
			.expect("error while adding settings");

		System::assert_has_event(Event::SettingsKeyError { key: 71 }.into());

		// The storage should be empty
		check_fees_storage_empty(vec![usdc().asset.id]);
	});
}

#[test]
fn sync_settings_invalid_key_unknown_settings_type() {
	// Get a test environment
	let mut env = setup();

	let mut events_batch = <Vec<UniversalEvent> as UniversalEventArray>::new();
	let mut usdc_fees = <SettingsAdded as SettingsAddedTrait>::get_usdc_fees_settings();
	usdc_fees.settings[0].key = U256::from(379547907649619482664783_i128);
	events_batch.add_settings_event(usdc_fees);

	let events_batch_hash = events_batch.compute_hash();

	let mut signature_array = <Vec<SyncSignature> as SyncSignatureArray>::new();
	signature_array.add_new_signature(
		events_batch_hash,
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133c"),
		FieldElement::from(12345_u16),
	);

	env.execute_with(|| {
		// synchronize the events
		SyncFacade::synchronize_events(RuntimeOrigin::signed(1), events_batch, signature_array)
			.expect("error while adding settings");

		System::assert_has_event(Event::SettingsKeyError { key: 80 }.into());

		// The storage should be empty
		check_fees_storage_empty(vec![usdc().asset.id]);
	});
}

#[test]
fn sync_settings_unknown_id() {
	// Get a test environment
	let mut env = setup();

	let mut events_batch = <Vec<UniversalEvent> as UniversalEventArray>::new();
	let frax_fees = <SettingsAdded as SettingsAddedTrait>::get_frax_fees_settings();
	events_batch.add_settings_event(frax_fees);

	let events_batch_hash = events_batch.compute_hash();

	let mut signature_array = <Vec<SyncSignature> as SyncSignatureArray>::new();
	signature_array.add_new_signature(
		events_batch_hash,
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133c"),
		FieldElement::from(12345_u16),
	);

	env.execute_with(|| {
		// synchronize the events
		SyncFacade::synchronize_events(RuntimeOrigin::signed(1), events_batch, signature_array)
			.expect("error while adding settings");

		System::assert_has_event(Event::UnknownIdForFees { id: 1179795800 }.into());

		// The storage should be empty
		check_fees_storage_empty(vec![1179795800]);
	});
}

#[test]
fn sync_settings_invalid_key_order_side_key() {
	// Get a test environment
	let mut env = setup();

	let mut events_batch = <Vec<UniversalEvent> as UniversalEventArray>::new();
	let mut usdc_fees = <SettingsAdded as SettingsAddedTrait>::get_usdc_fees_settings();
	usdc_fees.settings[0].key = U256::from(332324242820850015559503_i128);
	events_batch.add_settings_event(usdc_fees);

	let events_batch_hash = events_batch.compute_hash();

	let mut signature_array = <Vec<SyncSignature> as SyncSignatureArray>::new();
	signature_array.add_new_signature(
		events_batch_hash,
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133c"),
		FieldElement::from(12345_u16),
	);

	env.execute_with(|| {
		// synchronize the events
		SyncFacade::synchronize_events(RuntimeOrigin::signed(1), events_batch, signature_array)
			.expect("error while adding settings");

		System::assert_has_event(Event::SettingsKeyError { key: 76 }.into());

		// The storage should be empty
		check_fees_storage_empty(vec![usdc().asset.id]);
	});
}

#[test]
fn sync_settings_invalid_key_maker_side_key() {
	// Get a test environment
	let mut env = setup();

	let mut events_batch = <Vec<UniversalEvent> as UniversalEventArray>::new();
	let mut usdc_fees = <SettingsAdded as SettingsAddedTrait>::get_usdc_fees_settings();
	usdc_fees.settings[0].key = U256::from(332324242820850015625050_i128);
	events_batch.add_settings_event(usdc_fees);

	let events_batch_hash = events_batch.compute_hash();

	let mut signature_array = <Vec<SyncSignature> as SyncSignatureArray>::new();
	signature_array.add_new_signature(
		events_batch_hash,
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133c"),
		FieldElement::from(12345_u16),
	);

	env.execute_with(|| {
		// synchronize the events
		SyncFacade::synchronize_events(RuntimeOrigin::signed(1), events_batch, signature_array)
			.expect("error while adding settings");

		System::assert_has_event(Event::SettingsKeyError { key: 90 }.into());

		// The storage should be empty
		check_fees_storage_empty(vec![usdc().asset.id]);
	});
}

#[test]
fn sync_settings_invalid_key_taker_side_key() {
	// Get a test environment
	let mut env = setup();

	let mut events_batch = <Vec<UniversalEvent> as UniversalEventArray>::new();
	let mut usdc_fees = <SettingsAdded as SettingsAddedTrait>::get_usdc_fees_settings();
	usdc_fees.settings[0].key = U256::from(332324242820850016083794_i128);
	events_batch.add_settings_event(usdc_fees);

	let events_batch_hash = events_batch.compute_hash();

	let mut signature_array = <Vec<SyncSignature> as SyncSignatureArray>::new();
	signature_array.add_new_signature(
		events_batch_hash,
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133c"),
		FieldElement::from(12345_u16),
	);

	env.execute_with(|| {
		// synchronize the events
		SyncFacade::synchronize_events(RuntimeOrigin::signed(1), events_batch, signature_array)
			.expect("error while adding settings");

		System::assert_has_event(Event::SettingsKeyError { key: 82 }.into());

		// The storage should be empty
		check_fees_storage_empty(vec![usdc().asset.id]);
	});
}

#[test]
fn sync_settings_event_insuffient_data_usdt() {
	// Get a test environment
	let mut env = setup();

	let mut events_batch = <Vec<UniversalEvent> as UniversalEventArray>::new();
	let mut modified_usdt_fees = <SettingsAdded as SettingsAddedTrait>::get_usdt_fees_settings();
	modified_usdt_fees.settings.pop();
	events_batch.add_settings_event(modified_usdt_fees);

	let events_batch_hash = events_batch.compute_hash();

	let mut signature_array = <Vec<SyncSignature> as SyncSignatureArray>::new();
	signature_array.add_new_signature(
		events_batch_hash,
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133c"),
		FieldElement::from(12345_u16),
	);

	env.execute_with(|| {
		// synchronize the events
		SyncFacade::synchronize_events(RuntimeOrigin::signed(1), events_batch, signature_array)
			.expect("error while adding settings");
		System::assert_has_event(Event::InsufficientFeeData { id: usdt().asset.id }.into());

		// The storage should be empty
		check_fees_storage_empty(vec![usdt().asset.id]);
	});
}

#[test]
fn sync_settings_event_invalid_key_pattern() {
	// Get a test environment
	let mut env = setup();

	let mut events_batch = <Vec<UniversalEvent> as UniversalEventArray>::new();
	let mut modified_usdt_fees = <SettingsAdded as SettingsAddedTrait>::get_usdt_fees_settings();
	modified_usdt_fees.settings[0].key = U256::from(5070865521559494477_i128);
	events_batch.add_settings_event(modified_usdt_fees);

	let events_batch_hash = events_batch.compute_hash();

	let mut signature_array = <Vec<SyncSignature> as SyncSignatureArray>::new();
	signature_array.add_new_signature(
		events_batch_hash,
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133c"),
		FieldElement::from(12345_u16),
	);

	env.execute_with(|| {
		// synchronize the events
		SyncFacade::synchronize_events(RuntimeOrigin::signed(1), events_batch, signature_array)
			.expect("error while adding settings");
		System::assert_has_event(
			Event::TokenParsingError { key: U256::from(5070865521559494477_i128) }.into(),
		);

		// The storage should be empty
		check_fees_storage_empty(vec![usdt().asset.id]);
	});
}

#[test]
fn sync_settings_event_invalid_length_usdt() {
	// Get a test environment
	let mut env = setup();

	let mut events_batch = <Vec<UniversalEvent> as UniversalEventArray>::new();
	let mut modified_usdt_fees = <SettingsAdded as SettingsAddedTrait>::get_usdt_fees_settings();
	modified_usdt_fees.settings[0].values.pop();
	modified_usdt_fees.settings[0].values.pop();
	events_batch.add_settings_event(modified_usdt_fees);

	let events_batch_hash = events_batch.compute_hash();

	let mut signature_array = <Vec<SyncSignature> as SyncSignatureArray>::new();
	signature_array.add_new_signature(
		events_batch_hash,
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133c"),
		FieldElement::from(12345_u16),
	);

	env.execute_with(|| {
		// synchronize the events
		SyncFacade::synchronize_events(RuntimeOrigin::signed(1), events_batch, signature_array)
			.expect("error while adding settings");

		System::assert_has_event(Event::FeeDataLengthMismatch { id: usdt().asset.id }.into());

		// The storage should be empty
		check_fees_storage_empty(vec![usdt().asset.id]);
	});
}

#[test]
fn sync_deposit_event_non_existent_asset() {
	// Get a test environment
	let mut env = setup();

	let alice_account = TradingAccountMinimal {
		account_address: U256::from(100),
		pub_key: U256::from(1000),
		index: 1,
	};
	let alice_account_id = get_trading_account_id(alice_account);

	let deposit_event_1 = <UserDeposit as UserDepositTrait>::new(
		1,
		alice_account,
		12345_u128,
		U256::from(1),
		FixedI128::from(123),
		1337,
	);

	let mut events_batch = <Vec<UniversalEvent> as UniversalEventArray>::new();
	events_batch.add_user_deposit_event(deposit_event_1);

	let events_batch_hash = events_batch.compute_hash();

	let mut signature_array = <Vec<SyncSignature> as SyncSignatureArray>::new();
	signature_array.add_new_signature(
		events_batch_hash,
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133c"),
		FieldElement::from(12345_u16),
	);

	env.execute_with(|| {
		// synchronize the events
		SyncFacade::synchronize_events(RuntimeOrigin::signed(1), events_batch, signature_array)
			.expect("error while adding signer");

		let alice_balance = TradingAccounts::balances(alice_account_id, 12345_u128);

		assert_eq!(alice_balance, 0.into());

		// Assert error event has been emitted
		System::assert_has_event(Event::UserDepositError { collateral_id: 12345_u128 }.into());
	});
}

#[test]
fn sync_deposit_event_non_collateral_asset() {
	// Get a test environment
	let mut env = setup();

	let alice_account = TradingAccountMinimal {
		account_address: U256::from(100),
		pub_key: U256::from(1000),
		index: 1,
	};
	let alice_account_id = get_trading_account_id(alice_account);

	let deposit_event_1 = <UserDeposit as UserDepositTrait>::new(
		1,
		alice_account,
		btc().asset.id,
		U256::from(1),
		FixedI128::from(123),
		1337,
	);

	let mut events_batch = <Vec<UniversalEvent> as UniversalEventArray>::new();
	events_batch.add_user_deposit_event(deposit_event_1);

	let events_batch_hash = events_batch.compute_hash();

	let mut signature_array = <Vec<SyncSignature> as SyncSignatureArray>::new();
	signature_array.add_new_signature(
		events_batch_hash,
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133c"),
		FieldElement::from(12345_u16),
	);

	env.execute_with(|| {
		Assets::replace_all_assets(RuntimeOrigin::signed(1), vec![usdc(), usdt(), btc()])
			.expect("error while adding assets");

		// synchronize the events
		SyncFacade::synchronize_events(RuntimeOrigin::signed(1), events_batch, signature_array)
			.expect("error while adding deposit event");

		let alice_balance = TradingAccounts::balances(alice_account_id, btc().asset.id);

		assert_eq!(alice_balance, 0.into());

		// Assert error event has been emitted
		System::assert_has_event(Event::UserDepositError { collateral_id: btc().asset.id }.into());
	});
}
