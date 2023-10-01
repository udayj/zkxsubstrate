use crate::{mock::*, Event};
use super::*;
    use frame_support::{assert_ok, assert_noop, assert_err, dispatch::DispatchError};

#[test]
fn add_signer_works() {
	// Initialize a test environment
	new_test_ext().execute_with(|| {
		// Ensure the initial signer list is empty
		assert_eq!(SyncFacade::Signers::get().len(), 0);
	});
}
