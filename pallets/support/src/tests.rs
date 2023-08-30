use starknet_ff::FieldElement;
use starknet_crypto::{pedersen_hash, poseidon_hash_many, poseidon_hash, get_public_key, Signature};
use starknet_core::crypto::{ecdsa_sign};
use sp_arithmetic::fixed_point::FixedI128;
use crate::helpers::{fixed_i128_to_u256, u256_to_field_element, pedersen_hash_multiple};
use crate::types::{ Side, Order, OrderType, Direction, TimeInForce, HashType};
use crate::traits::Hashable;
use crate::ecdsa_verify;
use primitive_types::U256;
use frame_support::inherent::Vec;

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
    assert_eq!(pedersen_hash(&val1, &val2),
             FieldElement::from_dec_str("2592987851775965742543459319508348457290966253241455514226127639100457844774").unwrap());

    let u256_1 = U256::from_dec_str("1").unwrap();
    let u256_2 = U256::from_dec_str("2").unwrap();

    let u256_fe1 = u256_to_field_element(&u256_1).unwrap();
    assert_eq!(val1, u256_fe1);

    let u256_fe2 = u256_to_field_element(&u256_2).unwrap();
    assert_eq!(pedersen_hash(&u256_fe1, &u256_fe2),
                FieldElement::from_dec_str("2592987851775965742543459319508348457290966253241455514226127639100457844774").unwrap());
    let fixed1 = FixedI128::from_inner(-100);
    let fixed2 = FixedI128::from_inner(100);
    
    let fixed1_u256 = fixed_i128_to_u256(&fixed1);
    
    let fixed2_u256 = fixed_i128_to_u256(&fixed2);

    let fixed1_fe = u256_to_field_element(&fixed1_u256).unwrap();
    let fixed2_fe = u256_to_field_element(&fixed2_u256).unwrap();
    assert_ne!(fixed1_fe, fixed2_fe);

    // -100 = -100 % PRIME == PRIME - 100
    assert_eq!(fixed1_fe, 
                FieldElement::from_dec_str("3618502788666131213697322783095070105623107215331596699973092056135872020381").unwrap());
    // correct value - pedersen_hash(-100, 100)
    assert_eq!(pedersen_hash(&fixed1_fe, &fixed2_fe),
                FieldElement::from_dec_str("680466094421187899442641443530692173273805852339864212305404387206976193972").unwrap());
    let mut elements = Vec::new();
    elements.push(fixed1_fe);
    elements.push(fixed2_fe);
    elements.push(val1);
    elements.push(val2);

    // correct value = compute_hash_on_elements([-100,100,1,2])
    assert_eq!(pedersen_hash_multiple(&elements), 
            FieldElement::from_dec_str("1420103144340050848018289014363061324075028314390235365070247630498414256754").unwrap());

    assert_ne!(pedersen_hash(&zero, &fixed1_fe), pedersen_hash(&zero, &fixed2_fe));

}

#[test]
fn test_order_signature() {

    let order = Order {
        account_id: U256::from_dec_str("100").unwrap(),
        order_id: 200_u128,
        market_id: U256::from_dec_str("300").unwrap(),
        order_type: OrderType::Market,
        direction: Direction::Long,
        side: Side::Buy,
        price: FixedI128::from_inner(10000000_i128),
        size: FixedI128::from_inner(01_i128),
        leverage: FixedI128::from_inner(-100_i128),
        slippage: FixedI128::from_inner(-200_i128),
        post_only: true,
        time_in_force: TimeInForce::GTC
    };

    let order_hash = order.hash(HashType::Pedersen).unwrap();

    // correct value of order_hash is the hash as calculated using compute_hash_on_elements (from cairo-lang package) using the 
    // serialized values of the different types
    // compute_hash_on_elements([100,200,300,1,0,0,10000000,0x800000000000010ffffffffffffffffffffffffffffffffffffffffffffff9d,
    // 0x800000000000010ffffffffffffffffffffffffffffffffffffffffffffff39,1,0])
    let expected_hash = FieldElement::from_dec_str("779455944553865873074074863659363906459964867916460440519908583353736546068").unwrap();
    assert_eq!(order_hash, expected_hash);

    let private_key = FieldElement::from_dec_str("100").unwrap();
    let public_key = get_public_key(&private_key);
    let signature = ecdsa_sign(&private_key, &order_hash).unwrap();
    let verification = ecdsa_verify(
        &public_key, 
        &expected_hash, 
        &Signature::from(signature)).unwrap();
    assert_eq!(verification, true);

}