#![cfg_attr(not(feature = "std"), no_std)]

use crate::traits::FixedI128Ext;
use primitive_types::U256;
use sp_arithmetic::{fixed_point::FixedI128, traits::CheckedDiv, FixedPointNumber};
use starknet_ff::FromByteSliceError;

// Re-export ecdsa_verify to be used as is
pub use starknet_core::crypto::{ecdsa_verify, Signature};
pub use starknet_ff::FieldElement;

// Custom types and data structures.
pub mod types;

#[cfg(test)]
mod tests;

/// Trait definitions.
pub mod traits;

impl FixedI128Ext<FixedI128> for FixedI128 {
	fn round_to_precision(t: FixedI128, precision: u32) -> FixedI128 {
		// Get the inner value (number * 10^18) from FixedI128 representation
		let inner_value: i128 = FixedI128::into_inner(t);
		// Get the integer part and decimal part separately
		let divisor: i128 = 10_u64.pow(18).into();
		let integer_part: i128 = inner_value / divisor;
		let decimal_part: i128 = inner_value % divisor;
		// Multiply decimal part with (10 ^ precision) and round it
		// so that now we have the required precision
		let ten_power_precision: i128 = 10_u64.pow(precision).into();
		let decimal_part: FixedI128 = FixedI128::from_inner(decimal_part * ten_power_precision);
		let decimal_part_rounded: FixedI128 = decimal_part.round();

		// Divide the decimal part with (10 ^ precision)
		// so that we get it to required decimal representaion
		let ten_power_precision: FixedI128 =
			FixedI128::checked_from_integer(ten_power_precision).unwrap();
		let decimal_part: FixedI128 =
			decimal_part_rounded.checked_div(&ten_power_precision).unwrap();
		let integer_part: FixedI128 = FixedPointNumber::checked_from_integer(integer_part).unwrap();
		// Add both the parts together to get the final result
		let res: FixedI128 = FixedI128::add(integer_part, decimal_part);
		res
	}
}

pub mod helpers {
	use super::{FieldElement, FixedI128, FixedPointNumber, FromByteSliceError, U256};
	use frame_support::inherent::Vec;
	use itertools::fold;
	use starknet_crypto::pedersen_hash;

	pub fn str_to_felt(text: &str) -> u128 {
		let a = FieldElement::from_byte_slice_be(text.as_bytes());
		u128::try_from(a.unwrap()).unwrap()
	}

	// Function to convert from fixed point number to U256 inside the PRIME field
	// This function does the appropriate mod arithmetic to ensure the returned value is actually less than PRIME
	pub fn fixed_i128_to_u256(val: &FixedI128) -> U256 {
		let inner_val: U256;
		// Max prime 2^251 + 17*2^192 + 1
		let prime: U256 = U256::from_dec_str(
			"3618502788666131213697322783095070105623107215331596699973092056135872020481",
		)
		.unwrap();
		// If the fixed point number is positive, we directly convert the inner val to U256
		if !val.is_negative() {
			inner_val = U256::from(val.into_inner());
		} else {
			// If the fixed point number is negative then we need to wrap the value
			// i.e. -x is equivalent to PRIME - x (or -x % PRIME) where x is a positive number
			inner_val = prime - (U256::from(-val.into_inner()));
		}
		inner_val
	}

	pub fn u256_to_field_element(val: &U256) -> Result<FieldElement, FromByteSliceError> {
		let mut buffer: [u8; 32] = [0; 32];
		val.to_big_endian(&mut buffer);
		FieldElement::from_byte_slice_be(&buffer)
	}

	// Function to perform pedersen hash of an array of field elements
	pub fn pedersen_hash_multiple(data: &Vec<FieldElement>) -> FieldElement {
		// hash is computed as follows
		// h(h(h(h(0, data[0]), data[1]), ...), data[n-1]), n).
		let first_element = FieldElement::from(0_u8);
		let last_element = FieldElement::from(data.len());

		// append length of data array to the array
		let mut elements = data.clone();
		elements.push(last_element);

		// The FieldElement array is then reduced with the pedersen hash function
		fold(&elements, first_element, foldable_pedersen_hash)
	}

	// This function is a wrapper around pedersen_hash function where 1st element is consumable
	// instead of being borrowed through a reference
	// It is required due to the fn sig requirement of fold
	fn foldable_pedersen_hash(a: FieldElement, b: &FieldElement) -> FieldElement {
		pedersen_hash(&a, b)
	}

	pub fn sig_u256_to_sig_felt(
		sig_r: &U256,
		sig_s: &U256,
	) -> Result<(FieldElement, FieldElement), FromByteSliceError> {
		let sig_r_felt = u256_to_field_element(sig_r)?;
		let sig_s_felt = u256_to_field_element(sig_s)?;
		Ok((sig_r_felt, sig_s_felt))
	}
}
