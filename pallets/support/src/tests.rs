use crate::{
	ecdsa_verify,
	helpers::{calc_30day_volume, compute_hash_on_elements, get_day_diff, shift_and_recompute},
	traits::{FixedI128Ext, Hashable, U256Ext},
	types::{HashType, Order, Side},
	Signature,
};
use codec::alloc::vec;
use frame_support::dispatch::Vec;
use hex;
use primitive_types::U256;
use sp_arithmetic::fixed_point::FixedI128;
use starknet_core::crypto::ecdsa_sign;
use starknet_crypto::{get_public_key, pedersen_hash};
use starknet_ff::{FieldElement, FromByteSliceError};

// reference implementation to test conversion from string to FieldElement
pub fn hex_to_field_element(text: &str) -> Result<FieldElement, FromByteSliceError> {
	let cleaned_hex_string = text.trim_start_matches("0x");
	let mut bytes_vec =
		hex::decode(cleaned_hex_string).map_err(|_err| FromByteSliceError::InvalidLength)?;
	while bytes_vec.len() < 32 {
		bytes_vec.insert(0, 0);
	}
	let bytes_array = bytes_vec[..32].try_into().expect("Wrong length for bytes array");
	FieldElement::from_bytes_be(&bytes_array).map_err(|_err| FromByteSliceError::InvalidLength)
}

pub fn string_to_felt(str: &str) -> Result<FieldElement, FromByteSliceError> {
	if !str.is_ascii() {
		return Err(FromByteSliceError::OutOfRange)
	}
	if str.len() > 31 {
		return Err(FromByteSliceError::InvalidLength)
	}
	let encoded_str = hex::encode(str);

	hex_to_field_element(&encoded_str)
}
#[test]
fn test_enum_felt() {
	let a = FieldElement::from_hex_be(hex::encode("BUY").as_str()).unwrap();
	let b = string_to_felt("BUY").unwrap();
	assert_eq!(a, b, "Error");

	let a = FieldElement::from_hex_be(hex::encode("LONG").as_str()).unwrap();
	let b = string_to_felt("LONG").unwrap();
	assert_eq!(a, b, "Error");

	let a = FieldElement::from_hex_be(hex::encode("GTT").as_str()).unwrap();
	let b = string_to_felt("GTT").unwrap();
	assert_eq!(a, b, "Error");

	let a = FieldElement::from_hex_be(hex::encode("MARKET").as_str()).unwrap();
	let b = string_to_felt("MARKET").unwrap();
	assert_eq!(a, b, "Error");
}
#[test]
fn test_felt_and_hash_values() {
	let val1 = FieldElement::from(1_u8);
	let val2 = FieldElement::from(2_u128);
	let zero = FieldElement::from(0_u8);
	assert_eq!(FieldElement::from(1_u8), FieldElement::from(1_u128));
	assert_ne!(FieldElement::from(1_u8), FieldElement::from(0_u8));

	let side = Side::Buy;
	let side_str: &str = side.into();
	let side2 = Side::Buy;
	let side2_str: &str = side2.into();

	assert_eq!(
		FieldElement::from_hex_be(hex::encode(side_str).as_str()).unwrap(),
		FieldElement::from_hex_be(hex::encode(side2_str).as_str()).unwrap()
	);
	let side3 = Side::Sell;
	let side3_str: &str = side3.into();
	assert_ne!(
		FieldElement::from_hex_be(hex::encode(side_str).as_str()).unwrap(),
		FieldElement::from_hex_be(hex::encode(side3_str).as_str()).unwrap()
	);

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
		compute_hash_on_elements(&elements),
		FieldElement::from_dec_str(
			"1420103144340050848018289014363061324075028314390235365070247630498414256754"
		)
		.unwrap()
	);

	assert_ne!(pedersen_hash(&zero, &fixed1_fe), pedersen_hash(&zero, &fixed2_fe));
}

