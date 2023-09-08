#![cfg_attr(not(feature = "std"), no_std)]

// Re-export ecdsa_verify to be used as is
pub use starknet_core::crypto::{ecdsa_verify, Signature};
pub use starknet_ff::{FieldElement, FromByteSliceError};

// Custom types and data structures.
pub mod types;

// Tests for support pallet
#[cfg(test)]
mod tests;

// Trait definitions.
pub mod traits;

// helper fns to be used by other pallets
pub mod helpers {
	use super::{FieldElement, FromByteSliceError};
	use frame_support::inherent::Vec;
	use itertools::fold;
	use primitive_types::U256;
	use sp_arithmetic::{fixed_point::FixedI128, FixedPointNumber};
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

	pub fn field_element_to_u256(val: FieldElement) -> U256 {
		let buffer = val.to_bytes_be();
		U256::from_big_endian(&buffer)
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
