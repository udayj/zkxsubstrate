use primitive_types::U256;
use zkx_support::helpers::pedersen_hash_multiple;
use zkx_support::types::{
	TradingAccountMinimal, UniversalEventL2, UserDepositL2,
};
use zkx_support::{FieldElement, helpers};

pub mod test_helper;
use test_helper::*;

fn setup() {
	let x = 1;
}

#[test]
fn basic_working() {
	let alice_trading_account_0 =
		TradingAccountMinimal { account_address: U256::from(100), index: 0 };

	// let deposit_event = UserDepositL2::new(
	// 	alice_trading_account_0,
	// 	123,
	// 	U256::from(0),
	// 	U256::from(12),
	// 	U256::from(12),
	// 	1337,
	// );
	// let events_batch: Vec<UniversalEventL2> =
	// 	<Vec<UniversalEventL2> as UniversalEventArray>::new().add_user_deposit_event(deposit_event);

	// let events_hash = events_batch.compute_hash();

	let arr: Vec<FieldElement> = vec![FieldElement::from(1_u32), FieldElement::from(2_u32)];
	let hashed_value = pedersen_hash_multiple(&arr);

	println!("Computed hash on L3: {hashed_value}");
}
