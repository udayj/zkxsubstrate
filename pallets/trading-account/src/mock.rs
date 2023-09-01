use crate as pallet_trading_account;
use frame_support::traits::{ConstU16, ConstU64};
use pallet_asset;
use pallet_timestamp;
use pallet_trading_fees;
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
		TradingAccountModule: pallet_trading_account,
		Timestamp: pallet_timestamp,
		Assets: pallet_asset,
		Markets: pallet_market,
		Trading: pallet_trading,
		MarketPrices: pallet_market_prices,
		TradingFees: pallet_trading_fees
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

impl pallet_trading_account::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type AssetPallet = Assets;
	type TradingPallet = Trading;
	type MarketPallet = Markets;
	type MarketPricesPallet = MarketPrices;
}

impl pallet_asset::Config for Test {
	type RuntimeEvent = RuntimeEvent;
}

impl pallet_market::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type AssetPallet = Assets;
}

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = ConstU64<5>;
	type WeightInfo = ();
}

impl pallet_market_prices::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type MarketPallet = Markets;
	type TimeProvider = Timestamp;
}

impl pallet_trading_fees::Config for Test {
	type RuntimeEvent = RuntimeEvent;
}

impl pallet_trading::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type MarketPallet = Markets;
	type TradingAccountPallet = TradingAccountModule;
	type TradingFeesPallet = TradingFees;
	type MarketPricesPallet = MarketPrices;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}
