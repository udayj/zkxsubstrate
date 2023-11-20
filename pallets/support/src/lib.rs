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
	use frame_support::dispatch::Vec;
	use itertools::fold;
	use primitive_types::U256;
	use sp_arithmetic::{
		fixed_point::FixedI128,
		traits::{One, Zero},
	};
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

	pub fn max(a: FixedI128, b: FixedI128) -> FixedI128 {
		if a >= b {
			a
		} else {
			b
		}
	}

	fn factorial(n: u64) -> FixedI128 {
		(1..=n).fold(FixedI128::one(), |acc, x| acc * FixedI128::from(x as i128))
	}

	pub fn fixed_pow(base: FixedI128, exp: u64) -> FixedI128 {
		if exp == 0 {
			// Anything raised to the power of 0 is 1
			return FixedI128::one();
		}

		let mut result = FixedI128::one();
		let mut current_base = base;

		let mut remaining_exp = exp;

		while remaining_exp > 0 {
			if remaining_exp % 2 == 1 {
				result = result * current_base;
			}

			current_base = current_base * current_base;
			remaining_exp /= 2;
		}

		result
	}

	fn power_series(x: FixedI128, terms: u64) -> FixedI128 {
		(0..terms).fold(FixedI128::zero(), |acc, i| {
			let term = fixed_pow(x, i) / factorial(i);
			acc + term
		})
	}

	pub fn ln(x: FixedI128) -> FixedI128 {
		if x <= FixedI128::zero() {
			// Logarithm is undefined for non-positive numbers
			panic!("Natural logarithm is undefined for non-positive numbers.");
		}

		// ln(x) = (x - 1) - (x - 1)^2/2 + (x - 1)^3/3 - (x - 1)^4/4 + ...
		(x - FixedI128::one()) * power_series((x - FixedI128::one()) / x, 10_u64)
	}
}
