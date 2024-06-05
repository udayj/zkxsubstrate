use super::*;

pub mod migrations {
	use super::*;
	use frame_support::{
		traits::{Get, GetStorageVersion, StorageVersion},
		weights::Weight,
	};
	use primitive_types::U256;
	use sp_arithmetic::fixed_point::FixedI128;

	pub fn migrate_to_v2<T: Config>() -> Weight {
		let insurance_fund_address: U256 = U256::from(1);
		let collateral_id: u128 = 1431520323;
		let amount: FixedI128 = FixedI128::from_u32(1000000);

		let onchain_version = Pallet::<T>::on_chain_storage_version();

		if onchain_version < 2 {
			// Set the default insurance fund
			// It Should be the address returned by get_insurance_fund fn inside L2 contract
			DefaultInsuranceFund::<T>::set(Some(insurance_fund_address));

			// Set the current balance of the above insurance fund
			InsuranceFundBalances::<T>::set(insurance_fund_address, collateral_id, amount);

			// Update the storage version
			StorageVersion::new(2).put::<Pallet<T>>();

			T::DbWeight::get().reads_writes(2, 2)
		} else {
			Weight::zero()
		}
	}
}
