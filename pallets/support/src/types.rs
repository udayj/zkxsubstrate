
use codec::{Encode, Decode};
use sp_runtime::{
    RuntimeDebug,
};
use scale_info::TypeInfo;
use frame_support::pallet_prelude::*;

#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
pub struct TradingAccount {
    pub account_id: [u8;32]
}