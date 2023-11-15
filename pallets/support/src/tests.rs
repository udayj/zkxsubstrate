use crate::{
	ecdsa_verify,
	helpers::pedersen_hash_multiple,
	traits::{FixedI128Ext, Hashable, U256Ext},
	types::{HashType, Order, Side},
	Signature,
};
use frame_support::dispatch::Vec;
use primitive_types::U256;
use sp_arithmetic::fixed_point::FixedI128;
use starknet_core::crypto::ecdsa_sign;
use starknet_crypto::{get_public_key, pedersen_hash};
use starknet_ff::FieldElement;

#[test]
fn test_felt_and_hash_values() {
	let val1 = FieldElement::from(1_u8);
	let val2 = FieldElement::from(2_u128);
	let zero = FieldElement::from(0_u8);
	assert_eq!(FieldElement::from(1_u8), FieldElement::from(1_u128));
	assert_ne!(FieldElement::from(1_u8), FieldElement::from(0_u8));

	let side = Side::Buy;
	let side2 = Side::Buy;
	assert_eq!(FieldElement::from(u8::from(side)), FieldElement::from(u8::from(side2)));
	let side3 = Side::Sell;
	assert_ne!(FieldElement::from(u8::from(side)), FieldElement::from(u8::from(side3)));

	// The value of the hash is obtained from the pedersen_hash function in cairo-lang package
	// correct value = pedersen_hash(1,2)
	assert_eq!(
		pedersen_hash(&val1, &val2),
		FieldElement::from_dec_str(
			"2592987851775965742543459319508348457290966253241455514226127639100457844774"
		)
		.unwrap()
	);

	let u256_1 = U256::from_dec_str("1").unwrap();
	let u256_2 = U256::from_dec_str("2").unwrap();

	let u256_fe1 = &u256_1.try_to_felt().unwrap();
	assert_eq!(val1, *u256_fe1);

	let u256_fe2 = &u256_2.try_to_felt().unwrap();
	assert_eq!(
		pedersen_hash(&u256_fe1, &u256_fe2),
		FieldElement::from_dec_str(
			"2592987851775965742543459319508348457290966253241455514226127639100457844774"
		)
		.unwrap()
	);
	let fixed1 = FixedI128::from_inner(-100);
	let fixed2 = FixedI128::from_inner(100);

	let fixed1_u256 = fixed1.to_u256();

	let fixed2_u256 = fixed2.to_u256();

	let fixed1_fe = fixed1_u256.try_to_felt().unwrap();
	let fixed2_fe = fixed2_u256.try_to_felt().unwrap();
	assert_ne!(fixed1_fe, fixed2_fe);

	// -100 = -100 % PRIME == PRIME - 100
	assert_eq!(
		fixed1_fe,
		FieldElement::from_dec_str(
			"3618502788666131213697322783095070105623107215331596699973092056135872020381"
		)
		.unwrap()
	);
	// correct value - pedersen_hash(-100, 100)
	assert_eq!(
		pedersen_hash(&fixed1_fe, &fixed2_fe),
		FieldElement::from_dec_str(
			"680466094421187899442641443530692173273805852339864212305404387206976193972"
		)
		.unwrap()
	);
	let mut elements = Vec::new();
	elements.push(fixed1_fe);
	elements.push(fixed2_fe);
	elements.push(val1);
	elements.push(val2);

	// correct value = compute_hash_on_elements([-100,100,1,2])
	assert_eq!(
		pedersen_hash_multiple(&elements),
		FieldElement::from_dec_str(
			"1420103144340050848018289014363061324075028314390235365070247630498414256754"
		)
		.unwrap()
	);

	assert_ne!(pedersen_hash(&zero, &fixed1_fe), pedersen_hash(&zero, &fixed2_fe));
}

#[test]
fn test_order_signature() {
	let order = Order::new(201_u128, U256::from(0));

	let order_hash = order.hash(&HashType::Pedersen).unwrap();
	let expected_hash = FieldElement::from_dec_str(
		"3327157168071900049168742382147316426375465909292722120167383177610287160716",
	)
	.unwrap();
	assert_eq!(order_hash, expected_hash);

	let private_key = FieldElement::from_dec_str("100").unwrap();
	let public_key = get_public_key(&private_key);
	let signature = ecdsa_sign(&private_key, &order_hash).unwrap();
	let verification =
		ecdsa_verify(&public_key, &expected_hash, &Signature::from(signature)).unwrap();
	assert_eq!(verification, true);
}

