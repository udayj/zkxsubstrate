#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod weights;
pub use weights::*;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use super::*;
	use frame_support::inherent::Vec;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use primitive_types::U256;
	use sp_arithmetic::fixed_point::FixedI128;
	use sp_io::hashing::blake2_256;
	use zkx_support::traits::{AssetInterface, TradingAccountInterface};
	use zkx_support::types::{BalanceUpdate, TradingAccount, TradingAccountWithoutId};

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
	// It stores no.of accounts
	pub(super) type AccountsCount<T: Config> = StorageValue<_, u128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn accounts)]
	// Here, key is the trading_account_id and value is the trading account
	pub(super) type AccountMap<T: Config> =
		StorageMap<_, Blake2_128Concat, U256, TradingAccount, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn account_presence)]
	// Here, key is the account_id and value is the true/false
	pub(super) type AccountPresenceMap<T: Config> =
		StorageMap<_, Blake2_128Concat, U256, bool, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn balances)]
	// Here, key1 is account_id,  key2 is asset_id and value is the balance
	pub(super) type BalancesMap<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, U256, Blake2_128Concat, U256, FixedI128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn locked_margin)]
	// Here, key1 is account_id,  key2 is asset_id and value is the locked margin
	pub(super) type LockedMarginMap<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, U256, Blake2_128Concat, U256, FixedI128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn account_collaterals)]
	// Here, key1 is account_id and value is vector of collateral_ids
	pub(super) type AccountCollateralsMap<T: Config> =
		StorageMap<_, Blake2_128Concat, U256, Vec<U256>, ValueQuery>;

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Account already exists
		DuplicateAccount,
		/// Asset not created
		AssetNotFound,
		/// Asset provided as collateral is not marked as collateral in the system
		AssetNotCollateral,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Several accounts added
		AccountsAdded { length: u128 },
		/// Balances for an account updated
		BalancesUpdated { account_id: U256 },
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Add several accounts together
		#[pallet::weight(0)]
		pub fn add_accounts(
			origin: OriginFor<T>,
			accounts: Vec<TradingAccountWithoutId>,
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;

			let length: u128 = u128::try_from(accounts.len()).unwrap();
			let mut current_length = AccountsCount::<T>::get();
			let final_length: u128 = length + current_length;
			let mut account_id: U256;

			for element in accounts {
				let account_address = U256::from(element.account_address);
				let mut account_array: [u8; 32] = [0; 32];
				account_address.to_little_endian(&mut account_array);

				let mut concatenated_bytes: Vec<u8> = account_array.to_vec();
				concatenated_bytes.push(element.index);
				let result: [u8; 33] = concatenated_bytes.try_into().unwrap();

				account_id = blake2_256(&result).into();

				// Check if the account exists in the presence storage map
				ensure!(
					!AccountPresenceMap::<T>::contains_key(account_id),
					Error::<T>::DuplicateAccount
				);
				AccountPresenceMap::<T>::insert(account_id, true);
				let trading_account: TradingAccount = TradingAccount {
					account_id,
					account_address: element.account_address,
					index: element.index,
					pub_key: element.pub_key,
				};

				AccountMap::<T>::insert(account_id, trading_account);
				current_length += 1;

				// Add predefined balance for default collateral to the account
				let default_collateral = T::Asset::get_default_collateral();
				BalancesMap::<T>::set(account_id, default_collateral, 10000.into());
				let mut collaterals: Vec<U256> = Vec::new();
				collaterals.push(default_collateral);
				AccountCollateralsMap::<T>::insert(account_id, collaterals);
			}

			AccountsCount::<T>::put(final_length);

			Self::deposit_event(Event::AccountsAdded { length });

			Ok(())
		}

		/// Add balances for a particular user
		#[pallet::weight(0)]
		pub fn set_balances(
			origin: OriginFor<T>,
			account_id: U256,
			balances: Vec<BalanceUpdate>,
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;

			// Check if the account exists in the presence storage map
			ensure!(
				AccountPresenceMap::<T>::contains_key(account_id),
				Error::<T>::DuplicateAccount
			);

			for element in balances {
				// Validate that the asset exists and it is a collateral
				let asset_collateral = T::Asset::get_asset(element.asset_id);
				ensure!(asset_collateral.is_some(), Error::<T>::AssetNotFound);
				ensure!(asset_collateral.unwrap().is_collateral, Error::<T>::AssetNotCollateral);

				let current_balance: FixedI128 =
					BalancesMap::<T>::get(account_id, element.asset_id);
				if current_balance == 0.into() {
					Self::add_collateral(account_id, element.asset_id);
				}
				// Update the map with new balance
				BalancesMap::<T>::set(account_id, element.asset_id, element.balance_value);
			}

			Self::deposit_event(Event::BalancesUpdated { account_id });

			Ok(())
		}
	}

	impl<T: Config> TradingAccountInterface for Pallet<T> {
		fn get_balance(account: U256, asset_id: U256) -> FixedI128 {
			BalancesMap::<T>::get(account, asset_id)
		}

		fn get_locked_margin(account: U256, asset_id: U256) -> FixedI128 {
			LockedMarginMap::<T>::get(account, asset_id)
		}

		fn set_locked_margin(account: U256, asset_id: U256, new_amount: FixedI128) {
			LockedMarginMap::<T>::set(account, asset_id, new_amount);
		}

		fn transfer(account: U256, asset_id: U256, amount: FixedI128) {
			let current_balance = BalancesMap::<T>::get(&account, asset_id);
			let new_balance = current_balance.add(amount);
			BalancesMap::<T>::set(account, asset_id, new_balance);
		}

		fn transfer_from(account: U256, asset_id: U256, amount: FixedI128) {
			let current_balance = BalancesMap::<T>::get(&account, asset_id);
			let new_balance = current_balance.sub(amount);
			BalancesMap::<T>::set(account, asset_id, new_balance);
		}

		fn is_registered_user(account: U256) -> bool {
			AccountPresenceMap::<T>::contains_key(&account)
		}
	}

	// Pallet internal functions
	impl<T: Config> Pallet<T> {
		fn add_collateral(account_id: U256, collateral_id: U256) {
			let mut collaterals = AccountCollateralsMap::<T>::get(account_id);
			for element in &collaterals {
				if element == &collateral_id {
					return;
				}
			}

			collaterals.push(collateral_id);
			AccountCollateralsMap::<T>::insert(account_id, collaterals);
		}
	}
}
