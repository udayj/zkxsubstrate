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
			return FixedI128::one()
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

	// Function to recompute volume vector based on the difference in days between current trade and last trade
	// The difference in days is represented by the shift_value
	// new_volume for current day is added to the 1st element of volume vector which represents the cumulative volume 
	// for present day
	pub fn shift_and_recompute(volume_array: &Vec<FixedI128>, new_volume: FixedI128, shift_value: usize) -> (Vec<FixedI128>, FixedI128) {

		let mut updated_volume_array: Vec<FixedI128>;
		if shift_value > 30 {
			// No trades have happened in last 30 days
			updated_volume_array = Vec::from([FixedI128::from_inner(0);31]);
		}
		else {
			updated_volume_array = volume_array.clone();
			// Based on the difference in no. of days, zero out the volume for those many days
			// Shift the volume vector prior to zeroing days of no trade
			updated_volume_array.rotate_right(shift_value);
			for i in 0..shift_value {
				if let Some(elem) = updated_volume_array.get_mut(i) {
					*elem = FixedI128::from_inner(0);
				}
			}
		}
		// add new volume to present day cumulative volume which is stored in index 0 of the volume vector
		let mut present_day_volume = updated_volume_array.get_mut(0).unwrap();
		*present_day_volume = present_day_volume.clone().add(new_volume);

		let total_volume = calc_30day_volume(&updated_volume_array);
		(updated_volume_array, total_volume)
	}

	// Function to calculate difference in days between two timestamps
	// Assumes that timestamp_cur > timestamp_prev
	pub fn get_day_diff(timestamp_prev: u64, timestamp_cur: u64) -> usize {

		// We use timestamp start as a dummy reference value to calculate day no. for any given timestamp
		// We do this since we are only intereted in the relative difference between given timestamps
		let timestamp_start:u64 = 1701129600; // Unix timestamp for 28th Nov 12:00 AM UTC
		let one_day = 24*60*60;
		let day_prev = (timestamp_prev-timestamp_start)/(one_day);
		let day_cur = (timestamp_cur-timestamp_start)/(one_day);
		return (day_cur - day_prev) as usize;
	}

	// Function to calculate 30day volume from volume vector
	pub fn calc_30day_volume(volume_array: &Vec<FixedI128>) -> FixedI128 {

		let mut total_volume: FixedI128 = FixedI128::from_inner(0);
		// start from 2nd element since 1st element stores volume for current day
		for elem in &volume_array[1..] {
			total_volume = total_volume.add(elem.clone());
		}
		total_volume
	}
}
