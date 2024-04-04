use crate as pallet_prices;
use frame_support::traits::{ConstU16, ConstU64};
use pallet_asset;
use pallet_market;
use pallet_prices::crypto::AuthId;
use pallet_risk_management;
use pallet_timestamp;
use pallet_trading;
use pallet_trading_account;
use pallet_trading_fees;
use sp_core::{sr25519::Signature, H256};
use sp_runtime::{
	testing::TestXt,
	traits::{BlakeTwo256, Extrinsic as ExtrinsicT, IdentifyAccount, IdentityLookup, Verify},
	BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;
type Extrinsic = TestXt<RuntimeCall, ()>;
type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		PricesModule: pallet_prices,
		MarketModule: pallet_market,
		Timestamp: pallet_timestamp,
		AssetModule: pallet_asset,
		RiskManagement: pallet_risk_management,
		Trading: pallet_trading,
		TradingFees: pallet_trading_fees,
		TradingAccounts: pallet_trading_account
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
	type AccountId = sp_core::sr25519::Public;
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
	type TradingPallet = Trading;
	type TradingAccountPallet = TradingAccounts;
	type PricesPallet = PricesModule;
}

impl pallet_trading_fees::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type AssetPallet = AssetModule;
	type MarketPallet = MarketModule;
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
	type RiskManagementPallet = RiskManagement;
	type TradingAccountPallet = TradingAccounts;
	type TradingFeesPallet = TradingFees;
	type TimeProvider = Timestamp;
	type AuthorityId = AuthId;
}

impl pallet_trading_account::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type AssetPallet = AssetModule;
	type MarketPallet = MarketModule;
	type PricesPallet = PricesModule;
	type TradingPallet = Trading;
	type TimeProvider = Timestamp;
}

impl pallet_prices::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type AssetPallet = AssetModule;
	type MarketPallet = MarketModule;
	type TimeProvider = Timestamp;
	type TradingAccountPallet = TradingAccounts;
	type TradingPallet = Trading;
	type AuthorityId = AuthId;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::<Test>::default().build_storage().unwrap().into()
}

impl frame_system::offchain::SigningTypes for Test {
	type Public = <Signature as Verify>::Signer;
	type Signature = Signature;
}

impl<LocalCall> frame_system::offchain::SendTransactionTypes<LocalCall> for Test
where
	RuntimeCall: From<LocalCall>,
{
	type OverarchingCall = RuntimeCall;
	type Extrinsic = Extrinsic;
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Test
where
	RuntimeCall: From<LocalCall>,
{
	fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
		call: RuntimeCall,
		_public: <Signature as Verify>::Signer,
		_account: AccountId,
		nonce: u64,
	) -> Option<(RuntimeCall, <Extrinsic as ExtrinsicT>::SignaturePayload)> {
		Some((call, (nonce, ())))
	}
}
