use crate::traits::{FieldElementExt, Hashable};
use crate::types::{TradingAccountMinimal, WithdrawalRequest};
use primitive_types::U256;
use sp_io::hashing::blake2_256;
use starknet_crypto::{sign, FieldElement};

pub fn sign_withdrawal_request(
	mut withdrawal_request: WithdrawalRequest,
	private_key: FieldElement,
) -> WithdrawalRequest {
	let withdrawal_request_hash = withdrawal_request.hash(&withdrawal_request.hash_type).unwrap();
	let signature = sign(&private_key, &withdrawal_request_hash, &FieldElement::ONE).unwrap();

	withdrawal_request.sig_r = signature.r.to_u256();
	withdrawal_request.sig_s = signature.s.to_u256();

	withdrawal_request
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
			"454932787469224290468444410084879070088819078827906347654495047407276534283",
		)
		.unwrap(),
	}
}

pub fn bob() -> TradingAccountMinimal {
	TradingAccountMinimal {
		account_address: U256::from(101_u8),
		index: 0,
		pub_key: U256::from_dec_str(
			"2101677845476848141002376837472833021659088026888369432434421980160153750090",
		)
		.unwrap(),
	}
}

pub fn charlie() -> TradingAccountMinimal {
	TradingAccountMinimal {
		account_address: U256::from(102_u8),
		index: 0,
		pub_key: U256::from_dec_str(
			"1927799101328918885926814969993421873905724180750168745093131010179897850144",
		)
		.unwrap(),
	}
}

pub fn dave() -> TradingAccountMinimal {
	TradingAccountMinimal {
		account_address: U256::from(103_u8),
		index: 0,
		pub_key: U256::from_dec_str(
			"824120678599933675767871867465569325984720238047137957464936400424120564339",
		)
		.unwrap(),
	}
}

// let user_pri_key_1: U256 = U256::from_dec_str(
//     "217039137810971208563823259722717297948702641410765313684702872265493782699",
// )
// .unwrap();

// let user_pri_key_2: U256 = U256::from_dec_str(
//     "2835524789612495000294332407161775540542356260492319813526822636942276039073",
// )
// .unwrap();

// let user_pri_key_3: U256 = U256::from_dec_str(
//     "3388506857955987752046415916181604993164423072000548640801744803879383940670",
// )
// .unwrap();

// let user_pri_key_4: U256 = U256::from_dec_str(
//     "84035867551811388210596922086133550045728262314839423570645036080104955628",
// )
// .unwrap();
