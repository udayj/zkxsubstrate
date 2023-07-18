#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use sp_arithmetic::fixed_point::FixedI128;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod weights;
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use zkx_support::traits::AssetInterface;
	use zkx_support::types::TradingAccount;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type WeightInfo: WeightInfo;
		type Asset: AssetInterface;
	}

	#[pallet::storage]
	#[pallet::getter(fn accounts_count)]
	pub(super) type AccountsCount<T: Config> = StorageValue<_, u128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn accounts)]
	pub type Accounts<T: Config> = StorageMap<_, Blake2_128Concat, u8, TradingAccount, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn balances)]
	pub(super) type Balances<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		[u8; 32],
		Blake2_128Concat,
		u8,
		FixedI128,
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		AccountAdded { account_id: [u8; 32] },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::do_something())]
		pub fn record_account(
			origin: OriginFor<T>,
			trading_account: TradingAccount,
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// <Accounts<T>>::insert(trading_account.account_id, trading_account.clone());
			let default_collateral = T::Asset::get_default_collateral();

			Self::deposit_event(Event::AccountAdded { account_id: trading_account.account_id });

			Ok(())
		}
	}
}
