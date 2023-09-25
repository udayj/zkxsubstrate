use crate::mock::*;
use frame_support::assert_ok;
use primitive_types::U256;
use sp_io::hashing::blake2_256;
use zkx_support::types::{Asset, BalanceUpdate, TradingAccount, TradingAccountWithoutId};

pub mod test_helper;
use test_helper::*;

fn setup() {
    let x = 1;
}


#[test]
fn basic_working() {
	assert_eq!(1, 1);
}
