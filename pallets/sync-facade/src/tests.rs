use super::*;
use crate::{mock::*, Event};
use frame_support::inherent::Vec;
use frame_system::{self as system};
use primitive_types::U256;
use system::RawOrigin;
use zkx_support::test_helpers::asset_helper::{usdc, usdt};
use zkx_support::types::Asset;

// declare test_helper module
pub mod test_helper;
use test_helper::*;

fn get_collaterals() -> Vec<Asset> {
	vec![usdc(), usdt()]
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

fn setup() -> sp_io::TestExternalities {
	// Create a new test environment
	let mut test_evn = new_test_ext();

	// Set the signers using admin account
	test_evn.execute_with(|| {
		SyncFacade::add_signer(RuntimeOrigin::root(), get_signers()[0])
			.expect("error while adding signer");
		SyncFacade::set_signers_quorum(RuntimeOrigin::root(), 1_u8)
			.expect("error while setting quorum");
		Assets::replace_all_assets(RuntimeOrigin::signed(1), get_collaterals())
			.expect("error while adding assets");
	});

	test_evn.into()
}

#[test]
fn add_signer_authorized() {
	// Get a test environment
	let mut env = setup();

	env.execute_with(|| {
		// Ensure the initial signer list is empty
		SyncFacade::add_signer(RuntimeOrigin::root(), get_signers()[1])
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
		SyncFacade::add_signer(RuntimeOrigin::root(), U256::from(0)).expect("Error in code");
	});
}

#[test]
#[should_panic(expected = "DuplicateSigner")]
fn add_signer_authorized_duplicate_pub_key() {
	// Get a test environment
	let mut env = setup();

	env.execute_with(|| {
		// Add signer; error
		SyncFacade::add_signer(RuntimeOrigin::root(), get_signers()[0]).expect("Error in code");
	});
}

#[test]
#[should_panic(expected = "InsufficientSigners")]
fn remove_signer_authorized_insufficient_signer() {
	// Get a test environment
	let mut env = setup();

	env.execute_with(|| {
		// Remove signer; error
		SyncFacade::remove_signer(RuntimeOrigin::root(), get_signers()[0]).expect("Error in code");
	});
}

#[test]
#[should_panic(expected = "SignerNotWhitelisted")]
fn remove_signer_authorized_invalid_signer() {
	// Get a test environment
	let mut env = setup();

	env.execute_with(|| {
		// Remove signer; error
		SyncFacade::remove_signer(RuntimeOrigin::root(), U256::from(0)).expect("Error in code");
	});
}

#[test]
fn remove_signer_authorized() {
	// Get a test environment
	let mut env = setup();

	env.execute_with(|| {
		// Add signer
		SyncFacade::add_signer(RuntimeOrigin::root(), get_signers()[1])
			.expect("error while adding signer");
		// Add signer
		SyncFacade::add_signer(RuntimeOrigin::root(), get_signers()[2])
			.expect("error while adding signer");
		// Remove signer
		SyncFacade::remove_signer(RuntimeOrigin::root(), get_signers()[0])
			.expect("error while removing signer");
		assert_eq!(SyncFacade::signers().len(), 2);
		assert_eq!(SyncFacade::signers(), get_signers()[1..3]);
		assert_eq!(SyncFacade::is_signer_valid(get_signers()[0]), false);
		assert_eq!(SyncFacade::is_signer_valid(get_signers()[1]), true);
		assert_eq!(SyncFacade::is_signer_valid(get_signers()[2]), true);

		// Remove signer
		SyncFacade::remove_signer(RuntimeOrigin::root(), get_signers()[1])
			.expect("error while removing signer");
		assert_eq!(SyncFacade::signers().len(), 1);
		assert_eq!(SyncFacade::signers(), vec![get_signers()[2]]);
		assert_eq!(SyncFacade::is_signer_valid(get_signers()[1]), false);
	});
}

// #[test]
// fn sync_deposit_events() {
// 	// Get a test environment
// 	let mut env = setup();

// 	let deposit_event_1 = UserDepositL2Trait::new()

// 	env.execute_with(|| {
// 		// Add signer
// 		SyncFacade::add_signer(RuntimeOrigin::root(), get_signers()[1])
// 			.expect("error while adding signer");
// 		// Add signer
// 		SyncFacade::add_signer(RuntimeOrigin::root(), get_signers()[2])
// 			.expect("error while adding signer");
// 		// Remove signer
// 		SyncFacade::remove_signer(RuntimeOrigin::root(), get_signers()[0])
// 			.expect("error while removing signer");
// 		assert_eq!(SyncFacade::signers().len(), 2);
// 		assert_eq!(SyncFacade::signers(), get_signers()[1..3]);
// 		assert_eq!(SyncFacade::is_signer_valid(get_signers()[0]), false);
// 		assert_eq!(SyncFacade::is_signer_valid(get_signers()[1]), true);
// 		assert_eq!(SyncFacade::is_signer_valid(get_signers()[2]), true);

// 		// Remove signer
// 		SyncFacade::remove_signer(RuntimeOrigin::root(), get_signers()[1])
// 			.expect("error while removing signer");
// 		assert_eq!(SyncFacade::signers().len(), 1);
// 		assert_eq!(SyncFacade::signers(), vec![get_signers()[2]]);
// 		assert_eq!(SyncFacade::is_signer_valid(get_signers()[1]), false);
// 	});
// }
