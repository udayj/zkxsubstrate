use crate::mock::*;
use frame_support::inherent::Vec;
use primitive_types::U256;
use sp_arithmetic::fixed_point::FixedI128;
use sp_io::hashing::blake2_256;
use zkx_support::test_helpers::asset_helper::{usdc, usdt};
use zkx_support::types::{
	Asset, SignerAdded, SignerRemoved, SyncSignature, TradingAccountMinimal, UniversalEvent,
	UserDeposit,
};
use zkx_support::FieldElement;

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
		System::set_block_number(1336);
	});

	test_evn.into()
}

#[test]
fn add_signer_authorized() {
	// Get a test environment
	let mut env = setup();

	env.execute_with(|| {
		// Add a signer
		SyncFacade::add_signer(RuntimeOrigin::root(), get_signers()[1])
			.expect("error while adding signer");
		assert_eq!(SyncFacade::signers().len(), 2);
		assert_eq!(SyncFacade::signers(), get_signers()[0..2]);
		assert_eq!(SyncFacade::is_signer_valid(get_signers()[0]), true);
		assert_eq!(SyncFacade::is_signer_valid(get_signers()[1]), true);
	});
}

#[test]
#[should_panic(expected = "NotAdmin")]
fn add_signer_unauthorized() {
	// Get a test environment
	let mut env = setup();

	env.execute_with(|| {
		// Add a signer
		SyncFacade::add_signer(RuntimeOrigin::signed(1), get_signers()[1])
			.expect("error while adding signer");
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
#[should_panic(expected = "NotAdmin")]
fn remove_signer_authorized() {
	// Get a test environment
	let mut env = setup();

	env.execute_with(|| {
		// Remove signer
		SyncFacade::remove_signer(RuntimeOrigin::signed(1), get_signers()[0])
			.expect("error while removing signer");
	});
}

#[test]
fn remove_signer_unauthorized() {
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

#[test]
#[should_panic(expected = "NotAdmin")]
fn set_quorum_unauthorized() {
	// Get a test environment
	let mut env = setup();

	env.execute_with(|| {
		SyncFacade::add_signer(RuntimeOrigin::root(), get_signers()[1])
			.expect("error while adding signer");
		SyncFacade::add_signer(RuntimeOrigin::root(), get_signers()[2])
			.expect("error while adding signer");
		// Set quorum; error
		SyncFacade::set_signers_quorum(RuntimeOrigin::signed(1), 3_u8)
			.expect("error while setting quorum");
	});
}

#[test]
#[should_panic(expected = "InsufficientSigners")]
fn set_quorum_authorized_insufficient_signers() {
	// Get a test environment
	let mut env = setup();

	env.execute_with(|| {
		SyncFacade::add_signer(RuntimeOrigin::root(), get_signers()[1])
			.expect("error while adding signer");
		// Set quorum; error
		SyncFacade::set_signers_quorum(RuntimeOrigin::root(), 3_u8)
			.expect("error while setting quorum");
	});
}

#[test]
fn set_quorum_authorized() {
	// Get a test environment
	let mut env = setup();

	env.execute_with(|| {
		SyncFacade::add_signer(RuntimeOrigin::root(), get_signers()[1])
			.expect("error while adding signer");
		SyncFacade::add_signer(RuntimeOrigin::root(), get_signers()[2])
			.expect("error while adding signer");
		// Set quorum; error
		SyncFacade::set_signers_quorum(RuntimeOrigin::root(), 3_u8)
			.expect("error while setting quorum");
		let quorum = SyncFacade::get_signers_quorum();
		assert_eq!(quorum, 3_u8);
	});
}

#[test]
fn sync_add_signer_events() {
	// Get a test environment
	let mut env = setup();

	let add_signer_event_1 = <SignerAdded as SignerAddedTrait>::new(get_signers()[1], 1337);

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
#[should_panic(expected = "DuplicateBatch")]
fn sync_add_signer_events_duplicate_batch() {
	// Get a test environment
	let mut env = setup();

	let add_signer_event_1 = <SignerAdded as SignerAddedTrait>::new(get_signers()[1], 1337);

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

	let add_signer_event_1 = <SignerAdded as SignerAddedTrait>::new(get_signers()[1], 1337);
	let mut events_batch = <Vec<UniversalEvent> as UniversalEventArray>::new();
	events_batch.add_signer_added_event(add_signer_event_1);

	let events_batch_hash = events_batch.compute_hash();

	let mut signature_array = <Vec<SyncSignature> as SyncSignatureArray>::new();
	signature_array.add_new_signature(
		events_batch_hash,
		U256::from("0x399ab58e2d17603eeccae95933c81d504ce475eb1bd0080d2316b84232e133c"),
		FieldElement::from(12345_u16),
	);

	let add_signer_event_2 = <SignerAdded as SignerAddedTrait>::new(get_signers()[2], 1336);
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

	let add_signer_event_1 = <SignerAdded as SignerAddedTrait>::new(get_signers()[1], 1337);
	let mut events_batch = <Vec<UniversalEvent> as UniversalEventArray>::new();
	events_batch.add_signer_added_event(add_signer_event_1);

	let events_batch_hash = events_batch.compute_hash();
	print!("batch hash in test: {}", events_batch_hash);

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
		SyncFacade::add_signer(RuntimeOrigin::root(), get_signers()[1]).expect("Error in code");
	});

	let remove_signer_event_1 = <SignerRemoved as SignerRemovedTrait>::new(get_signers()[1], 1337);

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

		assert_eq!(SyncFacade::signers().len(), 1);
		assert_eq!(SyncFacade::signers(), vec![get_signers()[0]]);
		assert_eq!(SyncFacade::is_signer_valid(get_signers()[0]), true);
		assert_eq!(SyncFacade::is_signer_valid(get_signers()[1]), false);
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
		alice_account,
		usdc().id,
		U256::from(1),
		FixedI128::from(123),
		1337,
	);
	let deposit_event_2 = <UserDeposit as UserDepositTrait>::new(
		bob_account,
		usdc().id,
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

		let alice_balance = TradingAccounts::balances(alice_account_id, usdc().id);
		let bob_balance = TradingAccounts::balances(bob_account_id, usdc().id);

		assert_eq!(alice_balance, deposit_event_1.amount);
		assert_eq!(bob_balance, deposit_event_2.amount);
	});
}
