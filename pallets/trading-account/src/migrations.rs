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
		// It Should be the address returned by get_insurance_fund fn inside L2 contract
		let insurance_fund_address: U256 =
			U256::from("0x0578b12cd73ebca3e8edd00a959d5428ebde350a36f896e2a5c5b87b6e6b6caf");
		let collateral_id: u128 = 1431520323;
		let amount: FixedI128 = FixedI128::from_inner(1010010909636000000000000);

		let onchain_version = Pallet::<T>::on_chain_storage_version();

		if onchain_version < 1 {
			// Set the default insurance fund
			DefaultInsuranceFund::<T>::set(Some(insurance_fund_address));

			// Set the current balance of the above insurance fund
			InsuranceFundBalances::<T>::set(insurance_fund_address, collateral_id, amount);

			// Update the storage version
			StorageVersion::new(1).put::<Pallet<T>>();

			T::DbWeight::get().reads_writes(0, 2)
		} else {
			Weight::zero()
		}
	}
}