#[test]
fn test_round_to_precision_1() {
	// 4.99, 1
	let val = FixedI128::from_inner(4990000000000000000).round_to_precision(1);
	assert_eq!(val, 5.into());

	// 4.99, 2
	let val = FixedI128::from_inner(4990000000000000000).round_to_precision(2);
	assert_eq!(val, FixedI128::from_inner(4990000000000000000));

	// 1.222, 0
	let val = FixedI128::from_inner(1222000000000000000).round_to_precision(0);
	assert_eq!(val, 1.into());

	// 1.222, 1
	let val = FixedI128::from_inner(1222000000000000000).round_to_precision(1);
	assert_eq!(val, FixedI128::from_inner(1200000000000000000));

	// 1.222, 2
	let val = FixedI128::from_inner(1222000000000000000).round_to_precision(2);
	assert_eq!(val, FixedI128::from_inner(1220000000000000000));

	// 1.222, 3
	let val = FixedI128::from_inner(1222000000000000000).round_to_precision(3);
	assert_eq!(val, FixedI128::from_inner(1222000000000000000));

	// 1.222, 18
	let val = FixedI128::from_inner(1222000000000000000).round_to_precision(18);
	assert_eq!(val, FixedI128::from_inner(1222000000000000000));

	// 1.24567, 1
	let val = FixedI128::from_inner(1245670000000000000).round_to_precision(1);
	assert_eq!(val, FixedI128::from_inner(1200000000000000000));

	// 1.24567, 2
	let val = FixedI128::from_inner(1245670000000000000).round_to_precision(2);
	assert_eq!(val, FixedI128::from_inner(1250000000000000000));

	// 1.24567, 3
	let val = FixedI128::from_inner(1245670000000000000).round_to_precision(3);
	assert_eq!(val, FixedI128::from_inner(1246000000000000000));

	// 1.24567, 4
	let val = FixedI128::from_inner(1245670000000000000).round_to_precision(4);
	assert_eq!(val, FixedI128::from_inner(1245700000000000000));

	// 100, 1
	let val = FixedI128::from_inner(100000000000000000000).round_to_precision(1);
	assert_eq!(val, 100.into());

	// 100, 2
	let val = FixedI128::from_inner(100000000000000000000).round_to_precision(2);
	assert_eq!(val, 100.into());

	// -2.345, 2
	let val = FixedI128::from_inner(-2345000000000000000).round_to_precision(2);
	assert_eq!(val, FixedI128::from_inner(-2350000000000000000));

	// -2.345, 1
	let val = FixedI128::from_inner(-2345000000000000000).round_to_precision(1);
	assert_eq!(val, FixedI128::from_inner(-2300000000000000000));

	// -2.345, 0
	let val = FixedI128::from_inner(-2345000000000000000).round_to_precision(0);
	assert_eq!(val, FixedI128::from_inner(-2000000000000000000));

	// 0.001, 3
	let val = FixedI128::from_inner(1000000000000000).round_to_precision(3);
	assert_eq!(val, FixedI128::from_inner(1000000000000000));

	// 0.001, 1
	let val = FixedI128::from_inner(1000000000000000).round_to_precision(1);
	assert_eq!(val, 0.into());

	// 0.156, 1
	let val = FixedI128::from_inner(156000000000000000).round_to_precision(1);
	assert_eq!(val, FixedI128::from_inner(200000000000000000));

	// 0.156, 2
	let val = FixedI128::from_inner(156000000000000000).round_to_precision(2);
	assert_eq!(val, FixedI128::from_inner(160000000000000000));

	// 0.156, 3
	let val = FixedI128::from_inner(156000000000000000).round_to_precision(3);
	assert_eq!(val, FixedI128::from_inner(156000000000000000));

	// 10000000000000.98123456789, 6
	let val = FixedI128::from_inner(10000000000000981234567890000000).round_to_precision(6);
	assert_eq!(val, FixedI128::from_inner(10000000000000981235000000000000));

	// 10000000000000.98123456789, 3
	let val = FixedI128::from_inner(10000000000000981234567890000000).round_to_precision(3);
	assert_eq!(val, FixedI128::from_inner(10000000000000981000000000000000));

	// 10000000000000.98123456789, 2
	let val = FixedI128::from_inner(10000000000000981234567890000000).round_to_precision(2);
	assert_eq!(val, FixedI128::from_inner(10000000000000980000000000000000));

	// 10000000000000.98123456789, 1
	let val = FixedI128::from_inner(10000000000000981234567890000000).round_to_precision(1);
	assert_eq!(val, FixedI128::from_inner(10000000000001000000000000000000));

	// 1467.0000001, 3
	let val = FixedI128::from_inner(14670000001000000000).round_to_precision(3);
	assert_eq!(val, FixedI128::from_inner(14670000000000000000));

	// 756.99999999, 4
	let val = FixedI128::from_inner(756999999990000000000).round_to_precision(4);
	assert_eq!(val, FixedI128::from_inner(757000000000000000000));

	// 10000000000000.123456789123456789, 18
	let val = FixedI128::from_inner(10000000000000123456789123456789).round_to_precision(18);
	assert_eq!(val, FixedI128::from_inner(10000000000000123456789123456789));

	// 10000000000000.123456789123456789, 16
	let val = FixedI128::from_inner(10000000000000123456789123456789).round_to_precision(16);
	assert_eq!(val, FixedI128::from_inner(10000000000000123456789123456800));

	// 10000000000000.123456789123456789, 1
	let val = FixedI128::from_inner(10000000000000123456789123456789).round_to_precision(1);
	assert_eq!(val, FixedI128::from_inner(10000000000000100000000000000000));
}
