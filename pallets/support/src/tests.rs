use starknet_ff::FieldElement;
use sp_runtime::traits::Printable;
use crate::types::Side;
use starknet_crypto::{pedersen_hash, poseidon_hash_many, poseidon_hash};
use sp_arithmetic::fixed_point::FixedI128;
use crate:: {FixedI128_to_U256, pedersen_hash_multiple};
use primitive_types::U256;
use sp_runtime::print;
use frame_support::inherent::Vec;

#[test]
fn test_fe_values() {

    let val1 = FieldElement::from(1_u8);
    let val2 = FieldElement::from(2_u128);
    let zero = FieldElement::from(0_u8);
    assert_eq!(FieldElement::from(1_u8), FieldElement::from(1_u128));
    let side = Side::Buy;
    let side2 = Side::Buy;
    assert_eq!(FieldElement::from(u8::from(side)), FieldElement::from(u8::from(side2)));
    assert_ne!(pedersen_hash(&val1, &val2), pedersen_hash(&val2, &val1));
    assert_eq!(U256::from_dec_str("100").unwrap(), U256::from_dec_str("100").unwrap());

    let u256_1 = U256::from_dec_str("1").unwrap();
    let u256_2 = U256::from_dec_str("2").unwrap();
    let u256_3 = U256::from_dec_str("3").unwrap();
    let u256_4 = U256::from_dec_str("4").unwrap();
    let u256_5 = U256::from_dec_str("5").unwrap();

    let fixed1 = FixedI128::from_inner(-10000000000);
    let fixed2 = FixedI128::from_inner(20000000000);
    //let fixed3 = fixed1.into_inner()*(-1_i128);
    //let fixed4 = -10000000000;
    //assert_eq!(fixed3, fixed4);
    "check1".print();
    let fixed1_u256 = FixedI128_to_U256(fixed1);
    "check2".print();
    let fixed2_u256 = FixedI128_to_U256(fixed2);

    //assert_eq!(fixed1_u256, fixed2_u256);
    let mut buffer1:[u8;32] = [0;32];
    let mut buffer2:[u8;32] = [0;32];

    u256_1.to_big_endian(&mut buffer1);
    u256_2.to_big_endian(&mut buffer2);
    let fe1 = FieldElement::from_byte_slice_be(&buffer1).unwrap();
    let fe2 = FieldElement::from_byte_slice_be(&buffer2).unwrap();
    u256_3.to_big_endian(&mut buffer1);
    u256_4.to_big_endian(&mut buffer2);
    let fe3 = FieldElement::from_byte_slice_be(&buffer1).unwrap();
    let fe4 = FieldElement::from_byte_slice_be(&buffer2).unwrap();
    let mut elements = Vec::new();
    elements.push(fe1);
    elements.push(fe2);
    elements.push(fe3);
    elements.push(fe4);
    assert_eq!(pedersen_hash(&zero, &fe1), poseidon_hash_many(&elements));

    fixed1_u256.to_big_endian(&mut buffer1);
    fixed2_u256.to_big_endian(&mut buffer2);

    let fixed1_fe = FieldElement::from_byte_slice_be(&buffer1).unwrap();
    let fixed2_fe = FieldElement::from_byte_slice_be(&buffer2).unwrap();

    assert_eq!(pedersen_hash(&zero, &fixed1_fe), pedersen_hash(&zero, &fixed2_fe));

}