#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	// use frame_support::pallet_prelude::*;
	// use frame_system::pallet_prelude::*;
	use pallet_support::traits::TradingAccountInterface;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type TradingAccountPallet: TradingAccountInterface;
	}
}
