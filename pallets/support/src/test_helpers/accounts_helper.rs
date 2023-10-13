use crate::helpers::pedersen_hash_multiple;
use crate::traits::{FieldElementExt, FixedI128Ext, U256Ext};
use crate::types::{convert_to_u128_pair, HashType, TradingAccountMinimal, WithdrawalRequest};
use primitive_types::U256;
use sp_arithmetic::fixed_point::FixedI128;
use sp_io::hashing::blake2_256;
use starknet_crypto::{sign, FieldElement};
use starknet_ff::FromByteSliceError;
type ConversionError = FromByteSliceError;

pub fn create_withdrawal_request(
	account_id: U256,
	collateral_id: u128,
	amount: FixedI128,
	private_key: FieldElement,
) -> Result<WithdrawalRequest, ConversionError> {
	let (account_id_low, account_id_high) = convert_to_u128_pair(account_id)?;
	let elements = vec![
		account_id_low,
		account_id_high,
		FieldElement::from(collateral_id),
		amount.to_u256().try_to_felt()?,
	];

	let msg_hash = pedersen_hash_multiple(&elements);

	// Get the signature
	let signature = sign(&private_key, &msg_hash, &FieldElement::ONE).unwrap();

	Ok(WithdrawalRequest {
		account_id,
		collateral_id,
		amount,
		sig_r: signature.r.to_u256(),
		sig_s: signature.s.to_u256(),
		hash_type: HashType::Pedersen,
	})
}

pub fn get_private_key(pub_key: U256) -> FieldElement {
	if pub_key == alice().pub_key {
		FieldElement::from(12345_u128)
	} else if pub_key == bob().pub_key {
		FieldElement::from(12346_u128)
	} else if pub_key == charlie().pub_key {
		FieldElement::from(12347_u128)
	} else if pub_key == dave().pub_key {
		FieldElement::from(12348_u128)
	} else if pub_key == eduard().pub_key {
		FieldElement::from(12349_u128)
	} else {
		FieldElement::from(0_u128)
	}
}

pub fn get_trading_account_id(trading_account: TradingAccountMinimal) -> U256 {
	let mut result: [u8; 33] = [0; 33];
	trading_account.account_address.to_little_endian(&mut result[0..32]);
	result[32] = trading_account.index;

	blake2_256(&result).into()
}

pub fn alice() -> TradingAccountMinimal {
	TradingAccountMinimal {
		account_address: U256::from(100_u8),
		index: 0,
		pub_key: U256::from_dec_str(
			"1628448741648245036800002906075225705100596136133912895015035902954123957052",
		)
		.unwrap(),
	}
}

pub fn bob() -> TradingAccountMinimal {
	TradingAccountMinimal {
		account_address: U256::from(101_u8),
		index: 0,
		pub_key: U256::from_dec_str(
			"2734587570975953215033319696922164262260826928445675130094490350860110775927",
		)
		.unwrap(),
	}
}

pub fn charlie() -> TradingAccountMinimal {
	TradingAccountMinimal {
		account_address: U256::from(102_u8),
		index: 0,
		pub_key: U256::from_dec_str(
			"2457376002264611280816655453925405884371013933241232222259054612596603485629",
		)
		.unwrap(),
	}
}

pub fn dave() -> TradingAccountMinimal {
	TradingAccountMinimal {
		account_address: U256::from(103_u8),
		index: 0,
		pub_key: U256::from_dec_str(
			"297021124508995887059365693034777910037712736776962756431504561970877219904",
		)
		.unwrap(),
	}
}

pub fn eduard() -> TradingAccountMinimal {
	TradingAccountMinimal {
		account_address: U256::from(104_u8),
		index: 0,
		pub_key: U256::from_dec_str(
			"1973230609706632603859995910093337519395409734785764258434843072841781303122",
		)
		.unwrap(),
	}
}
