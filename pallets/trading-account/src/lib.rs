#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use super::*;
	use frame_support::dispatch::Vec;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use primitive_types::U256;
	use sp_arithmetic::traits::Bounded;
	use sp_arithmetic::traits::Zero;
	use sp_arithmetic::FixedI128;
	use sp_io::hashing::blake2_256;
	use zkx_support::helpers::sig_u256_to_sig_felt;
	use zkx_support::traits::{
		AssetInterface, FieldElementExt, Hashable, MarketInterface, PricesInterface,
		TradingAccountInterface, TradingInterface, U256Ext,
	};
	use zkx_support::types::{
		BalanceChangeReason, BalanceUpdate, Direction, ForceClosureFlag, FundModifyType, Position,
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
		type PricesPallet: PricesInterface;
	}

	#[pallet::storage]
	#[pallet::getter(fn accounts_count)]
	// It stores no.of accounts
	pub(super) type AccountsCount<T: Config> = StorageValue<_, u128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn standard_withdrawal_fee)]
	// It stores the standard withdrawal fee
	pub(super) type StandardWithdrawalFee<T: Config> = StorageValue<_, FixedI128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn accounts)]
	// Here, key is the trading_account_id and value is the trading account
	pub(super) type AccountMap<T: Config> =
		StorageMap<_, Blake2_128Concat, U256, TradingAccount, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn withdrawal_status)]
	// Here, key is the hash of the withdrawal request and value is boolean value
	pub(super) type IsWithdrawalProcessed<T: Config> =
		StorageMap<_, Blake2_128Concat, U256, bool, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn accounts_list)]
	// Here, key is the index and value is the account_id
	pub(super) type AccountsListMap<T: Config> =
		StorageMap<_, Blake2_128Concat, u128, U256, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn balances)]
	// Here, key1 is account_id,  key2 is asset_id and value is the balance
	pub(super) type BalancesMap<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, U256, Blake2_128Concat, u128, FixedI128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn deferred_deposits)]
	// Here, key1 is account_id and value is the array of type DeferredBalances
	pub(super) type DeferredBalancesMap<T: Config> =
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
		/// Duplicate Withdrawal Request
		DuplicateWithdrawal,
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
		/// Invalid standard withdrawal fee
		InvalidWithdrawalFee,
		/// Invalid arguments in the withdrawal request
		InvalidWithdrawalRequest,
		/// Deposit and Withdrawal are not allowed if deleveraging or liquidation is in progress
		ForceClosureFlagSet,
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
			block_number: BlockNumberFor<T>,
		},
		/// Event emitted for deferred deposits
		DeferredBalance { account_id: U256, collateral_id: u128, amount: FixedI128 },
		/// Event to be synced by L2, for pnl changes
		UserBalanceChange {
			trading_account: TradingAccountMinimal,
			collateral_id: u128,
			amount: FixedI128,
			modify_type: FundModifyType,
			reason: u8,
			block_number: BlockNumberFor<T>,
		},
		/// Event to be synced by L2, for withdrawal requests
		UserWithdrawal {
			trading_account: TradingAccountMinimal,
			collateral_id: u128,
			amount: FixedI128,
			block_number: BlockNumberFor<T>,
		},
		/// Account created
		AccountCreated { account_id: U256, account_address: U256, index: u8 },
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// TODO(merkle-groot): To be removed in production
		/// To test depositing funds
		#[pallet::weight(0)]
		pub fn deposit(
			origin: OriginFor<T>,
			trading_account: TradingAccountMinimal,
			collateral_id: u128,
			amount: FixedI128,
		) -> DispatchResult {
			ensure_signed(origin)?;

			// Call the internal function to facililate the deposit
			Self::deposit_internal(trading_account, collateral_id, amount);
			Ok(())
		}

		// TODO(merkle-groot): To be removed in production
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
				account_id = Self::get_trading_account_id(element);

				// Check if the account already exists
				ensure!(!AccountMap::<T>::contains_key(account_id), Error::<T>::DuplicateAccount);
				let trading_account: TradingAccount = TradingAccount {
					account_id,
					account_address: element.account_address,
					index: element.index,
					pub_key: element.pub_key,
				};

				AccountMap::<T>::insert(account_id, trading_account);
				AccountsListMap::<T>::insert(current_length, account_id);
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

		// TODO(merkle-groot): To be removed in production
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
				if let Some(asset) = T::AssetPallet::get_asset(element.asset_id) {
					ensure!(asset.is_collateral, Error::<T>::AssetNotCollateral);
				} else {
					ensure!(false, Error::<T>::AssetNotFound);
				}

				let current_balance: FixedI128 =
					BalancesMap::<T>::get(account_id, element.asset_id);
				if current_balance == FixedI128::zero() {
					Self::add_collateral(account_id, element.asset_id);
				}
				// Update the map with new balance
				BalancesMap::<T>::set(account_id, element.asset_id, element.balance_value);

				let account = AccountMap::<T>::get(&account_id)
					.ok_or(Error::<T>::AccountDoesNotExist)?
					.to_trading_account_minimal();
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

		/// Set standard withdrawal fee
		#[pallet::weight(0)]
		pub fn set_standard_withdrawal_fee(
			origin: OriginFor<T>,
			withdrawal_fee: FixedI128,
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			ensure!(withdrawal_fee >= FixedI128::zero(), Error::<T>::InvalidWithdrawalFee);
			StandardWithdrawalFee::<T>::put(withdrawal_fee);
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

			// Liquidation and deleverage flags must be false
			let force_closure_flag = T::TradingPallet::get_force_closure_flags(
				withdrawal_request.account_id,
				withdrawal_request.collateral_id,
			);
			ensure!(force_closure_flag.is_none(), Error::<T>::ForceClosureFlagSet);

			// Check if the signature is valid
			Self::verify_signature(&withdrawal_request)?;

			// Get the standard fee for a withdrawal tx
			let withdrawal_fee = StandardWithdrawalFee::<T>::get();

			// Get the current balance
			let current_balance: FixedI128 = BalancesMap::<T>::get(
				withdrawal_request.account_id,
				withdrawal_request.collateral_id,
			);

			// Get the new balance of the user
			let new_balance = current_balance - withdrawal_fee;

			// Get the account struct
			let account = AccountMap::<T>::get(&withdrawal_request.account_id)
				.ok_or(Error::<T>::AccountDoesNotExist)?
				.to_trading_account_minimal();

			// Get the current block number
			let block_number = <frame_system::Pallet<T>>::block_number();

			// Update the balance, after deducting fees
			BalancesMap::<T>::set(
				withdrawal_request.account_id,
				withdrawal_request.collateral_id,
				new_balance,
			);

			// BalanceUpdated event is emitted for reducing the withdrawal fee
			Self::deposit_event(Event::BalanceUpdated {
				account_id: withdrawal_request.account_id,
				account: account.clone(),
				collateral_id: withdrawal_request.collateral_id,
				amount: withdrawal_request.amount,
				modify_type: FundModifyType::Decrease.into(),
				reason: BalanceChangeReason::WithdrawalFee.into(),
				previous_balance: current_balance,
				new_balance,
				block_number,
			});

			// Check whether the withdrawal leads to the position to be liquidatable or deleveraged
			let (_, withdrawable_amount) = Self::calculate_amount_to_withdraw(
				withdrawal_request.account_id,
				withdrawal_request.collateral_id,
			);

			ensure!(
				withdrawal_request.amount <= withdrawable_amount,
				Error::<T>::InvalidWithdrawalRequest
			);

			// Update the balance
			BalancesMap::<T>::set(
				withdrawal_request.account_id,
				withdrawal_request.collateral_id,
				new_balance - withdrawal_request.amount,
			);

			// BalanceUpdated event is emitted for reducing the withdrawal amount
			Self::deposit_event(Event::BalanceUpdated {
				account_id: withdrawal_request.account_id,
				account: account.clone(),
				collateral_id: withdrawal_request.collateral_id,
				amount: withdrawal_request.amount,
				modify_type: FundModifyType::Decrease.into(),
				reason: BalanceChangeReason::Withdrawal.into(),
				previous_balance: new_balance,
				new_balance: new_balance - withdrawal_request.amount,
				block_number,
			});

			Self::deposit_event(Event::UserWithdrawal {
				trading_account: account.clone(),
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

			if market_price == FixedI128::zero() {
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

				// Get Index price
				let market_price = T::PricesPallet::get_index_price(curr_market_id);

				if market_price == FixedI128::zero() {
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
			// Convert the r and s value to fieldElement
			let (sig_r, sig_s) =
				sig_u256_to_sig_felt(&withdrawal_request.sig_r, &withdrawal_request.sig_s)
					.map_err(|_| Error::<T>::InvalidSignatureFelt)?;

			// Construct the signature struct
			let signature = Signature { r: sig_r, s: sig_s };

			// Hash the withdrawal request struct
			let withdrawal_request_hash = withdrawal_request
				.hash(&withdrawal_request.hash_type)
				.map_err(|_| Error::<T>::InvalidWithdrawalRequestHash)?;

			// Check if the withdrawal is already processed
			let withdrawal_request_hash_u256 = withdrawal_request_hash.to_u256();
			ensure!(
				!IsWithdrawalProcessed::<T>::contains_key(withdrawal_request_hash_u256),
				Error::<T>::DuplicateWithdrawal
			);

			// Fetch the public key of account
			let public_key = Self::get_public_key(&withdrawal_request.account_id)
				.ok_or(Error::<T>::NoPublicKeyFound)?;

			// Convert the public key to felt
			let public_key_felt =
				public_key.try_to_felt().map_err(|_| Error::<T>::InvalidPublicKey)?;

			// Verify the signature
			let verification = ecdsa_verify(&public_key_felt, &withdrawal_request_hash, &signature)
				.map_err(|_| Error::<T>::InvalidSignature)?;

			// Signature verification returned error or false
			ensure!(verification, Error::<T>::InvalidSignature);

			// Mark the request as being processed
			IsWithdrawalProcessed::<T>::insert(withdrawal_request_hash_u256, true);

			Ok(())
		}

		fn calculate_amount_to_withdraw(
			account_id: U256,
			collateral_id: u128,
		) -> (FixedI128, FixedI128) {
			// Get the current balance
			let current_balance: FixedI128 = BalancesMap::<T>::get(account_id, collateral_id);
			if current_balance <= FixedI128::zero() {
				return (FixedI128::zero(), FixedI128::zero());
			}

			let (
				liq_result,
				total_account_value,
				_,
				_,
				total_maintenance_requirement,
				_,
				least_collateral_ratio_position,
				_,
			) = Self::get_margin_info(account_id, collateral_id, FixedI128::zero(), FixedI128::zero());

			// if TMR == 0, it means that market price is not within TTL, so user should be possible to withdraw whole balance
			if total_maintenance_requirement == FixedI128::zero() {
				return (current_balance, current_balance);
			}

			// if TAV <= 0, it means that user is already under water and thus withdrawal is not possible
			if total_account_value <= FixedI128::zero() {
				return (FixedI128::zero(), FixedI128::zero());
			}

			let safe_withdrawal_amount;
			// Returns 0, if the position is to be deleveraged or liquiditable
			if liq_result == true {
				safe_withdrawal_amount = FixedI128::zero();
			} else {
				let safe_amount = total_account_value - total_maintenance_requirement;
				if current_balance < safe_amount {
					return (current_balance, current_balance);
				}
				safe_withdrawal_amount = safe_amount;
			}

			let withdrawable_amount = Self::get_amount_to_withdraw(
				total_account_value,
				total_maintenance_requirement,
				least_collateral_ratio_position,
				current_balance,
			);

			return (safe_withdrawal_amount, withdrawable_amount);
		}

		fn get_amount_to_withdraw(
			total_account_value: FixedI128,
			total_maintenance_requirement: FixedI128,
			least_collateral_ratio_position: PositionDetailsForRiskManagement,
			current_balance: FixedI128,
		) -> FixedI128 {
			let two_point_five = FixedI128::from_inner(2500000000000000000);

			// This function will only be called in these cases:
			// i) if TAV < TMR ii) if (TAV - TMR) < balance
			// we calculate maximum amount that can be sold so that the position won't get liquidated
			// calculate new TAV and new TMR to get maximum withdrawable amount
			// amount_to_sell = initial_size - ((2.5 * margin_amount)/current_asset_price)

			// Get Market price
			let market_price =
				T::PricesPallet::get_index_price(least_collateral_ratio_position.market_id);

			let min_leverage_times_margin =
				two_point_five * least_collateral_ratio_position.margin_amount;
			let new_size = min_leverage_times_margin / market_price;

			// calculate account value and maintenance requirement of least collateral position before reducing size
			// AV = (size * current_price) - borrowed_amount
			// MR = req_margin * size * avg_execution_price
			let account_value_initial = (least_collateral_ratio_position.size * market_price)
				- least_collateral_ratio_position.borrowed_amount;

			let market =
				T::MarketPallet::get_market(least_collateral_ratio_position.market_id).unwrap();
			let req_margin = market.maintenance_margin_fraction;
			let leveraged_position_value_initial = least_collateral_ratio_position.size
				* least_collateral_ratio_position.avg_execution_price;
			let maintenance_requirement_initial = req_margin * leveraged_position_value_initial;

			// calculate account value and maintenance requirement of least collateral position after reducing size
			let amount_to_be_sold = least_collateral_ratio_position.size - new_size;
			let amount_to_be_sold_value = amount_to_be_sold * market_price;
			let new_borrowed_amount =
				least_collateral_ratio_position.borrowed_amount - amount_to_be_sold_value;
			let account_value_after = (new_size * market_price) - new_borrowed_amount;
			let leveraged_position_value_after =
				new_size * least_collateral_ratio_position.avg_execution_price;
			let maintenance_requirement_after = req_margin * leveraged_position_value_after;

			// calculate new TAV and new TMR after reducing size
			let account_value_difference = account_value_after - account_value_initial;
			let maintenance_requirement_difference =
				maintenance_requirement_after - maintenance_requirement_initial;
			let new_tav = total_account_value + account_value_difference;
			let new_tmr = total_maintenance_requirement + maintenance_requirement_difference;

			let new_sub_result = new_tav - new_tmr;
			if new_sub_result <= FixedI128::zero() {
				return FixedI128::zero();
			}
			if current_balance <= new_sub_result {
				return current_balance;
			} else {
				return new_sub_result;
			}
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

		fn get_trading_account_id(trading_account: TradingAccountMinimal) -> U256 {
			let mut result: [u8; 33] = [0; 33];
			trading_account.account_address.to_little_endian(&mut result[0..32]);
			result[32] = trading_account.index;

			blake2_256(&result).into()
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

		fn deposit_internal(
			trading_account: TradingAccountMinimal,
			collateral_id: u128,
			amount: FixedI128,
		) {
			let account_address = trading_account.account_address;
			let index = trading_account.index;
			let pub_key = trading_account.pub_key;

			// Generate trading account id
			let account_id = Self::get_trading_account_id(trading_account);

			// Check if the account is under risk-management
			if T::TradingPallet::get_force_closure_flags(account_id, collateral_id).is_some() {
				// Get the current balance
				let previous_deferred_balance =
					DeferredBalancesMap::<T>::get(account_id, collateral_id);

				// Save it to storage
				DeferredBalancesMap::<T>::insert(
					account_id,
					collateral_id,
					previous_deferred_balance + amount,
				);

				// Emit the deferred deposit event
				Self::deposit_event(Event::DeferredBalance { account_id, collateral_id, amount });
			}

			// Check if the account already exists, if it doesn't exist then create an account
			if !AccountMap::<T>::contains_key(&account_id) {
				let current_length = AccountsCount::<T>::get();

				let trading_account: TradingAccount =
					TradingAccount { account_id, account_address, index, pub_key };
				AccountMap::<T>::insert(account_id, trading_account);
				AccountsListMap::<T>::insert(current_length, account_id);
				AccountsCount::<T>::put(current_length + 1);

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

		fn get_account_list(start_index: u128, end_index: u128) -> Vec<U256> {
			let mut account_list = Vec::<U256>::new();
			let accounts_count = AccountsCount::<T>::get();
			for index in start_index..end_index {
				if (start_index > end_index)
					|| (index >= accounts_count)
					|| (start_index >= accounts_count)
				{
					break;
				}
				account_list.push(AccountsListMap::<T>::get(index).unwrap());
			}
			account_list
		}

		fn add_deferred_balance(account_id: U256, collateral_id: u128) -> DispatchResult {
			// Get the current deferred balance
			let deferred_balance = DeferredBalancesMap::<T>::get(account_id, collateral_id);

			if deferred_balance != FixedI128::zero() {
				// Get the current balance
				let previous_balance = BalancesMap::<T>::get(account_id, collateral_id);

				// Calculate the new balance
				let new_balance = previous_balance + deferred_balance;

				// Update the balance
				BalancesMap::<T>::insert(account_id, collateral_id, new_balance);

				// Reset the deferred balance
				DeferredBalancesMap::<T>::insert(account_id, collateral_id, FixedI128::zero());

				// Get the account details for the event
				let account = AccountMap::<T>::get(&account_id)
					.ok_or(Error::<T>::AccountDoesNotExist)?
					.to_trading_account_minimal();

				// Emit the balance updated event
				Self::deposit_event(Event::BalanceUpdated {
					account_id,
					account,
					collateral_id,
					amount: deferred_balance,
					modify_type: FundModifyType::Increase.into(),
					reason: BalanceChangeReason::DeferredDeposit.into(),
					previous_balance,
					new_balance,
					block_number: <frame_system::Pallet<T>>::block_number(),
				});
			}

			Ok(())
		}
	}
}
