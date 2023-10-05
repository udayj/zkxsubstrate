#![cfg_attr(not(feature = "std"), no_std)]

// Re-export ecdsa_verify to be used as is
pub use starknet_core::crypto::{ecdsa_sign, ecdsa_verify, Signature};
pub use starknet_ff::{FieldElement, FromByteSliceError};

// Custom types and data structures.
pub mod types;

// Tests for support pallet
#[cfg(test)]
mod tests;

// Trait definitions.
pub mod traits;

// Test Helpers
pub mod test_helpers;

// helper fns to be used by other pallets
pub mod helpers {
	use super::{FieldElement, FromByteSliceError};
	use crate::traits::U256Ext;
	use frame_support::inherent::Vec;
	use itertools::fold;
	use primitive_types::U256;
	use starknet_crypto::pedersen_hash;

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
		let sig_r_felt = sig_r.try_to_felt()?;
		let sig_s_felt = sig_s.try_to_felt()?;
		Ok((sig_r_felt, sig_s_felt))
	}
}
