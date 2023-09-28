use crate as trading;
use frame_support::traits::{ConstU16, ConstU64};
use pallet_asset;
use pallet_market;
use pallet_market_prices;
use pallet_risk_management;
use pallet_timestamp;
use pallet_trading_fees;
use pallet_zkx_trading_account;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system,
		Markets: pallet_market,
		MarketPrices: pallet_market_prices,
		Assets: pallet_asset,
		RiskManagement: pallet_risk_management,
		Timestamp: pallet_timestamp,
		TradingAccounts: pallet_zkx_trading_account,
		TradingFees: pallet_trading_fees,
		Trading: trading,
	}
);

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
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
	type AssetPallet = Assets;
}

impl pallet_market_prices::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type MarketPallet = Markets;
	type TimeProvider = Timestamp;
}

impl pallet_trading_fees::Config for Test {
	type RuntimeEvent = RuntimeEvent;
}

impl pallet_zkx_trading_account::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type AssetPallet = Assets;
	type MarketPallet = Markets;
	type MarketPricesPallet = MarketPrices;
	type TradingPallet = Trading;
}

impl pallet_risk_management::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type MarketPallet = Markets;
	type TradingPallet = Trading;
	type TradingAccountPallet = TradingAccounts;
}

impl trading::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type MarketPallet = Markets;
	type MarketPricesPallet = MarketPrices;
	type RiskManagementPallet = RiskManagement;
	type TradingAccountPallet = TradingAccounts;
	type TradingFeesPallet = TradingFees;
}

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = ConstU64<5>;
	type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}
