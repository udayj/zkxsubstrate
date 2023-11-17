#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use pallet_support::{
		helpers::{fixed_pow, ln, max},
		traits::TradingAccountInterface,
	};
	use sp_arithmetic::{fixed_point::FixedI128, traits::Zero};

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type TradingAccountPallet: TradingAccountInterface;
	}

	impl<T: Config> Pallet<T> {
		fn calculate_effective_abr(premiums: &[FixedI128]) -> FixedI128 {
			let mut premium_sum = FixedI128::zero();
			let total_len = premiums.len();

			for iterator in 0..total_len {
				premium_sum = premium_sum + premiums[iterator];
			}

			premium_sum / (FixedI128::from((total_len * 8) as i128))
		}

		fn calculate_premium(
			mark_prices: &[FixedI128],
			index_prices: &[FixedI128],
		) -> Vec<FixedI128> {
			// Initialize the result vector with the size of prices vector
			let total_len = mark_prices.len();
			let mut premiums = Vec::<FixedI128>::with_capacity(total_len);

			for iterator in 0..total_len {
				// TODO(merkle-groot): possibly check for division by zero error here
				premiums[iterator] =
					(mark_prices[iterator] - index_prices[iterator]) / mark_prices[iterator];
			}

			premiums
		}

		fn calculate_jump(
			premiums: &mut [FixedI128],
			upper_band: &[FixedI128],
			lower_band: &[FixedI128],
			mark_prices: &[FixedI128],
			index_prices: &[FixedI128],
		) -> Vec<FixedI128> {
			let total_len = mark_prices.len();

			for iterator in 0..total_len {
				let upper_diff =
					max(FixedI128::zero(), mark_prices[iterator] - upper_band[iterator]);
				let lower_diff =
					max(FixedI128::zero(), lower_band[iterator] - mark_prices[iterator]);

				if upper_diff > FixedI128::zero() {
					premiums[iterator] = premiums[iterator] +
						max(ln(upper_diff) / index_prices[iterator], FixedI128::zero());
				} else if lower_diff > FixedI128::zero() {
					premiums[iterator] = premiums[iterator] -
						max(ln(lower_diff) / index_prices[iterator], FixedI128::zero());
				}
			}

			premiums.to_vec()
		}

		fn calculate_std(
			prices: &[FixedI128],
			mean: FixedI128,
			boll_width: FixedI128,
		) -> FixedI128 {
			// Initialize the diff_sum
			let total_len = prices.len();
			let mut diff_sum = FixedI128::zero();

			// Handle Edge case
			if total_len == 0 {
				return FixedI128::zero();
			}

			for iterator in 0..total_len {
				let diff = prices[iterator] - mean;
				diff_sum = diff_sum + fixed_pow(diff, 2_u64);
			}

			boll_width * (diff_sum / FixedI128::from(total_len as i128)).sqrt()
		}

		fn calculate_bollinger_bands(
			prices: &[FixedI128],
			mean_prices: &[FixedI128],
			window: usize,
			boll_width: FixedI128,
		) -> (Vec<FixedI128>, Vec<FixedI128>) {
			// Initialize the result vector with the size of prices vector
			let total_len = prices.len();
			let mut upper_band = Vec::<FixedI128>::with_capacity(total_len);
			let mut lower_band = Vec::<FixedI128>::with_capacity(total_len);

			// Handle Edge case
			if total_len == 0 || window == 0 {
				return (upper_band, lower_band);
			}

			for iterator in 0..total_len {
				// Calculate the sliding mean till the iterator
				if iterator < window {
					// Calculate the standard deviation factor
					let std = Self::calculate_std(
						&prices[0..iterator + 1],
						mean_prices[iterator],
						boll_width,
					);

					// Add to lower and upper band vectors
					lower_band[iterator] = mean_prices[iterator] - std;
					upper_band[iterator] = mean_prices[iterator] + std;
				} else {
					// Calculate the standard deviation factor
					let std = Self::calculate_std(
						&prices[iterator - window + 1..iterator + 1],
						mean_prices[iterator],
						boll_width,
					);

					// Add to lower and upper band vectors
					lower_band[iterator] = mean_prices[iterator] - std;
					upper_band[iterator] = mean_prices[iterator] + std;
				}
			}

			(lower_band, upper_band)
		}

		fn calculate_sliding_mean(prices: &[FixedI128], window: usize) -> Vec<FixedI128> {
			// Initialize the result vector with the size of prices vector
			let total_len = prices.len();
			let mut result = Vec::<FixedI128>::with_capacity(total_len);

			// Handle Edge case
			if total_len == 0 || window == 0 {
				return result;
			}

			// Initialize window_sum and convert window size to FixedI128
			let mut window_sum = FixedI128::zero();
			let window_fixed = FixedI128::from(window as i128);

			for iterator in 0..total_len {
				// Calculate the sliding mean till the iterator
				if iterator < window {
					// Add to window_sum;
					// since it's below window_size we don't need to remove the first price in the
					// sum
					window_sum = window_sum + prices[iterator];

					// Add to result array
					result[iterator] = window_sum / FixedI128::from((iterator + 1) as i128);
				} else {
					// Add the current price and remove the first price in the sum
					window_sum = window_sum - prices[iterator - window] + prices[iterator];

					// Add to result array
					result[iterator] = window_sum / window_fixed;
				}
			}

			result
		}

		fn calculate_abr(
			mark_prices: Vec<FixedI128>,
			index_prices: Vec<FixedI128>,
			base_abr_rate: FixedI128,
			boll_width: FixedI128,
			window: usize,
		) -> FixedI128 {
			let mean_prices = Self::calculate_sliding_mean(&mark_prices, window);
			let (upper_band, lower_band) =
				Self::calculate_bollinger_bands(&mark_prices, &mean_prices, window, boll_width);
			let mut premiums = Self::calculate_premium(&mark_prices, &index_prices);
			let premiums_w_jumps = Self::calculate_jump(
				&mut premiums,
				&upper_band,
				&lower_band,
				&mark_prices,
				&index_prices,
			);

			Self::calculate_effective_abr(&premiums_w_jumps) + base_abr_rate
		}
	}
}
