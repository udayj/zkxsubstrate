#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use pallet_support::traits::TradingAccountInterface;
	use sp_arithmetic::{fixed_point::FixedI128, FixedI128};

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type TradingAccountPallet: TradingAccountInterface;
	}

	impl<T: Config> Pallet<T> {

		fn calculate_sliding_mean(prices: Vec<FixedI128>) -> Vec<FixedI128> {
			let sliding_mean = Vec<FixedI128>;
		}

		fn calculate_abr(mark_prices: Vec<FixedI128>, index_prices: Vec<FixedI128>) -> () {
			
		}
	}
}
