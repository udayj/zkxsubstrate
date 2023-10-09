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
	use sp_arithmetic::traits::Bounded;
	use sp_arithmetic::traits::Zero;
	use sp_arithmetic::FixedI128;
	use sp_io::hashing::blake2_256;
	use zkx_support::helpers::sig_u256_to_sig_felt;
	use zkx_support::traits::{
		AssetInterface, Hashable, MarketInterface, MarketPricesInterface, TradingAccountInterface,
		TradingInterface, U256Ext,
	};
	use zkx_support::types::{
		BalanceChangeReason, BalanceUpdate, Direction, FundModifyType, Position,
		PositionDetailsForRiskManagement, TradingAccount, TradingAccountMinimal, WithdrawalRequest,
	};
	use zkx_support::{ecdsa_verify, Signature};

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type AssetPallet: AssetInterface;
		type TradingPallet: TradingInterface;
		type MarketPallet: MarketInterface;
		type MarketPricesPallet: MarketPricesInterface;
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
	#[pallet::getter(fn balances)]
	// Here, key1 is account_id,  key2 is asset_id and value is the balance
	pub(super) type BalancesMap<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, U256, Blake2_128Concat, u128, FixedI128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn locked_margin)]
	// Here, key1 is account_id,  key2 is asset_id and value is the locked margin
	pub(super) type LockedMarginMap<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, U256, Blake2_128Concat, u128, FixedI128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn account_collaterals)]
	// Here, key1 is account_id and value is vector of collateral_ids
	pub(super) type AccountCollateralsMap<T: Config> =
		StorageMap<_, Blake2_128Concat, U256, Vec<u128>, ValueQuery>;

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Account already exists
		DuplicateAccount,
		/// Account does not exist
		AccountDoesNotExist,
		/// Asset not created
		AssetNotFound,
		/// Asset provided as collateral is not marked as collateral in the system
		AssetNotCollateral,
		/// Withdrawal amount is less than available balance
		InsufficientBalance,
		/// Invalid withdrawal request hash - withdrawal request could not be hashed into a Field Element
		InvalidWithdrawalRequestHash,
		/// Invalid Signature Field Elements - sig_r and/or sig_s could not be converted into a Signature
		InvalidSignatureFelt,
		/// ECDSA Signature could not be verified
		InvalidSignature,
		/// Public Key not found for account id
		NoPublicKeyFound,
		/// Invalid public key - publickey u256 could not be converted to Field Element
		InvalidPublicKey,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Several accounts added
		AccountsAdded { length: u128 },
		/// Balances for an account updated
		BalanceUpdated {
			account_id: U256,
			account: TradingAccountMinimal,
			collateral_id: u128,
			amount: FixedI128,
			modify_type: u8,
			reason: u8,
			previous_balance: FixedI128,
			new_balance: FixedI128,
			block_number: T::BlockNumber,
		},
		/// Event to be synced by L2, for pnl changes
		UserBalanceChange {
			trading_account: TradingAccountMinimal,
			collateral_id: u128,
			amount: FixedI128,
			modify_type: FundModifyType,
			reason: u8,
			block_number: T::BlockNumber,
		},
		/// Event to be synced by L2, for withdrawal requests
		UserWithdrawal {
			trading_account: TradingAccountMinimal,
			collateral_id: u128,
			amount: FixedI128,
			block_number: T::BlockNumber,
		},
		/// Account created
		AccountCreated { account_id: U256, account_address: U256, index: u8 },
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Add several accounts together
		#[pallet::weight(0)]
		pub fn add_accounts(
			origin: OriginFor<T>,
			accounts: Vec<TradingAccountMinimal>,
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

				// Check if the account already exists
				ensure!(!AccountMap::<T>::contains_key(account_id), Error::<T>::DuplicateAccount);
				let trading_account: TradingAccount = TradingAccount {
					account_id,
					account_address: element.account_address,
					index: element.index,
					pub_key: element.pub_key,
				};

				AccountMap::<T>::insert(account_id, trading_account);
				current_length += 1;
				Self::deposit_event(Event::AccountCreated {
					account_id,
					account_address: element.account_address,
					index: element.index,
				});

				// Add predefined balance for default collateral to the account
				let default_collateral = T::AssetPallet::get_default_collateral();
				BalancesMap::<T>::set(account_id, default_collateral, 10000.into());
				let mut collaterals: Vec<u128> = Vec::new();
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

			// Check if the account already exists
			ensure!(AccountMap::<T>::contains_key(account_id), Error::<T>::AccountDoesNotExist);

			for element in balances {
				// Validate that the asset exists and it is a collateral
				let asset_collateral = T::AssetPallet::get_asset(element.asset_id);
				ensure!(asset_collateral.is_some(), Error::<T>::AssetNotFound);
				ensure!(asset_collateral.unwrap().is_collateral, Error::<T>::AssetNotCollateral);

				let current_balance: FixedI128 =
					BalancesMap::<T>::get(account_id, element.asset_id);
				if current_balance == 0.into() {
					Self::add_collateral(account_id, element.asset_id);
				}
				// Update the map with new balance
				BalancesMap::<T>::set(account_id, element.asset_id, element.balance_value);

				let account =
					AccountMap::<T>::get(&account_id).unwrap().to_trading_account_minimal();
				let block_number = <frame_system::Pallet<T>>::block_number();

				Self::deposit_event(Event::BalanceUpdated {
					account_id,
					account,
					collateral_id: element.asset_id,
					amount: element.balance_value,
					modify_type: FundModifyType::Increase.into(),
					reason: BalanceChangeReason::Deposit.into(),
					previous_balance: current_balance,
					new_balance: element.balance_value,
					block_number,
				});
			}

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn withdraw(
			origin: OriginFor<T>,
			withdrawal_request: WithdrawalRequest,
		) -> DispatchResult {
			ensure_signed(origin)?;

			// Check if the account already exists
			ensure!(
				AccountMap::<T>::contains_key(withdrawal_request.account_id),
				Error::<T>::AccountDoesNotExist
			);

			let _ = Self::verify_signature(&withdrawal_request);

			// Get the current balance
			let current_balance: FixedI128 = BalancesMap::<T>::get(
				withdrawal_request.account_id,
				withdrawal_request.collateral_id,
			);

			ensure!(withdrawal_request.amount <= current_balance, Error::<T>::InsufficientBalance);

			// Update the balance
			BalancesMap::<T>::set(
				withdrawal_request.account_id,
				withdrawal_request.collateral_id,
				current_balance - withdrawal_request.amount,
			);

			let account = AccountMap::<T>::get(&withdrawal_request.account_id)
				.unwrap()
				.to_trading_account_minimal();
			let block_number = <frame_system::Pallet<T>>::block_number();

			// BalanceUpdated event is emitted
			Self::deposit_event(Event::BalanceUpdated {
				account_id: withdrawal_request.account_id,
				account,
				collateral_id: withdrawal_request.collateral_id,
				amount: withdrawal_request.amount,
				modify_type: FundModifyType::Decrease.into(),
				reason: BalanceChangeReason::Withdrawal.into(),
				previous_balance: current_balance,
				new_balance: current_balance - withdrawal_request.amount,
				block_number,
			});

			// Get the trading account
			let account = Self::get_account(&withdrawal_request.account_id)
				.unwrap()
				.to_trading_account_minimal();

			// Get the block number
			let block_number = <frame_system::Pallet<T>>::block_number();

			Self::deposit_event(Event::UserWithdrawal {
				trading_account: account,
				collateral_id: withdrawal_request.collateral_id,
				amount: withdrawal_request.amount,
				block_number,
			});

			Ok(())
		}
	}

	// Pallet internal functions
	impl<T: Config> Pallet<T> {
		fn add_collateral(account_id: U256, collateral_id: u128) {
			let mut collaterals = AccountCollateralsMap::<T>::get(account_id);
			for element in &collaterals {
				if element == &collateral_id {
					return;
				}
			}

			collaterals.push(collateral_id);
			AccountCollateralsMap::<T>::insert(account_id, collaterals);
		}

		fn get_risk_parameters_position(
			position: &Position,
			direction: Direction,
			market_price: FixedI128,
			market_id: u128,
		) -> (FixedI128, FixedI128, FixedI128) {
			let market = T::MarketPallet::get_market(market_id).unwrap();
			let req_margin = market.maintenance_margin_fraction;

			// Calculate the maintenance requirement
			let maintenance_position = position.avg_execution_price * position.size;
			let maintenance_requirement = req_margin * maintenance_position;

			if market_price == 0.into() {
				return (0.into(), maintenance_requirement, 0.into());
			}

			// Calculate pnl to check if it is the least collateralized position
			let price_diff: FixedI128;
			if direction == Direction::Long {
				price_diff = market_price - position.avg_execution_price;
			} else {
				price_diff = position.avg_execution_price - market_price;
			}

			let pnl = price_diff * position.size;

			// Margin ratio calculation
			let numerator = position.margin_amount + pnl;
			let denominator = position.size * market_price;
			let collateral_ratio_position = numerator / denominator;

			return (pnl, maintenance_requirement, collateral_ratio_position);
		}

		fn calculate_margin_info(
			account_id: U256,
			new_position_maintanence_requirement: FixedI128,
			markets: Vec<u128>,
		) -> (FixedI128, FixedI128, FixedI128, PositionDetailsForRiskManagement, FixedI128) {
			let mut unrealized_pnl_sum: FixedI128 = 0.into();
			let mut maintenance_margin_requirement: FixedI128 =
				new_position_maintanence_requirement;
			let mut least_collateral_ratio: FixedI128 = FixedI128::max_value();
			let mut least_collateral_ratio_position: PositionDetailsForRiskManagement =
				PositionDetailsForRiskManagement {
					market_id: 0,
					direction: Direction::Long,
					avg_execution_price: 0.into(),
					size: 0.into(),
					margin_amount: 0.into(),
					borrowed_amount: 0.into(),
					leverage: 0.into(),
				};
			let mut least_collateral_ratio_position_asset_price: FixedI128 = 0.into();
			for curr_market_id in markets {
				// Get Long position
				let long_position: Position =
					T::TradingPallet::get_position(account_id, curr_market_id, Direction::Long);

				// Get Short position
				let short_position: Position =
					T::TradingPallet::get_position(account_id, curr_market_id, Direction::Short);

				// Get Market price
				let market_price = T::MarketPricesPallet::get_market_price(curr_market_id);

				if market_price == 0.into() {
					return (
						0.into(),
						0.into(),
						1.into(),
						PositionDetailsForRiskManagement {
							market_id: 0,
							direction: Direction::Long,
							avg_execution_price: 0.into(),
							size: 0.into(),
							margin_amount: 0.into(),
							borrowed_amount: 0.into(),
							leverage: 0.into(),
						},
						0.into(),
					);
				}

				let long_maintanence_requirement;
				let long_pnl;
				let long_collateral_ratio;

				if long_position.size == 0.into() {
					long_collateral_ratio = FixedI128::max_value();
					long_maintanence_requirement = 0.into();
					long_pnl = 0.into();
				} else {
					// Get risk parameters of the position
					(long_pnl, long_maintanence_requirement, long_collateral_ratio) =
						Self::get_risk_parameters_position(
							&long_position,
							Direction::Long,
							market_price,
							curr_market_id,
						);
				}

				let short_maintanence_requirement;
				let short_pnl;
				let short_collateral_ratio;

				if short_position.size == 0.into() {
					short_collateral_ratio = FixedI128::max_value();
					short_maintanence_requirement = 0.into();
					short_pnl = 0.into();
				} else {
					// Get risk parameters of the position
					(short_pnl, short_maintanence_requirement, short_collateral_ratio) =
						Self::get_risk_parameters_position(
							&short_position,
							Direction::Short,
							market_price,
							curr_market_id,
						);
				}

				let curr_long_position: PositionDetailsForRiskManagement =
					PositionDetailsForRiskManagement {
						market_id: curr_market_id,
						direction: Direction::Long,
						avg_execution_price: long_position.avg_execution_price,
						size: long_position.size,
						margin_amount: long_position.margin_amount,
						borrowed_amount: long_position.borrowed_amount,
						leverage: long_position.leverage,
					};

				let curr_short_position: PositionDetailsForRiskManagement =
					PositionDetailsForRiskManagement {
						market_id: curr_market_id,
						direction: Direction::Short,
						avg_execution_price: short_position.avg_execution_price,
						size: short_position.size,
						margin_amount: short_position.margin_amount,
						borrowed_amount: short_position.borrowed_amount,
						leverage: short_position.leverage,
					};

				let new_least_collateral_ratio_position: PositionDetailsForRiskManagement;
				let new_least_collateral_ratio_position_asset_price;
				let mut new_least_collateral_ratio =
					FixedI128::min(least_collateral_ratio, short_collateral_ratio);
				new_least_collateral_ratio =
					FixedI128::min(new_least_collateral_ratio, long_collateral_ratio);

				if new_least_collateral_ratio == least_collateral_ratio {
					new_least_collateral_ratio_position = least_collateral_ratio_position;
					new_least_collateral_ratio_position_asset_price =
						least_collateral_ratio_position_asset_price;
				} else if new_least_collateral_ratio == short_collateral_ratio {
					new_least_collateral_ratio_position_asset_price = market_price;
					new_least_collateral_ratio_position = curr_short_position;
				} else {
					new_least_collateral_ratio_position_asset_price = market_price;
					new_least_collateral_ratio_position = curr_long_position;
				}

				unrealized_pnl_sum = unrealized_pnl_sum + short_pnl + long_pnl;

				maintenance_margin_requirement = maintenance_margin_requirement
					+ short_maintanence_requirement
					+ long_maintanence_requirement;

				least_collateral_ratio = new_least_collateral_ratio;
				least_collateral_ratio_position = new_least_collateral_ratio_position;
				least_collateral_ratio_position_asset_price =
					new_least_collateral_ratio_position_asset_price;
			}
			return (
				unrealized_pnl_sum,
				maintenance_margin_requirement,
				least_collateral_ratio,
				least_collateral_ratio_position,
				least_collateral_ratio_position_asset_price,
			);
		}

		fn verify_signature(withdrawal_request: &WithdrawalRequest) -> Result<(), Error<T>> {
			// Signature validation
			let sig_felt =
				sig_u256_to_sig_felt(&withdrawal_request.sig_r, &withdrawal_request.sig_s);

			// Sig_r and/or Sig_s could not be converted to FieldElement
			ensure!(sig_felt.is_ok(), Error::<T>::InvalidSignatureFelt);

			let (sig_r_felt, sig_s_felt) = sig_felt.unwrap();
			let sig = Signature { r: sig_r_felt, s: sig_s_felt };

			let withdrawal_request_hash = withdrawal_request.hash(&withdrawal_request.hash_type);

			// withdrawal_request could not be hashed
			ensure!(withdrawal_request_hash.is_ok(), Error::<T>::InvalidWithdrawalRequestHash);

			let public_key = Self::get_public_key(&withdrawal_request.account_id);

			// Public key not found for this account_id
			ensure!(public_key.is_some(), Error::<T>::NoPublicKeyFound);

			let public_key_felt = public_key.unwrap().try_to_felt();

			// Public Key U256 could not be converted to FieldElement
			ensure!(public_key_felt.is_ok(), Error::<T>::InvalidPublicKey);

			let verification =
				ecdsa_verify(&public_key_felt.unwrap(), &withdrawal_request_hash.unwrap(), &sig);

			// Signature verification returned error or false
			ensure!(verification.is_ok() && verification.unwrap(), Error::<T>::InvalidSignature);

			Ok(())
		}
	}

	impl<T: Config> TradingAccountInterface for Pallet<T> {
		fn get_balance(account_id: U256, collateral_id: u128) -> FixedI128 {
			BalancesMap::<T>::get(account_id, collateral_id)
		}

		fn get_unused_balance(account_id: U256, collateral_id: u128) -> FixedI128 {
			let total_balance = BalancesMap::<T>::get(account_id, collateral_id);
			let locked_balance = LockedMarginMap::<T>::get(account_id, collateral_id);
			total_balance - locked_balance
		}

		fn get_locked_margin(account_id: U256, collateral_id: u128) -> FixedI128 {
			LockedMarginMap::<T>::get(account_id, collateral_id)
		}

		fn set_locked_margin(account_id: U256, collateral_id: u128, new_amount: FixedI128) {
			LockedMarginMap::<T>::set(account_id, collateral_id, new_amount);
		}

		fn transfer(
			account_id: U256,
			collateral_id: u128,
			amount: FixedI128,
			reason: BalanceChangeReason,
		) {
			let account = AccountMap::<T>::get(&account_id).unwrap().to_trading_account_minimal();
			let current_balance = BalancesMap::<T>::get(&account_id, collateral_id);
			let new_balance = current_balance.add(amount);
			let block_number = <frame_system::Pallet<T>>::block_number();
			BalancesMap::<T>::set(account_id, collateral_id, new_balance);

			Self::deposit_event(Event::BalanceUpdated {
				account_id,
				account,
				collateral_id,
				amount,
				modify_type: FundModifyType::Increase.into(),
				reason: reason.into(),
				previous_balance: current_balance,
				new_balance,
				block_number,
			});

			// Event to be synced by L2
			Self::deposit_event(Event::UserBalanceChange {
				trading_account: account,
				collateral_id,
				amount,
				modify_type: FundModifyType::Increase,
				reason: reason.into(),
				block_number,
			});
		}

		fn transfer_from(
			account_id: U256,
			collateral_id: u128,
			amount: FixedI128,
			reason: BalanceChangeReason,
		) {
			let account = AccountMap::<T>::get(&account_id).unwrap().to_trading_account_minimal();
			let current_balance = BalancesMap::<T>::get(&account_id, collateral_id);
			let new_balance = current_balance.sub(amount);
			let block_number = <frame_system::Pallet<T>>::block_number();
			BalancesMap::<T>::set(account_id, collateral_id, new_balance);

			Self::deposit_event(Event::BalanceUpdated {
				account_id,
				account,
				collateral_id,
				amount,
				modify_type: FundModifyType::Decrease.into(),
				reason: reason.into(),
				previous_balance: current_balance,
				new_balance,
				block_number,
			});

			// Event to be synced by L2
			Self::deposit_event(Event::UserBalanceChange {
				trading_account: account,
				collateral_id,
				amount,
				modify_type: FundModifyType::Decrease,
				reason: reason.into(),
				block_number,
			});
		}

		fn is_registered_user(account_id: U256) -> bool {
			AccountMap::<T>::contains_key(&account_id)
		}

		fn get_account(account_id: &U256) -> Option<TradingAccount> {
			let trading_account = AccountMap::<T>::get(&account_id)?;
			Some(trading_account)
		}

		fn get_public_key(account_id: &U256) -> Option<U256> {
			let trading_account = AccountMap::<T>::get(&account_id)?;
			Some(trading_account.pub_key)
		}

		fn get_margin_info(
			account_id: U256,
			collateral_id: u128,
			new_position_maintanence_requirement: FixedI128,
			new_position_margin: FixedI128,
		) -> (
			bool,
			FixedI128,
			FixedI128,
			FixedI128,
			FixedI128,
			FixedI128,
			PositionDetailsForRiskManagement,
			FixedI128,
		) {
			// Get markets corresponding of the collateral
			let markets: Vec<u128> =
				T::TradingPallet::get_markets_of_collateral(account_id, collateral_id);

			// Get balance for the given collateral
			let collateral_balance = BalancesMap::<T>::get(account_id, collateral_id);

			// Get the sum of initial margin of all positions under the given collateral
			let initial_margin_sum = LockedMarginMap::<T>::get(account_id, collateral_id);

			if markets.len() == 0 {
				let available_margin = collateral_balance + new_position_margin;
				return (
					false,              // is_liquidation
					collateral_balance, // total_margin
					available_margin,   // available_margin
					0.into(),           // unrealized_pnl_sum
					0.into(),           // maintenance_margin_requirement
					0.into(),           // least_collateral_ratio
					PositionDetailsForRiskManagement {
						market_id: 0,
						direction: Direction::Long,
						avg_execution_price: 0.into(),
						size: 0.into(),
						margin_amount: 0.into(),
						borrowed_amount: 0.into(),
						leverage: 0.into(),
					}, // least_collateral_ratio_position
					0.into(),           // least_collateral_ratio_position_asset_price
				);
			}

			let (
				unrealized_pnl_sum,
				maintenance_margin_requirement,
				least_collateral_ratio,
				least_collateral_ratio_position,
				least_collateral_ratio_position_asset_price,
			) = Self::calculate_margin_info(account_id, new_position_maintanence_requirement, markets);

			// If any of the position's ttl is outdated
			if least_collateral_ratio_position_asset_price == 0.into() {
				let available_margin_temp = collateral_balance - new_position_margin;
				let available_margin = available_margin_temp - initial_margin_sum;
				return (
					false,              // is_liquidation
					collateral_balance, // total_margin
					available_margin,   // available_margin
					0.into(),           // unrealized_pnl_sum
					0.into(),           // maintenance_margin_requirement
					0.into(),           // least_collateral_ratio
					PositionDetailsForRiskManagement {
						market_id: 0,
						direction: Direction::Long,
						avg_execution_price: 0.into(),
						size: 0.into(),
						margin_amount: 0.into(),
						borrowed_amount: 0.into(),
						leverage: 0.into(),
					}, // least_collateral_ratio_position
					0.into(),           // least_collateral_ratio_position_asset_price
				);
			}

			// Add the new position's margin
			let total_initial_margin_sum = initial_margin_sum + new_position_margin;

			// Compute total margin of the given collateral
			let total_margin = collateral_balance + unrealized_pnl_sum;

			// Compute available margin of the given collateral
			let available_margin = total_margin - total_initial_margin_sum;

			let mut is_liquidation = false;

			// If it's a long position with 1x leverage, ignore it
			if total_margin <= maintenance_margin_requirement {
				if !((least_collateral_ratio_position.direction == Direction::Long)
					&& (least_collateral_ratio_position.leverage == 1.into()))
				{
					is_liquidation = true;
				}
			}

			return (
				is_liquidation,
				total_margin,
				available_margin,
				unrealized_pnl_sum,
				maintenance_margin_requirement,
				least_collateral_ratio,
				least_collateral_ratio_position,
				least_collateral_ratio_position_asset_price,
			);
		}

		fn deposit(trading_account: TradingAccountMinimal, collateral_id: u128, amount: FixedI128) {
			let account_address = trading_account.account_address;
			let index = trading_account.index;
			let pub_key = trading_account.pub_key;

			// Create trading account id
			let mut account_array: [u8; 32] = [0; 32];
			account_address.to_little_endian(&mut account_array);

			let mut concatenated_bytes: Vec<u8> = account_array.to_vec();
			concatenated_bytes.push(index);
			let result: [u8; 33] = concatenated_bytes.try_into().unwrap();

			let account_id = blake2_256(&result).into();

			// Check if the account already exists, if it doesn't exist then create an account
			if !AccountMap::<T>::contains_key(&account_id) {
				let trading_account: TradingAccount =
					TradingAccount { account_id, account_address, index, pub_key };
				AccountMap::<T>::insert(account_id, trading_account);
				Self::deposit_event(Event::AccountCreated { account_id, account_address, index });
			}

			// Get the current balance
			let current_balance = BalancesMap::<T>::get(account_id, collateral_id);

			// If the current balance is 0, then add collateral to the AccountCollateralsMap
			if current_balance == FixedI128::zero() {
				Self::add_collateral(account_id, collateral_id);
			}

			let new_balance: FixedI128 = amount + current_balance;
			// Update the balance
			BalancesMap::<T>::set(account_id, collateral_id, new_balance);

			// Get the user account
			let account = AccountMap::<T>::get(&account_id).unwrap().to_trading_account_minimal();
			let block_number = <frame_system::Pallet<T>>::block_number();

			// BalanceUpdated event is emitted
			Self::deposit_event(Event::BalanceUpdated {
				account_id,
				account,
				collateral_id,
				amount,
				modify_type: FundModifyType::Increase.into(),
				reason: BalanceChangeReason::Deposit.into(),
				previous_balance: current_balance,
				new_balance,
				block_number,
			});
		}
	}
}
