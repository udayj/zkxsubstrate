use crate::types::{Asset, ExtendedAsset};
use primitive_types::U256;
use sp_runtime::BoundedVec;

impl ExtendedAsset {
	pub fn set_version(self: ExtendedAsset, version: u16) -> ExtendedAsset {
		let mut extended_asset = self;
		extended_asset.asset.version = version;

		extended_asset
	}

	pub fn set_is_tradable(self: ExtendedAsset, is_tradable: bool) -> ExtendedAsset {
		let mut extended_asset = self;
		extended_asset.asset.is_tradable = is_tradable;

		extended_asset
	}

	pub fn set_decimals(self: ExtendedAsset, decimals: u8) -> ExtendedAsset {
		let mut extended_asset = self;
		extended_asset.asset.decimals = decimals;

		extended_asset
	}
}

pub fn eth() -> ExtendedAsset {
	let mut metadata_url = BoundedVec::new();
	for &byte in b"https://x.com/zkxprotocol" {
		if let Err(_) = metadata_url.try_push(byte) {
			break; // If we reach the bound, stop adding elements.
		}
	}

	let mut icon_url = BoundedVec::new();
	for &byte in b"https://x.com/zkxprotocol/photo" {
		if let Err(_) = icon_url.try_push(byte) {
			break; // If we reach the bound, stop adding elements.
		}
	}

	ExtendedAsset {
		asset: Asset {
			id: 4543560,
			version: 1,
			short_name: U256::from("0x457468657265756D"),
			is_tradable: true,
			is_collateral: false,
			l2_address: U256::from(100),
			decimals: 18,
		},
		metadata_url: metadata_url.clone(),
		icon_url: icon_url.clone(),
	}
}

pub fn btc() -> ExtendedAsset {
	let mut metadata_url = BoundedVec::new();
	for &byte in b"https://x.com/zkxprotocol" {
		if let Err(_) = metadata_url.try_push(byte) {
			break; // If we reach the bound, stop adding elements.
		}
	}

	let mut icon_url = BoundedVec::new();
	for &byte in b"https://x.com/zkxprotocol/photo" {
		if let Err(_) = icon_url.try_push(byte) {
			break; // If we reach the bound, stop adding elements.
		}
	}

	ExtendedAsset {
		asset: Asset {
			id: 4346947,
			version: 1,
			short_name: U256::from("0x426974636F696E"),
			is_tradable: true,
			is_collateral: false,
			l2_address: U256::from(103),
			decimals: 6,
		},
		metadata_url: metadata_url.clone(),
		icon_url: icon_url.clone(),
	}
}

pub fn usdc() -> ExtendedAsset {
	let mut metadata_url = BoundedVec::new();
	for &byte in b"https://x.com/zkxprotocol" {
		if let Err(_) = metadata_url.try_push(byte) {
			break; // If we reach the bound, stop adding elements.
		}
	}

	let mut icon_url = BoundedVec::new();
	for &byte in b"https://x.com/zkxprotocol/photo" {
		if let Err(_) = icon_url.try_push(byte) {
			break; // If we reach the bound, stop adding elements.
		}
	}

	ExtendedAsset {
		asset: Asset {
			id: 1431520323,
			version: 1,
			short_name: U256::from("0x55534420436972636C65"),
			is_tradable: false,
			is_collateral: true,
			l2_address: U256::from(101),
			decimals: 6,
		},
		metadata_url: metadata_url.clone(),
		icon_url: icon_url.clone(),
	}
}

pub fn link() -> ExtendedAsset {
	let mut metadata_url = BoundedVec::new();
	for &byte in b"https://x.com/zkxprotocol" {
		if let Err(_) = metadata_url.try_push(byte) {
			break; // If we reach the bound, stop adding elements.
		}
	}

	let mut icon_url = BoundedVec::new();
	for &byte in b"https://x.com/zkxprotocol/photo" {
		if let Err(_) = icon_url.try_push(byte) {
			break; // If we reach the bound, stop adding elements.
		}
	}

	ExtendedAsset {
		asset: Asset {
			id: 1279872587,
			version: 1,
			short_name: U256::from("0x436861696E6C696E6B"),
			is_tradable: true,
			is_collateral: false,
			l2_address: U256::from(102),
			decimals: 6,
		},
		metadata_url: metadata_url.clone(),
		icon_url: icon_url.clone(),
	}
}

pub fn usdt() -> ExtendedAsset {
	let mut metadata_url = BoundedVec::new();
	for &byte in b"https://x.com/zkxprotocol" {
		if let Err(_) = metadata_url.try_push(byte) {
			break; // If we reach the bound, stop adding elements.
		}
	}

	let mut icon_url = BoundedVec::new();
	for &byte in b"https://x.com/zkxprotocol/photo" {
		if let Err(_) = icon_url.try_push(byte) {
			break; // If we reach the bound, stop adding elements.
		}
	}

	ExtendedAsset {
		asset: Asset {
			id: 1431520340,
			version: 1,
			short_name: U256::from("0x54657468657220555344"),
			is_tradable: false,
			is_collateral: true,
			l2_address: U256::from(105),
			decimals: 6,
		},
		metadata_url: metadata_url.clone(),
		icon_url: icon_url.clone(),
	}
}