#[test]
fn test_order_signature() {
	let order = Order::new(U256::from(201), U256::from(0));
	let order_hash = order.hash(&HashType::Pedersen).unwrap();
	let expected_hash = FieldElement::from_dec_str(
		"3132625918282695035920415711376638693136677687288415900988049051810724895775",
	)
	.unwrap();
	// order hash should match
	// compute_hash_on_elements(
	// [0,0,201,0,1,327647316308,1280265799,4347225,
	// 100000000000000000000,1000000000000000000,1000000000000000000,100000000000000000,0,4674627,
	// 1699940278000])
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

#[test]
fn test_calc_30day_volume() {
	let mut volume: Vec<FixedI128> = vec![];
	for i in 0..31 {
		let element: FixedI128 = i.into();
		volume.push(element);
	}
	assert_eq!(calc_30day_volume(&volume), 465.into(), "Error in calculating volume");
}

#[test]
fn test_get_day_diff() {
	let t_prev = 1701880189;
	let mut t_cur = 1701880189;
	assert_eq!(get_day_diff(t_prev, t_cur), 0, "Error in day diff");
	t_cur = 1701883789;
	assert_eq!(get_day_diff(t_prev, t_cur), 0, "Error in day diff");
	t_cur = 1701912589;
	assert_eq!(get_day_diff(t_prev, t_cur), 1, "Error in day diff");
	t_cur = 1701991789;
	assert_eq!(get_day_diff(t_prev, t_cur), 1, "Error in day diff");
	t_cur = 1701993601;
	assert_eq!(get_day_diff(t_prev, t_cur), 2, "Error in day diff");
}

#[test]
fn test_shift_and_recompute() {
	let mut volume: Vec<FixedI128> = vec![];
	for i in 0..31 {
		let element: FixedI128 = i.into();
		volume.push(element);
	}

	// trade in same day with 0 new volume
	let (updated_volume, total_30day_volume) = shift_and_recompute(&volume, 0.into(), 0);
	assert_eq!(updated_volume, volume, "Error in updated volume 1");
	assert_eq!(total_30day_volume, 465.into(), "Error in calculating volume 1");

	// trade in same day with 100 new volume
	let (updated_volume, total_30day_volume) = shift_and_recompute(&volume, 100.into(), 0);

	let mut new_volume = vec![];
	for i in 1..31 {
		let element: FixedI128 = i.into();
		new_volume.push(element);
	}
	new_volume.insert(0, 100.into());
	assert_eq!(updated_volume, new_volume, "Error in updated volume 2");
	assert_eq!(total_30day_volume, 465.into(), "Error in calculating volume 2");

	// trade on next day with 0 new volume
	let (updated_volume, total_30day_volume) = shift_and_recompute(&volume, 0.into(), 1);

	let mut new_volume = vec![];
	for i in 0..30 {
		let element: FixedI128 = i.into();
		new_volume.push(element);
	}
	new_volume.insert(0, 0.into());
	assert_eq!(updated_volume, new_volume, "Error in updated volume 3");
	assert_eq!(total_30day_volume, 435.into(), "Error in calculating volume 3");

	// trade on next day with 100 new volume
	let (updated_volume, total_30day_volume) = shift_and_recompute(&volume, 100.into(), 1);

	let mut new_volume = vec![];
	for i in 0..30 {
		let element: FixedI128 = i.into();
		new_volume.push(element);
	}
	new_volume.insert(0, 100.into());
	assert_eq!(updated_volume, new_volume, "Error in updated volume 4");
	assert_eq!(total_30day_volume, 435.into(), "Error in calculating volume 4");

	// trade after 2 days with 100 new volume
	let (updated_volume, total_30day_volume) = shift_and_recompute(&volume, 100.into(), 2);

	let mut new_volume = vec![];
	for i in 0..29 {
		let element: FixedI128 = i.into();
		new_volume.push(element);
	}
	new_volume.insert(0, 0.into());
	new_volume.insert(0, 100.into());
	assert_eq!(updated_volume, new_volume, "Error in updated volume 5");
	assert_eq!(total_30day_volume, 406.into(), "Error in calculating volume 5");

	// trade after 31 days with 100 new volume
	let (updated_volume, total_30day_volume) = shift_and_recompute(&volume, 100.into(), 31);

	let mut new_volume = Vec::from([FixedI128::from_inner(0); 30]);
	new_volume.insert(0, 100.into());
	assert_eq!(updated_volume, new_volume, "Error in updated volume 6");
	assert_eq!(total_30day_volume, 0.into(), "Error in calculating volume 6");
}
