use crate::mock::*;
use frame_support::{assert_ok, dispatch::Vec};
use primitive_types::U256;
use sp_arithmetic::{fixed_point::FixedI128, traits::Zero};
use sp_io::hashing::blake2_256;
use sp_runtime::{traits::ConstU32, BoundedVec};

fn setup() -> sp_io::TestExternalities {
	// Create a new test environment
	let mut test_env = new_test_ext();

	// Set the signers using admin account
	test_env.execute_with(|| {
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
		let prices = vec![FixedI128::from(1), FixedI128::from(1)];
		let result = ABRModule::calculate_sliding_mean(&prices, 8);
	});
}
