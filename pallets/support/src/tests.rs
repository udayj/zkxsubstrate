use starknet_ff::FieldElement;
use sp_runtime::traits::Printable;
use crate::types::Side;
use starknet_crypto::{pedersen_hash, poseidon_hash_many, poseidon_hash};
use sp_arithmetic::fixed_point::FixedI128;
use crate:: helpers::{FixedI128_to_U256, U256_to_FieldElement, pedersen_hash_multiple};
use primitive_types::U256;
use sp_runtime::print;
use frame_support::inherent::Vec;

#[test]
fn test_fe_and_hash_values() {

    // compare field elements of diff types - done
    // compare hash of single elements - done
    // check field elements of enum type - done
    // hash of enum types - done
    // compare u256 fe - done
    // compare hash of u256 - done
    // compare fixedi128 fe - done
    // compare hash fixedi128 - done
    // compare hash of arrays - done
    // compare hash of order type
    // verify signature
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
    let u256_3 = U256::from_dec_str("3").unwrap();
    let u256_4 = U256::from_dec_str("4").unwrap();
    let u256_5 = U256::from_dec_str("5").unwrap();

    let u256_fe1 = U256_to_FieldElement(&u256_1).unwrap();
    assert_eq!(val1, u256_fe1);

    let u256_fe2 = U256_to_FieldElement(&u256_2).unwrap();
    assert_eq!(pedersen_hash(&u256_fe1, &u256_fe2),
                FieldElement::from_dec_str("2592987851775965742543459319508348457290966253241455514226127639100457844774").unwrap());
    let fixed1 = FixedI128::from_inner(-100);
    let fixed2 = FixedI128::from_inner(100);
    
    let fixed1_u256 = FixedI128_to_U256(&fixed1);
    
    let fixed2_u256 = FixedI128_to_U256(&fixed2);

    let fixed1_fe = U256_to_FieldElement(&fixed1_u256).unwrap();
    let fixed2_fe = U256_to_FieldElement(&fixed2_u256).unwrap();
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