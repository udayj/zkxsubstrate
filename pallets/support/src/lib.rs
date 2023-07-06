#![cfg_attr(not(feature = "std"), no_std)]

pub mod types;

pub mod traits;

pub fn str_to_felt(text: &str) -> u64 {
    let b_text = text.as_bytes();
    let mut result: u64 = 0;

    for &byte in b_text {
        result = (result << 8) + u64::from(byte);
    }

    result
}