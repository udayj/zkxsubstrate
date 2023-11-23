use crate as pallet_prices;
use frame_support::traits::{ConstU16, ConstU64};
use pallet_asset;
use pallet_market;
use pallet_timestamp;
use pallet_trading;
use pallet_trading_account;
use pallet_risk_management;
use pallet_trading_fees;
use sp_core::H256;
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		PricesModule: pallet_prices,
		MarketModule: pallet_market,
		Timestamp: pallet_timestamp,
		AssetModule: pallet_asset,
		TradingAccountPallet: pallet_trading_account,
		TradingModule: pallet_trading,
		RiskManagementModule: pallet_risk_management,
		TradingFeesModule: pallet_trading_fees
	}
);

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Nonce = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_asset::Config for Test {
	type RuntimeEvent = RuntimeEvent;
}

impl pallet_market::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type AssetPallet = AssetModule;
}

impl pallet_risk_management::Config for Test {
	type MarketPallet = MarketModule;
	type TradingPallet = TradingModule;
	type TradingAccountPallet = TradingAccountPallet;
	type PricesPallet = PricesModule;
}

impl pallet_trading_fees::Config for Test {
	type RuntimeEvent = RuntimeEvent;
}

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = ConstU64<5>;
	type WeightInfo = ();
}

impl pallet_trading::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type AssetPallet = AssetModule;
	type MarketPallet = MarketModule;
	type PricesPallet = PricesModule;
	type RiskManagementPallet = RiskManagementModule;
	type TradingAccountPallet = TradingAccountPallet;
	type TradingFeesPallet = TradingFeesModule;
	type TimeProvider = Timestamp;
}

impl pallet_prices::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type MarketPallet = MarketModule;
	type TimeProvider = Timestamp;
	type TradingAccountPallet = TradingAccountPallet;
}

impl pallet_trading_account::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type AssetPallet = AssetModule;
	type MarketPallet = MarketModule;
	type PricesPallet = PricesModule;
	type TradingPallet = TradingModule;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::<Test>::default().build_storage().unwrap().into()
}
