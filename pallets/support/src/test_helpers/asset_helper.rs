use crate::types::Asset;
use primitive_types::U256;

pub fn eth() -> Asset {
	Asset {
		id: 0x4554480A,
		version: 1,
		short_name: U256::from("0x457468657265756D"),
		is_tradable: true,
		is_collateral: false,
		l2_address: U256::from(100),
		decimals: 18,
	}
}

pub fn btc() -> Asset {
	Asset {
		id: 0x425443,
		version: 1,
		short_name: U256::from("0x426974636F696E"),
		is_tradable: true,
		is_collateral: false,
		l2_address: U256::from(103),
		decimals: 6,
	}
}

pub fn usdc() -> Asset {
	Asset {
		id: 0x555344430A0A,
		version: 1,
		short_name: U256::from("0x55534420436972636C65"),
		is_tradable: false,
		is_collateral: true,
		l2_address: U256::from(101),
		decimals: 6,
	}
}

pub fn link() -> Asset {
	Asset {
		id: 0x4C494E4B,
		version: 1,
		short_name: U256::from("0x436861696E6C696E6B"),
		is_tradable: true,
		is_collateral: false,
		l2_address: U256::from(102),
		decimals: 6,
	}
}

pub fn usdt() -> Asset {
	Asset {
		id: 0x555344540A0A0A,
		version: 1,
		short_name: U256::from("0x54657468657220555344"),
		is_tradable: false,
		is_collateral: true,
		l2_address: U256::from(105),
		decimals: 6,
	}
}
