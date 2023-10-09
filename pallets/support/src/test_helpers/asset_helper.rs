use crate::types::Asset;
use primitive_types::U256;

pub fn eth() -> Asset {
	Asset {
		id: 1163151370,
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
		id: 4346947,
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
		id: 93816115890698,
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
		id: 1279872587,
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
		id: 24016925953231370,
		version: 1,
		short_name: U256::from("0x54657468657220555344"),
		is_tradable: false,
		is_collateral: true,
		l2_address: U256::from(105),
		decimals: 6,
	}
}
