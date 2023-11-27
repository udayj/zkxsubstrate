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
	use core::f64;
	use frame_support::dispatch::Vec;
	use itertools::fold;
	use libm::log10;
	use primitive_types::U256;
	use sp_arithmetic::{fixed_point::FixedI128, traits::One};
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

	pub fn ln(x: FixedI128) -> FixedI128 {
		let val = x.into_inner();
		let f_val: f64 = val as f64;
		let log10_div = 18 as f64; // DIV for FixedI128 is 10^18, hence log10(DIV) = 18
		let log10_val = log10(f_val) - log10_div as f64;
		let ln_val = log10_val * f64::consts::LN_10;
		FixedI128::from((ln_val * 10_u128.pow(18) as f64) as i128)
	}
}
