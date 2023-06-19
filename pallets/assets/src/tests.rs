use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};

#[test]
fn it_works_for_default_value() {
    new_test_ext().execute_with(|| {
        // Go past genesis block so events get deposited
        System::set_block_number(1);
        // Dispatch a signed extrinsic.
        assert_ok!(AssetModule::add_asset(RuntimeOrigin::signed(1), 1, 123, false, true, 6));
        assert_ok!(AssetModule::modify_default_collateral(RuntimeOrigin::signed(1), 1));
        // Read pallet storage and assert an expected result.
        assert_eq!(AssetModule::default_collateral_asset(), 1);
        // Assert that the correct event was deposited
        System::assert_last_event(Event::DefaultCollateralModified { id: 1 }.into());
    });
}

