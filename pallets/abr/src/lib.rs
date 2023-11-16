#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use pallet_support::traits::TradingAccountInterface;
	use sp_arithmetic::{fixed_point::FixedI128, traits::Zero, FixedI128};

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type TradingAccountPallet: TradingAccountInterface;
	}

	impl<T: Config> Pallet<T> {
		fn calculate_bollinger_bands(
			prices: Vec<FixedI128>,
			mean_prices: Vec<FixedI128>,
			window: usize,
			boll_width: u8,
		) -> (Vec<FixedI128>, Vec<FixedI128>) {
			// Initialize the result vector with the size of prices vector
			let total_len = prices.len();
			let mut upper_band = Vec::<FixedI128>::with_capacity(total_len);
			let mut lower_band = Vec::<FixedI128>::with_capacity(total_len);

			// Handle Edge case
			if total_len == 0 || window == 0 {
				return (upper_band, lower_band);
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

			(upper_band, lower_band)
		}

		fn calculate_sliding_mean(prices: Vec<FixedI128>, window: usize) -> Vec<FixedI128> {
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

		fn calculate_abr(mark_prices: Vec<FixedI128>, index_prices: Vec<FixedI128>) -> () {}
	}
}
