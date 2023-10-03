use super::*;
use crate::{mock::*, Event};

#[test]
fn add_signer_works() {
	// Initialize a test environment
	new_test_ext().execute_with(|| {
		// Ensure the initial signer list is empty
		assert_eq!(SyncFacade::accounts_count().len(), 0);
		let a = SyncFacade::accounts_count().len();
		println!("{:?}", a);
	});
}
