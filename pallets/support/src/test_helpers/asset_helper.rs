use crate::{
	traits::ChainConstants,
	types::{Asset, AssetAddress, ExtendedAsset},
};
use primitive_types::U256;
use sp_runtime::BoundedVec;

struct Chains;
impl ChainConstants for Chains {
	fn starknet_chain() -> U256 {
		U256::from(0x535441524b4e4554_u64)
	}

	fn zkx_sync_chain() -> U256 {
		U256::from(0x5a4b53594e43_u64)
	}
}

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

	let asset_addresses = BoundedVec::new();

	ExtendedAsset {
		asset: Asset {
			id: 4543560,
			version: 1,
			short_name: U256::from("0x457468657265756D"),
			is_tradable: true,
			is_collateral: false,
			decimals: 18,
		},
		asset_addresses,
		metadata_url,
	}
}

pub fn btc() -> ExtendedAsset {
	let mut metadata_url = BoundedVec::new();
	b"https://x.com/zkxprotocol"
		.iter()
		.for_each(|&byte| metadata_url.force_push(byte));

	let asset_addresses = BoundedVec::new();

	ExtendedAsset {
		asset: Asset {
			id: 4346947,
			version: 1,
			short_name: U256::from("0x426974636F696E"),
			is_tradable: true,
			is_collateral: false,
			decimals: 6,
		},
		asset_addresses,
		metadata_url,
	}
}

pub fn usdc() -> ExtendedAsset {
	let mut metadata_url = BoundedVec::new();
	b"https://x.com/zkxprotocol"
		.iter()
		.for_each(|&byte| metadata_url.force_push(byte));

	let mut asset_addresses = BoundedVec::new();
	asset_addresses
		.force_push(AssetAddress { chain: Chains::starknet_chain(), address: U256::from(200) });
	asset_addresses
		.force_push(AssetAddress { chain: Chains::zkx_sync_chain(), address: U256::from(201) });

	ExtendedAsset {
		asset: Asset {
			id: 1431520323,
			version: 1,
			short_name: U256::from("0x55534420436972636C65"),
			is_tradable: false,
			is_collateral: true,
			decimals: 6,
		},
		asset_addresses,
		metadata_url,
	}
}

pub fn link() -> ExtendedAsset {
	let mut metadata_url = BoundedVec::new();
	b"https://x.com/zkxprotocol"
		.iter()
		.for_each(|&byte| metadata_url.force_push(byte));

	let asset_addresses = BoundedVec::new();

	ExtendedAsset {
		asset: Asset {
			id: 1279872587,
			version: 1,
			short_name: U256::from("0x436861696E6C696E6B"),
			is_tradable: true,
			is_collateral: false,
			decimals: 6,
		},
		asset_addresses,
		metadata_url,
	}
}

pub fn usdt() -> ExtendedAsset {
	let mut metadata_url = BoundedVec::new();
	b"https://x.com/zkxprotocol"
		.iter()
		.for_each(|&byte| metadata_url.force_push(byte));

	let mut asset_addresses = BoundedVec::new();
	asset_addresses
		.force_push(AssetAddress { chain: Chains::starknet_chain(), address: U256::from(400) });
	asset_addresses
		.force_push(AssetAddress { chain: Chains::zkx_sync_chain(), address: U256::from(401) });

	ExtendedAsset {
		asset: Asset {
			id: 1431520340,
			version: 1,
			short_name: U256::from("0x54657468657220555344"),
			is_tradable: false,
			is_collateral: true,
			decimals: 6,
		},
		asset_addresses,
		metadata_url,
	}
}
