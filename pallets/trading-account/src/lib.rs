#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod migrations;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use super::*;
	use core::cmp::{max, min};
	use frame_support::{
		dispatch::Vec,
		pallet_prelude::{OptionQuery, *},
		traits::UnixTime,
		Blake2_128Concat,
	};
	use frame_system::pallet_prelude::*;
	use pallet_support::{
		ecdsa_verify,
		helpers::{get_day_diff, shift_and_recompute, sig_u256_to_sig_felt},
		traits::{
			AssetInterface, FieldElementExt, FixedI128Ext, Hashable, MarketInterface,
			PricesInterface, TradingAccountInterface, TradingInterface, U256Ext,
		},
		types::{
			BalanceChangeReason, BalanceUpdate, Direction, FeeSharesInput, FundModifyType,
			InsuranceWithdrawalRequest, MonetaryAccountDetails, Position, ReferralDetails,
			TradingAccount, TradingAccountMinimal, VolumeType, WithdrawalRequest,
		},
		Signature,
	};
	use primitive_types::U256;
	use sp_arithmetic::{fixed_point::FixedI128, traits::Zero, FixedPointNumber};
	use sp_io::hashing::blake2_256;

	#[cfg(not(feature = "dev"))]
	pub const IS_DEV_ENABLED: bool = false;

	#[cfg(feature = "dev")]
	pub const IS_DEV_ENABLED: bool = true;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type AssetPallet: AssetInterface;
		type TradingPallet: TradingInterface;
		type MarketPallet: MarketInterface;
		type PricesPallet: PricesInterface;
		type TimeProvider: UnixTime;
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
	#[pallet::getter(fn monetary_account_volume)]
	// Maps from (monetary_account_address,collateral_id)-> volume vector
	// This stores 31 days of volume, starting from day of last trade (index 0 stores volume for
	// most recent day of trade)
	pub(super) type MonetaryAccountVolumeMap<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		U256,
		Blake2_128Concat,
		u128,
		Vec<FixedI128>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn monetary_account_tx_timestamp)]
	// Maps from (monetary_account_address, collateral_id) -> timestamp for last trade
	pub(super) type MonetaryAccountLastTxTimestamp<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, U256, Blake2_128Concat, u128, u64, OptionQuery>;

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
	#[pallet::getter(fn monetary_to_trading_accounts)]
	// Here, key is the Monetary_account_address and value is the vector of trading_account_id's
	pub(super) type MonetaryToTradingAccountsMap<T: Config> =
		StorageMap<_, Blake2_128Concat, U256, Vec<U256>, ValueQuery>;

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

	#[pallet::storage]
	#[pallet::getter(fn master_account)]
	// Here, key1 is referral monetary account address and value is referral details with master
	// monetary address
	pub(super) type MasterAccountMap<T: Config> =
		StorageMap<_, Twox64Concat, U256, ReferralDetails, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn referral_accounts)]
	// Here, key1 is (master monetary account address, index) and value is referral
	// monetary account addresses
	pub(super) type ReferralAccountsMap<T: Config> =
		StorageMap<_, Twox64Concat, (U256, u64), U256, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn referrals_count)]
	// Here, key1 is monetary account address and value is number of referrals
	pub(super) type ReferralsCountMap<T: Config> =
		StorageMap<_, Twox64Concat, U256, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn master_account_level)]
	// It stores master account level
	// Here, key1 is the master account address and the values is the level
	pub(super) type MasterAccountLevel<T: Config> =
		StorageMap<_, Twox64Concat, U256, u8, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn master_account_volume)]
	// Maps from (monetary_account_address,collateral_id)-> volume vector
	// This stores 31 days of volume, starting from day of last trade (index 0 stores volume for
	// most recent day of trade)
	pub(super) type MasterAccountVolumeMap<T: Config> =
		StorageDoubleMap<_, Twox64Concat, U256, Twox64Concat, u128, Vec<FixedI128>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn master_account_tx_timestamp)]
	// Maps from (monetary_account_address, collateral_id) -> timestamp for last trade
	pub(super) type MasterAccountLastTxTimestamp<T: Config> =
		StorageDoubleMap<_, Twox64Concat, U256, Twox64Concat, u128, u64, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn master_account_fee_share)]
	// Maps from (monetary_account_address, collateral_id) -> accumulated fee share
	pub(super) type MasterAccountFeeShare<T: Config> =
		StorageDoubleMap<_, Twox64Concat, U256, Twox64Concat, u128, FixedI128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn insurance_fund_balance)]
	// Stores balance of Insurance Funds
	// Here, key1 is the insurance fund address, key2 is the collateral_id and the value is the
	// balance
	pub(super) type InsuranceFundBalances<T: Config> =
		StorageDoubleMap<_, Twox64Concat, U256, Twox64Concat, u128, FixedI128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn market_to_insurance_fund)]
	// Stores balance of Insurance Funds
	// Here, key1 is market_id, the value is the insurance_fund address and the revenue split
	// fraction
	pub(super) type MarketToFeeSplitMap<T: Config> =
		StorageMap<_, Twox64Concat, u128, (U256, FixedI128), OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn default_insurance_fund)]
	// Stores the default insurance fund
	// Here, the value is the Starknet address of the default Insurance fund
	pub(super) type DefaultInsuranceFund<T: Config> = StorageValue<_, U256, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn insurance_withdrawal_signer)]
	// Stores the signer that is authorized to do withdrawals
	// Here, the value is the pubkey of the signer
	pub(super) type InsuranceWithdrawalSigner<T: Config> = StorageValue<_, U256, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn standard_withdrawal_fee_v2)]
	// It stores the standard withdrawal fee
	// Here, the key_1 is the collateral_id and the value is the fee
	pub(super) type StandardWithdrawalFeeV2<T: Config> =
		StorageMap<_, Twox64Concat, u128, FixedI128, ValueQuery>;

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
		/// Invalid withdrawal request hash - withdrawal request could not be hashed into a Field
		/// Element
		InvalidWithdrawalRequestHash,
		/// Invalid Signature Field Elements - sig_r and/or sig_s could not be converted into a
		/// Signature
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
		/// Market does not exist
		MarketDoesNotExist,
		/// Invalid Call to dev mode only function
		DevOnlyCall,
		/// Invalid amount passed to pay_fee_shares fn
		InvalidFeeSharesAmount { invalid_index: u16 },
		/// Withdrawal amount not set
		ZeroWithdrawalSigner,
		/// Zero pub key sent
		ZeroSigner,
		/// Zero address passed for insurance withdrawal
		ZeroRecipient,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Several accounts added
		AccountsAdded {
			length: u128,
		},
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
		DeferredBalance {
			account_id: U256,
			collateral_id: u128,
			amount: FixedI128,
		},
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
		AccountCreated {
			account_id: U256,
			account_address: U256,
			index: u8,
		},
		/// Amount passed to transfer/transfer_from functions is negative
		AmountIsNegative {
			account_id: U256,
			collateral_id: u128,
			amount: FixedI128,
			reason: u8,
		},
		/// Insurance fund updation event
		InsuranceFundChange {
			collateral_id: u128,
			amount: FixedI128,
			modify_type: FundModifyType,
			block_number: BlockNumberFor<T>,
		},
		ReferralDetailsAdded {
			master_account_address: U256,
			referral_account_address: U256,
			fee_discount: FixedI128,
			referral_code: U256,
		},
		FeeShareTransfer {
			master_account_address: U256,
			collateral_id: u128,
			amount: FixedI128,
			block_number: BlockNumberFor<T>,
		},
		MasterAccountLevelChanged {
			master_account_address: U256,
			level: u8,
		},
		/// Event to be synced by L2, for pnl changes
		UserBalanceChangeV2 {
			trading_account: TradingAccountMinimal,
			market_id: u128,
			amount: FixedI128,
			revenue_amount: FixedI128,
			fee_share_amount: FixedI128,
			modify_type: FundModifyType,
			reason: u8,
			block_number: BlockNumberFor<T>,
		},
		/// Insurance fund updation event
		InsuranceFundChangeV2 {
			market_id: u128,
			amount: FixedI128,
			modify_type: FundModifyType,
			block_number: BlockNumberFor<T>,
		},
		FeeShareTransferV2 {
			master_account_address: U256,
			market_id: u128,
			amount: FixedI128,
			block_number: BlockNumberFor<T>,
		},
		InsuranceFundWithdrawal {
			insurance_fund: U256,
			recipient: U256,
			collateral_id: u128,
			amount: FixedI128,
			block_number: BlockNumberFor<T>,
		},
		UserBalanceDeficit {
			trading_account: TradingAccountMinimal,
			collateral_id: u128,
			amount: FixedI128,
			block_number: BlockNumberFor<T>,
		},
		UserWithdrawalFee {
			trading_account: TradingAccountMinimal,
			collateral_id: u128,
			amount: FixedI128,
			block_number: BlockNumberFor<T>,
		},
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_runtime_upgrade() -> frame_support::weights::Weight {
			migrations::migrations::migrate_to_v2::<T>()
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn update_monetary_to_trading_accounts(
			origin: OriginFor<T>,
			monetary_accounts: Vec<MonetaryAccountDetails>,
		) -> DispatchResult {
			ensure_root(origin)?;

			for account in monetary_accounts {
				for trading_account in account.trading_accounts {
					let trading_accounts =
						MonetaryToTradingAccountsMap::<T>::get(account.monetary_account);
					if !trading_accounts.contains(&trading_account) {
						MonetaryToTradingAccountsMap::<T>::append(
							account.monetary_account,
							trading_account,
						);
					}
				}
			}
			Ok(())
		}

		// TODO(merkle-groot): To be removed in production
		/// To test depositing funds
		#[pallet::weight(0)]
		pub fn deposit(
			origin: OriginFor<T>,
			trading_account: TradingAccountMinimal,
			collateral_id: u128,
			amount: FixedI128,
		) -> DispatchResult {
			if !IS_DEV_ENABLED {
				return Err(Error::<T>::DevOnlyCall.into())
			}
			ensure_signed(origin)?;

			// Call the internal function to facililate the deposit
			Self::deposit_internal(trading_account, collateral_id, amount);
			Ok(())
		}

		// TODO(merkle-groot): To be removed in production
		/// To test setting of insurance funds
		#[pallet::weight(0)]
		pub fn update_fee_split_details(
			origin: OriginFor<T>,
			market_id: u128,
			insurance_fund: U256,
			fee_split: FixedI128,
		) -> DispatchResult {
			if !IS_DEV_ENABLED {
				return Err(Error::<T>::DevOnlyCall.into())
			}
			ensure_signed(origin)?;

			Self::update_fee_split_details_internal(market_id, insurance_fund, fee_split);
			Ok(())
		}

		// TODO(merkle-groot): To be removed in production
		/// To test setting of default insurance funds
		#[pallet::weight(0)]
		pub fn set_default_insurance_fund(
			origin: OriginFor<T>,
			insurance_fund: U256,
		) -> DispatchResult {
			ensure_root(origin)?;

			DefaultInsuranceFund::<T>::set(Some(insurance_fund));

			Ok(())
		}

		// TODO(merkle-groot): To be removed in production
		/// To test setting of default insurance funds
		#[pallet::weight(0)]
		pub fn update_insurance_fund_balance(
			origin: OriginFor<T>,
			insurance_fund: U256,
			collateral_id: u128,
			amount: FixedI128,
		) -> DispatchResult {
			if !IS_DEV_ENABLED {
				return Err(Error::<T>::DevOnlyCall.into())
			}
			ensure_signed(origin)?;

			Self::update_insurance_fund_balance_internal(insurance_fund, collateral_id, amount);
			Ok(())
		}

		// TODO(merkle-groot): To be removed in production
		/// Add several accounts together
		#[pallet::weight(0)]
		pub fn add_accounts(
			origin: OriginFor<T>,
			accounts: Vec<TradingAccountMinimal>,
		) -> DispatchResult {
			if !IS_DEV_ENABLED {
				return Err(Error::<T>::DevOnlyCall.into())
			}
			let _ = ensure_signed(origin)?;

			let length: u128 = u128::try_from(accounts.len()).unwrap();
			let mut current_length = AccountsCount::<T>::get();
			let final_length: u128 = length + current_length;
			let mut account_id: U256;

			for element in accounts {
				account_id = Self::get_trading_account_id(element);

				// Check if the account already exists
				ensure!(!AccountMap::<T>::contains_key(account_id), Error::<T>::DuplicateAccount);

				MonetaryToTradingAccountsMap::<T>::append(element.account_address, account_id);

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
			if !IS_DEV_ENABLED {
				return Err(Error::<T>::DevOnlyCall.into())
			}
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

		#[pallet::weight(0)]
		pub fn update_insurance_withdrawal_signer(
			origin: OriginFor<T>,
			pub_key: U256,
		) -> DispatchResult {
			ensure_root(origin)?;

			// The pub key cannot be 0
			ensure!(pub_key != U256::zero(), Error::<T>::ZeroSigner);

			// Store the new signer
			InsuranceWithdrawalSigner::<T>::set(Some(pub_key));

			// Return ok
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn insurance_withdraw(
			origin: OriginFor<T>,
			insurance_withdrawal_request: InsuranceWithdrawalRequest,
		) -> DispatchResult {
			ensure_signed(origin)?;

			ensure!(
				insurance_withdrawal_request.recipient != U256::zero(),
				Error::<T>::ZeroRecipient
			);

			// Check if the signature is valid
			Self::verify_insurance_withdrawal_signature(&insurance_withdrawal_request)?;

			// Get the current balance
			let current_balance = InsuranceFundBalances::<T>::get(
				insurance_withdrawal_request.insurance_fund,
				insurance_withdrawal_request.collateral_id,
			);

			// Get the new balance of the user
			let new_balance = current_balance - insurance_withdrawal_request.amount;
			ensure!(new_balance >= FixedI128::zero(), Error::<T>::InsufficientBalance);

			// Update the balance, after deducting fees
			InsuranceFundBalances::<T>::set(
				insurance_withdrawal_request.insurance_fund,
				insurance_withdrawal_request.collateral_id,
				new_balance,
			);

			Self::deposit_event(Event::InsuranceFundWithdrawal {
				insurance_fund: insurance_withdrawal_request.insurance_fund,
				recipient: insurance_withdrawal_request.recipient,
				collateral_id: insurance_withdrawal_request.collateral_id,
				amount: insurance_withdrawal_request.amount,
				block_number: <frame_system::Pallet<T>>::block_number(),
			});

			Ok(())
		}

		/// To test adding of referral
		#[pallet::weight(0)]
		pub fn add_referral(
			origin: OriginFor<T>,
			referral_account_address: U256,
			referral_details: ReferralDetails,
			referral_code: U256,
		) -> DispatchResult {
			if !IS_DEV_ENABLED {
				return Err(Error::<T>::DevOnlyCall.into())
			}
			ensure_signed(origin)?;

			// Call the internal function to add referral
			Self::add_referral_internal(referral_account_address, referral_details, referral_code);
			Ok(())
		}

		/// To test updating master account level
		#[pallet::weight(0)]
		pub fn update_master_account_level(
			origin: OriginFor<T>,
			master_account_address: U256,
			level: u8,
		) -> DispatchResult {
			if !IS_DEV_ENABLED {
				return Err(Error::<T>::DevOnlyCall.into())
			}
			ensure_signed(origin)?;

			// Call the internal function to update account level
			Self::modify_master_account_level(master_account_address, level);
			Ok(())
		}

		/// Set standard withdrawal fee
		#[pallet::weight(0)]
		pub fn set_standard_withdrawal_fee(
			origin: OriginFor<T>,
			collateral_id: u128,
			withdrawal_fee: FixedI128,
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			ensure!(withdrawal_fee >= FixedI128::zero(), Error::<T>::InvalidWithdrawalFee);
			StandardWithdrawalFeeV2::<T>::set(collateral_id, withdrawal_fee);
			Ok(())
		}

		/// Adjust balances which were not rounded correctly
		#[pallet::weight(0)]
		pub fn adjust_balances(
			origin: OriginFor<T>,
			start_index: u128,
			end_index: u128,
			precision: u32,
		) -> DispatchResult {
			let _ = ensure_root(origin)?;
			let collateral_id = T::AssetPallet::get_default_collateral();

			for i in start_index..=end_index {
				let account_id = AccountsListMap::<T>::get(i);
				if account_id.is_none() {
					break;
				}
				let account_id = account_id.unwrap();
				let current_balance: FixedI128 = BalancesMap::<T>::get(account_id, collateral_id);
				let adjusted_balance = current_balance.floor_with_precision(precision);
				BalancesMap::<T>::set(account_id, collateral_id, adjusted_balance);
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

			// Liquidation and deleverage flags must be false
			let force_closure_flag = T::TradingPallet::get_force_closure_flags(
				withdrawal_request.account_id,
				withdrawal_request.collateral_id,
			);
			ensure!(force_closure_flag.is_none(), Error::<T>::ForceClosureFlagSet);

			// Check if the signature is valid
			Self::verify_signature(&withdrawal_request)?;

			// Get the standard fee for a withdrawal tx
			let withdrawal_fee =
				StandardWithdrawalFeeV2::<T>::get(withdrawal_request.collateral_id);

			// Get the current balance
			let current_balance: FixedI128 = BalancesMap::<T>::get(
				withdrawal_request.account_id,
				withdrawal_request.collateral_id,
			);

			// Get the account struct
			let account = AccountMap::<T>::get(&withdrawal_request.account_id)
				.ok_or(Error::<T>::AccountDoesNotExist)?
				.to_trading_account_minimal();

			// Get the current block number
			let block_number = <frame_system::Pallet<T>>::block_number();

			// Get the new balance of the user
			let new_balance = current_balance - withdrawal_fee;
			// Update the balance, after deducting fees
			BalancesMap::<T>::set(
				withdrawal_request.account_id,
				withdrawal_request.collateral_id,
				new_balance,
			);

			if withdrawal_fee != FixedI128::zero() {
				Self::deposit_event(Event::UserWithdrawalFee {
					trading_account: account,
					collateral_id: withdrawal_request.collateral_id,
					amount: withdrawal_fee,
					block_number,
				});

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

				if let Some(insurance_fund) = DefaultInsuranceFund::<T>::get() {
					let current_insurance_fund_balance = InsuranceFundBalances::<T>::get(
						insurance_fund,
						withdrawal_request.collateral_id,
					);

					// Increment the local balance of insurance fund
					InsuranceFundBalances::<T>::set(
						insurance_fund,
						withdrawal_request.collateral_id,
						current_insurance_fund_balance + withdrawal_fee,
					);
				}
			}

			// Get withdrawal amount before withdrawal leads to the position to be liquidatable or
			// deleveraged
			let safe_withdrawal_amount = Self::calculate_amount_to_withdraw(
				withdrawal_request.account_id,
				withdrawal_request.collateral_id,
			);

			ensure!(
				withdrawal_request.amount <= safe_withdrawal_amount,
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

		#[pallet::weight(0)]
		pub fn pay_fee_shares(
			origin: OriginFor<T>,
			fee_shares_inputs: Vec<FeeSharesInput>,
		) -> DispatchResult {
			ensure_signed(origin)?;

			for (index, fee_shares_input) in fee_shares_inputs.iter().enumerate() {
				let FeeSharesInput { master_account_address, collateral_id, amount } =
					*fee_shares_input;

				let fee_share =
					MasterAccountFeeShare::<T>::get(master_account_address, collateral_id);

				// If the passed amount is invalid, we return an error
				ensure!(
					amount <= fee_share && amount > FixedI128::zero(),
					Error::<T>::InvalidFeeSharesAmount { invalid_index: index as u16 }
				);

				// Reduce the fee share
				MasterAccountFeeShare::<T>::set(
					master_account_address,
					collateral_id,
					fee_share - amount,
				);

				Self::deposit_event(Event::FeeShareTransfer {
					master_account_address,
					collateral_id,
					amount,
					block_number: <frame_system::Pallet<T>>::block_number(),
				});
			}

			Ok(())
		}
	}

	// Pallet internal functions
	impl<T: Config> Pallet<T> {
		fn add_collateral(account_id: U256, collateral_id: u128) {
			let mut collaterals = AccountCollateralsMap::<T>::get(account_id);
			for element in &collaterals {
				if element == &collateral_id {
					return
				}
			}

			collaterals.push(collateral_id);
			AccountCollateralsMap::<T>::insert(account_id, collaterals);
		}

		fn get_risk_parameters(
			position: &Position,
			direction: Direction,
			mark_price: FixedI128,
			market_id: u128,
		) -> (FixedI128, FixedI128) {
			let market = T::MarketPallet::get_market(market_id).unwrap();
			let req_margin_fraction = market.maintenance_margin_fraction;

			// Calculate the maintenance requirement
			let maintenance_position = mark_price * position.size;
			let maintenance_requirement = req_margin_fraction * maintenance_position;

			if mark_price == FixedI128::zero() {
				return (0.into(), maintenance_requirement)
			}

			// Calculate pnl
			let price_diff = if direction == Direction::Long {
				mark_price - position.avg_execution_price
			} else {
				position.avg_execution_price - mark_price
			};

			let pnl = price_diff * position.size;

			return (pnl, maintenance_requirement)
		}

		fn calculate_margin_info(
			account_id: U256,
			new_position_maintanence_requirement: FixedI128,
			markets: Vec<u128>,
		) -> (FixedI128, FixedI128, FixedI128) {
			let mut unrealized_pnl_sum: FixedI128 = FixedI128::zero();
			let mut negative_unrealized_pnl_sum = FixedI128::zero();
			let mut maintenance_margin_requirement: FixedI128 =
				new_position_maintanence_requirement;
			for curr_market_id in markets {
				// Get Long position
				let long_position: Position =
					T::TradingPallet::get_position(account_id, curr_market_id, Direction::Long);

				// Get Short position
				let short_position: Position =
					T::TradingPallet::get_position(account_id, curr_market_id, Direction::Short);

				// Get Mark price
				let mark_price = T::PricesPallet::get_mark_price(curr_market_id);

				if mark_price == FixedI128::zero() {
					return (0.into(), 0.into(), 0.into())
				}

				let long_maintanence_requirement;
				let long_pnl;

				if long_position.size == 0.into() {
					long_maintanence_requirement = 0.into();
					long_pnl = 0.into();
				} else {
					// Get risk parameters of the position
					(long_pnl, long_maintanence_requirement) = Self::get_risk_parameters(
						&long_position,
						Direction::Long,
						mark_price,
						curr_market_id,
					);
				}

				let short_maintanence_requirement;
				let short_pnl;

				if short_position.size == 0.into() {
					short_maintanence_requirement = 0.into();
					short_pnl = 0.into();
				} else {
					// Get risk parameters of the position
					(short_pnl, short_maintanence_requirement) = Self::get_risk_parameters(
						&short_position,
						Direction::Short,
						mark_price,
						curr_market_id,
					);
				}

				unrealized_pnl_sum = unrealized_pnl_sum + short_pnl + long_pnl;

				if short_pnl.is_negative() {
					negative_unrealized_pnl_sum = negative_unrealized_pnl_sum + short_pnl;
				}

				if long_pnl.is_negative() {
					negative_unrealized_pnl_sum = negative_unrealized_pnl_sum + long_pnl;
				}

				maintenance_margin_requirement = maintenance_margin_requirement +
					short_maintanence_requirement +
					long_maintanence_requirement;
			}
			return (unrealized_pnl_sum, maintenance_margin_requirement, negative_unrealized_pnl_sum)
		}

		fn verify_insurance_withdrawal_signature(
			insurance_withdrawl_request: &InsuranceWithdrawalRequest,
		) -> Result<(), Error<T>> {
			// Convert the r and s value to fieldElement
			let (sig_r, sig_s) = sig_u256_to_sig_felt(
				&insurance_withdrawl_request.sig_r,
				&insurance_withdrawl_request.sig_s,
			)
			.map_err(|_| Error::<T>::InvalidSignatureFelt)?;

			// Construct the signature struct
			let signature = Signature { r: sig_r, s: sig_s };

			// Hash the withdrawal request struct
			let withdrawal_request_hash = insurance_withdrawl_request
				.hash(&insurance_withdrawl_request.hash_type)
				.map_err(|_| Error::<T>::InvalidWithdrawalRequestHash)?;

			// Check if the withdrawal is already processed
			let withdrawal_request_hash_u256 = withdrawal_request_hash.to_u256();
			ensure!(
				!IsWithdrawalProcessed::<T>::contains_key(withdrawal_request_hash_u256),
				Error::<T>::DuplicateWithdrawal
			);

			// Fetch the public key of signer
			let public_key =
				InsuranceWithdrawalSigner::<T>::get().ok_or(Error::<T>::NoPublicKeyFound)?;

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

		fn calculate_amount_to_withdraw(account_id: U256, collateral_id: u128) -> FixedI128 {
			// Get the current balance
			let current_balance: FixedI128 = BalancesMap::<T>::get(account_id, collateral_id);
			let margin_locked: FixedI128 = LockedMarginMap::<T>::get(account_id, collateral_id);

			if current_balance <= FixedI128::zero() {
				return FixedI128::zero()
			}

			let (
				liq_result,
				total_account_value,
				_,
				_,
				total_maintenance_requirement,
				negative_unrealized_pnl_sum,
			) = Self::get_margin_info(account_id, collateral_id, FixedI128::zero(), FixedI128::zero());

			// if TMR == 0, it means that market price is not within TTL, so user should be possible
			// to withdraw whole balance
			if total_maintenance_requirement == FixedI128::zero() {
				return current_balance
			}

			// if TAV <= 0, it means that user is already under water and thus withdrawal is not
			// possible
			if total_account_value <= FixedI128::zero() {
				return FixedI128::zero()
			}

			// Returns 0, if the position is to be deleveraged or liquiditable
			if liq_result == true {
				return FixedI128::zero()
			} else {
				return current_balance - max(total_maintenance_requirement, margin_locked) +
					negative_unrealized_pnl_sum
			}
		}

		fn modify_master_account_level(master_account_address: U256, level: u8) {
			MasterAccountLevel::<T>::set(master_account_address, level);

			Self::deposit_event(Event::MasterAccountLevelChanged { master_account_address, level })
		}
	}

	impl<T: Config> TradingAccountInterface for Pallet<T> {
		// Error type used for trade volume instrumentation error
		type VolumeError = Error<T>;
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
			market_id: u128,
			amount: FixedI128,
			reason: BalanceChangeReason,
		) {
			if amount.is_negative() {
				Self::deposit_event(Event::AmountIsNegative {
					account_id,
					collateral_id,
					amount,
					reason: reason.into(),
				});
				return
			}
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
			Self::deposit_event(Event::UserBalanceChangeV2 {
				trading_account: account,
				market_id,
				amount,
				revenue_amount: FixedI128::zero(),
				fee_share_amount: FixedI128::zero(),
				modify_type: FundModifyType::Increase,
				reason: reason.into(),
				block_number,
			});
		}

		fn transfer_from(
			account_id: U256,
			collateral_id: u128,
			market_id: u128,
			amount: FixedI128,
			reason: BalanceChangeReason,
		) {
			if amount.is_negative() {
				Self::deposit_event(Event::AmountIsNegative {
					account_id,
					collateral_id,
					amount,
					reason: reason.into(),
				});
				return
			}
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

			if reason != BalanceChangeReason::Fee {
				Self::deposit_event(Event::UserBalanceChangeV2 {
					trading_account: account,
					market_id,
					amount,
					revenue_amount: FixedI128::zero(),
					fee_share_amount: FixedI128::zero(),
					modify_type: FundModifyType::Decrease.into(),
					reason: reason.into(),
					block_number,
				});
			}
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
		) -> (bool, FixedI128, FixedI128, FixedI128, FixedI128, FixedI128) {
			// Get markets corresponding of the collateral
			let markets: Vec<u128> =
				T::TradingPallet::get_markets_of_collateral(account_id, collateral_id);

			let collateral_asset = T::AssetPallet::get_asset(collateral_id).unwrap();
			let collateral_token_decimal = collateral_asset.decimals;

			// Get balance for the given collateral
			let collateral_balance = BalancesMap::<T>::get(account_id, collateral_id);

			// Get the sum of initial margin of all positions under the given collateral
			let initial_margin_sum = LockedMarginMap::<T>::get(account_id, collateral_id);

			if markets.len() == 0 {
				let available_margin = collateral_balance - new_position_margin;
				return (
					false,              // is_liquidation
					collateral_balance, // total_margin
					available_margin,   // available_margin
					0.into(),           // unrealized_pnl_sum
					0.into(),           // maintenance_margin_requirement
					0.into(),           // negative_unrealized_pnl_sum
				)
			}

			let (unrealized_pnl_sum, maintenance_margin_requirement, negative_unrealized_pnl_sum) =
				Self::calculate_margin_info(
					account_id,
					new_position_maintanence_requirement,
					markets,
				);

			let unrealized_pnl_sum =
				unrealized_pnl_sum.round_to_precision(collateral_token_decimal.into());

			// Add the new position's margin
			let total_initial_margin_sum = initial_margin_sum + new_position_margin;

			// Compute total margin of the given collateral
			let total_margin = collateral_balance + unrealized_pnl_sum;

			// Compute available margin of the given collateral
			let available_margin = total_margin - total_initial_margin_sum;

			let mut is_liquidation = false;

			// If it's a long position with 1x leverage, ignore it
			if total_margin <= maintenance_margin_requirement {
				is_liquidation = true;
			}

			return (
				is_liquidation,
				total_margin,
				available_margin,
				unrealized_pnl_sum,
				maintenance_margin_requirement,
				negative_unrealized_pnl_sum,
			)
		}

		fn update_fee_split_details_internal(
			market_id: u128,
			insurance_fund: U256,
			fee_split: FixedI128,
		) {
			MarketToFeeSplitMap::<T>::set(market_id, Some((insurance_fund, fee_split)));
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
				MonetaryToTradingAccountsMap::<T>::append(account_address, account_id);
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
			let block_number = <frame_system::Pallet<T>>::block_number();

			// If the current balance is 0, then add collateral to the AccountCollateralsMap
			if current_balance == FixedI128::zero() {
				Self::add_collateral(account_id, collateral_id);
			} else if current_balance.is_negative() {
				let absolute_amount = min(-current_balance, amount);

				Self::deposit_event(Event::UserBalanceDeficit {
					trading_account,
					collateral_id,
					amount: absolute_amount,
					block_number,
				});

				// Add amount to Default InsuranceFund
				if let Some(insurance_fund) = DefaultInsuranceFund::<T>::get() {
					let current_insurance_fund_balance =
						InsuranceFundBalances::<T>::get(insurance_fund, collateral_id);

					// Increment the local balance of insurance fund
					InsuranceFundBalances::<T>::set(
						insurance_fund,
						collateral_id,
						current_insurance_fund_balance + absolute_amount,
					);
				}
			}

			let new_balance: FixedI128 = amount + current_balance;
			// Update the balance
			BalancesMap::<T>::set(account_id, collateral_id, new_balance);

			// Get the user account
			let account = AccountMap::<T>::get(&account_id).unwrap().to_trading_account_minimal();

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

		fn handle_fee_split(
			account_id: U256,
			collateral_id: u128,
			market_id: u128,
			amount: FixedI128,
			fee_share_amount: FixedI128,
		) {
			// Get the insurance fund and fee split details
			let collateral_asset = T::AssetPallet::get_asset(collateral_id).unwrap();
			let collateral_token_decimal = collateral_asset.decimals;

			let (insurance_fund, fee_split) = Self::get_fee_split_details(market_id);
			let current_insurance_fund_balance =
				InsuranceFundBalances::<T>::get(insurance_fund, collateral_id);
			let amount_after_fee_share = amount - fee_share_amount;
			let revenue_amount = (amount_after_fee_share * fee_split)
				.round_to_precision(collateral_token_decimal.into());
			let fee_amount = amount_after_fee_share - revenue_amount;

			// Increment the local balance of insurance fund
			InsuranceFundBalances::<T>::set(
				insurance_fund,
				collateral_id,
				current_insurance_fund_balance + fee_amount,
			);

			// Emit the event to be picked up by the Synchronizer
			Self::deposit_event(Event::UserBalanceChangeV2 {
				trading_account: AccountMap::<T>::get(&account_id)
					.unwrap()
					.to_trading_account_minimal(),
				market_id,
				amount,
				revenue_amount,
				fee_share_amount,
				modify_type: FundModifyType::Decrease,
				reason: BalanceChangeReason::Fee.into(),
				block_number: <frame_system::Pallet<T>>::block_number(),
			});
		}

		fn handle_insurance_fund_update(
			collateral_id: u128,
			market_id: u128,
			amount: FixedI128,
			modify_type: FundModifyType,
		) {
			// Get the insurance fund and update the value
			let collateral_asset = T::AssetPallet::get_asset(collateral_id).unwrap();
			let collateral_token_decimal = collateral_asset.decimals;
			let rounded_amount = amount.round_to_precision(collateral_token_decimal.into());

			let (insurance_fund, _) = Self::get_fee_split_details(market_id);
			let current_insurance_fund_balance =
				InsuranceFundBalances::<T>::get(insurance_fund, collateral_id);

			match modify_type {
				FundModifyType::Increase => InsuranceFundBalances::<T>::set(
					insurance_fund,
					collateral_id,
					current_insurance_fund_balance + rounded_amount,
				),
				FundModifyType::Decrease => InsuranceFundBalances::<T>::set(
					insurance_fund,
					collateral_id,
					current_insurance_fund_balance - rounded_amount,
				),
			}

			Self::deposit_event(Event::InsuranceFundChangeV2 {
				market_id,
				amount: rounded_amount,
				modify_type,
				block_number: <frame_system::Pallet<T>>::block_number(),
			});
		}

		fn get_account_list(start_index: u128, end_index: u128) -> Vec<U256> {
			let mut account_list = Vec::<U256>::new();
			let accounts_count = AccountsCount::<T>::get();
			for index in start_index..end_index {
				if (start_index > end_index) ||
					(index >= accounts_count) ||
					(start_index >= accounts_count)
				{
					break
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
					reason: BalanceChangeReason::Deposit.into(),
					previous_balance,
					new_balance,
					block_number: <frame_system::Pallet<T>>::block_number(),
				});
			}

			Ok(())
		}

		fn get_accounts_count() -> u128 {
			AccountsCount::<T>::get()
		}

		fn get_collaterals_of_user(account_id: U256) -> Vec<u128> {
			AccountCollateralsMap::<T>::get(account_id)
		}

		fn get_amount_to_withdraw(account_id: U256, collateral_id: u128) -> FixedI128 {
			Self::calculate_amount_to_withdraw(account_id, collateral_id)
		}

		fn update_and_get_user_and_master_volume(
			account_id: U256,
			market_id: u128,
			new_volume: FixedI128,
		) -> Result<(FixedI128, FixedI128), Self::VolumeError> {
			// Get the corresponsing monetary address of the user
			let user_monetary_address = AccountMap::<T>::get(account_id)
				.ok_or(Error::<T>::AccountDoesNotExist)?
				.account_address;

			// Gets the 30 day volume for the user
			let user_30_day_volume = Self::update_and_get_cumulative_volume(
				user_monetary_address,
				market_id,
				new_volume,
				VolumeType::UserVolume,
			)?;

			// If monetary address has a master account, add volume as part of master account
			let mut master_30_day_volume = FixedI128::zero();
			// Retrieve referral details
			if let Some(referral_details) = MasterAccountMap::<T>::get(user_monetary_address) {
				// Update and get cumulative volume if referral details exist
				let master_volume = Self::update_and_get_cumulative_volume(
					referral_details.master_account_address,
					market_id,
					new_volume,
					VolumeType::MasterVolume,
				)?;

				master_30_day_volume = master_volume;
			}

			Ok((user_30_day_volume, master_30_day_volume))
		}

		// This function updates the 31 day volume vector (present day in index 0 and last 30 days'
		// volume) and returns the current last 30 days volume not including the present day volume
		// This function is meant to be called only during trade execution and hence volume vector
		// is updated only during trade execution
		fn update_and_get_cumulative_volume(
			monetary_account_address: U256,
			market_id: u128,
			new_volume: FixedI128,
			volume_update_type: VolumeType,
		) -> Result<FixedI128, Self::VolumeError> {
			// find collateral id corresponding to the given market id
			// return error if market not found
			let collateral_id = T::MarketPallet::get_market(market_id)
				.ok_or(Error::<T>::MarketDoesNotExist)?
				.asset_collateral;

			let current_timestamp = T::TimeProvider::now().as_secs();
			if let Some(vol31) = match volume_update_type {
				VolumeType::UserVolume =>
					MonetaryAccountVolumeMap::<T>::get(monetary_account_address, collateral_id),
				VolumeType::MasterVolume =>
					MasterAccountVolumeMap::<T>::get(monetary_account_address, collateral_id),
			} {
				// we are bound to find last tx timestamp if volume vector was found, hence we
				// can directly unwrap
				let last_tx_timestamp = match volume_update_type {
					VolumeType::UserVolume => MonetaryAccountLastTxTimestamp::<T>::get(
						monetary_account_address,
						collateral_id,
					)
					.unwrap(),
					VolumeType::MasterVolume => MasterAccountLastTxTimestamp::<T>::get(
						monetary_account_address,
						collateral_id,
					)
					.unwrap(),
				};

				let day_diff = get_day_diff(last_tx_timestamp, current_timestamp);

				// Compute new volume vector, shifting the volumes, if there is a day diff
				// Also find last 30 days volume after computing the new volume vector
				let (updated_volume, last_30day_volume) =
					shift_and_recompute(&vol31, new_volume, day_diff);
				match volume_update_type {
					VolumeType::UserVolume => {
						MonetaryAccountVolumeMap::<T>::set(
							monetary_account_address,
							collateral_id,
							Some(updated_volume),
						);

						// update current tx timestamp
						MonetaryAccountLastTxTimestamp::<T>::set(
							monetary_account_address,
							collateral_id,
							Some(current_timestamp),
						);
					},
					VolumeType::MasterVolume => {
						MasterAccountVolumeMap::<T>::set(
							monetary_account_address,
							collateral_id,
							Some(updated_volume),
						);

						// update current tx timestamp
						MasterAccountLastTxTimestamp::<T>::set(
							monetary_account_address,
							collateral_id,
							Some(current_timestamp),
						);
					},
				};

				return Ok(last_30day_volume)
			} else {
				// Here the updated_volume vector will be all 0s except 1st element which will
				// store the new_volume
				let (updated_volume, last_30day_volume) =
					shift_and_recompute(&Vec::from([FixedI128::from_inner(0); 31]), new_volume, 31);
				match volume_update_type {
					VolumeType::UserVolume => {
						MonetaryAccountVolumeMap::<T>::set(
							monetary_account_address,
							collateral_id,
							Some(updated_volume),
						);

						// update current tx timestamp
						MonetaryAccountLastTxTimestamp::<T>::set(
							monetary_account_address,
							collateral_id,
							Some(current_timestamp),
						);
					},
					VolumeType::MasterVolume => {
						MasterAccountVolumeMap::<T>::set(
							monetary_account_address,
							collateral_id,
							Some(updated_volume),
						);

						// update current tx timestamp
						MasterAccountLastTxTimestamp::<T>::set(
							monetary_account_address,
							collateral_id,
							Some(current_timestamp),
						);
					},
				};

				return Ok(last_30day_volume)
			}
		}

		fn get_30day_user_volume(
			account_id: U256,
			market_id: u128,
		) -> Result<FixedI128, Self::VolumeError> {
			let monetary_account_address = AccountMap::<T>::get(account_id)
				.ok_or(Error::<T>::AccountDoesNotExist)?
				.account_address;

			Self::get_30day_volume(monetary_account_address, market_id, VolumeType::UserVolume)
		}

		fn get_30day_master_volume(
			monetary_account_address: U256,
			market_id: u128,
		) -> Result<FixedI128, Self::VolumeError> {
			Self::get_30day_volume(monetary_account_address, market_id, VolumeType::MasterVolume)
		}

		// This is a read-only function that returns the last 30 days volume (not including the
		// current day) The volume vector is updated only when update_and_get_cumulative_volume() is
		// called during execution of trade
		fn get_30day_volume(
			monetary_account_address: U256,
			market_id: u128,
			volume_type: VolumeType,
		) -> Result<FixedI128, Self::VolumeError> {
			// find collateral id corresponding to the market id given
			// return error if market not found
			let collateral_id = T::MarketPallet::get_market(market_id)
				.ok_or(Error::<T>::MarketDoesNotExist)?
				.asset_collateral;

			let current_timestamp = T::TimeProvider::now().as_secs();
			if let Some(vol31) = match volume_type {
				VolumeType::UserVolume =>
					MonetaryAccountVolumeMap::<T>::get(monetary_account_address, collateral_id),
				VolumeType::MasterVolume =>
					MasterAccountVolumeMap::<T>::get(monetary_account_address, collateral_id),
			} {
				// we are bound to find last tx timestamp if volume vector was found, hence we
				// can directly unwrap
				let last_tx_timestamp = match volume_type {
					VolumeType::UserVolume => MonetaryAccountLastTxTimestamp::<T>::get(
						monetary_account_address,
						collateral_id,
					)
					.unwrap(),
					VolumeType::MasterVolume => MasterAccountLastTxTimestamp::<T>::get(
						monetary_account_address,
						collateral_id,
					)
					.unwrap(),
				};

				let day_diff = get_day_diff(last_tx_timestamp, current_timestamp);

				// we can ignore the updated volume vector since this function is meant to be
				// used in read-only calls
				let (_, last_30day_volume) =
					shift_and_recompute(&vol31, FixedI128::from_inner(0), day_diff);
				return Ok(last_30day_volume)
			} else {
				// if no volume was found then no trade has happend in last 30 days, return 0
				return Ok(FixedI128::from_inner(0))
			}
		}

		fn add_referral_internal(
			referral_account_address: U256,
			referral_details: ReferralDetails,
			referral_code: U256,
		) -> bool {
			let existing_master = MasterAccountMap::<T>::get(referral_account_address);
			// Referral account can belong to only one master account
			if existing_master.is_some() && referral_details.fee_discount >= FixedI128::zero() {
				return false;
			}

			MasterAccountMap::<T>::insert(referral_account_address, referral_details);
			let referrals_count =
				ReferralsCountMap::<T>::get(referral_details.master_account_address);
			ReferralAccountsMap::<T>::set(
				(referral_details.master_account_address, referrals_count),
				referral_account_address,
			);
			ReferralsCountMap::<T>::set(
				referral_details.master_account_address,
				referrals_count + 1,
			);

			Self::deposit_event(Event::ReferralDetailsAdded {
				master_account_address: referral_details.master_account_address,
				referral_account_address,
				fee_discount: referral_details.fee_discount,
				referral_code,
			});

			true
		}

		fn update_master_account_level_internal(master_account_address: U256, level: u8) {
			Self::modify_master_account_level(master_account_address, level);
		}

		fn update_insurance_fund_balance_internal(
			insurance_fund: U256,
			collateral_id: u128,
			amount: FixedI128,
		) {
			let current_balance = InsuranceFundBalances::<T>::get(insurance_fund, collateral_id);
			InsuranceFundBalances::<T>::set(insurance_fund, collateral_id, current_balance + amount)
		}

		fn get_fee_discount(trading_account_id: U256) -> FixedI128 {
			let trading_account = AccountMap::<T>::get(trading_account_id);
			// Here, unwrap will not lead to any error becuase, we are checking
			// account existence in trading flow before calling this function
			let trading_account_details = trading_account.unwrap();
			let referral = MasterAccountMap::<T>::get(trading_account_details.account_address);
			// Check if current account has Master
			// If yes, return discount else return zero
			if referral.is_some() {
				let referral_details = referral.unwrap();
				return referral_details.fee_discount;
			} else {
				return FixedI128::zero();
			}
		}

		fn get_account_address_and_referral_details(account_id: U256) -> Option<ReferralDetails> {
			// This unwrap won't fail because we are checking whether this is a registered user
			// prior to this
			let account_address = AccountMap::<T>::get(account_id).unwrap().account_address;
			let referral_details = MasterAccountMap::<T>::get(account_address);
			if let Some(details) = referral_details {
				Some(details)
			} else {
				None
			}
		}

		fn get_master_account_level(account_address: U256) -> u8 {
			MasterAccountLevel::<T>::get(account_address)
		}

		fn update_master_fee_share(
			account_address: U256,
			collateral_id: u128,
			current_fee_share: FixedI128,
		) {
			let new_fee_share =
				MasterAccountFeeShare::<T>::get(account_address, collateral_id) + current_fee_share;
			MasterAccountFeeShare::<T>::set(account_address, collateral_id, new_fee_share);
		}

		fn get_fee_split_details(market_id: u128) -> (U256, FixedI128) {
			match MarketToFeeSplitMap::<T>::get(market_id) {
				Some((insurance_fund, fee_split)) => (insurance_fund, fee_split),
				None => match DefaultInsuranceFund::<T>::get() {
					Some(insurance_fund) => (insurance_fund, FixedI128::zero()),
					None => panic!("No default insurance fund set"),
				},
			}
		}
	}
}
