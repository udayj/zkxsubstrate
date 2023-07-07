#![cfg_attr(not(feature = "std"), no_std)]
use starknet_ff::FieldElement;

pub mod types;

pub mod traits;

pub fn str_to_felt(text: &str) -> u64 {
    let a = FieldElement::from_byte_slice_be(text.as_bytes());
    u64::try_from(a.unwrap()).unwrap()
}