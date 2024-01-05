use crate::types::{ExtendedMarket, Market};
use sp_arithmetic::fixed_point::FixedI128;
use sp_runtime::BoundedVec;

impl ExtendedMarket {
	pub fn set_id(self: ExtendedMarket, id: u128) -> ExtendedMarket {
		let mut extended_market = self;
		extended_market.market.id = id;

		extended_market
	}

	pub fn set_is_tradable(self: ExtendedMarket, is_tradable: bool) -> ExtendedMarket {
		let mut extended_market = self;
		extended_market.market.is_tradable = is_tradable;

		extended_market
	}

	pub fn set_asset(self: ExtendedMarket, asset: u128) -> ExtendedMarket {
		let mut extended_market = self;
		extended_market.market.asset = asset;

		extended_market
	}

	pub fn set_asset_collateral(self: ExtendedMarket, asset_collateral: u128) -> ExtendedMarket {
		let mut extended_market = self;
		extended_market.market.asset_collateral = asset_collateral;

		extended_market
	}

	pub fn set_maximum_leverage(
		self: ExtendedMarket,
		maximum_leverage: FixedI128,
	) -> ExtendedMarket {
		let mut extended_market = self;
		extended_market.market.maximum_leverage = maximum_leverage;

		extended_market
	}

	pub fn set_minimum_leverage(
		self: ExtendedMarket,
		minimum_leverage: FixedI128,
	) -> ExtendedMarket {
		let mut extended_market = self;
		extended_market.market.minimum_leverage = minimum_leverage;

		extended_market
	}

	pub fn set_minimum_order_size(
		self: ExtendedMarket,
		minimum_order_size: FixedI128,
	) -> ExtendedMarket {
		let mut extended_market = self;
		extended_market.market.minimum_order_size = minimum_order_size;

		extended_market
	}

	pub fn set_currently_allowed_leverage(
		self: ExtendedMarket,
		currently_allowed_leverage: FixedI128,
	) -> ExtendedMarket {
		let mut extended_market = self;
		extended_market.market.currently_allowed_leverage = currently_allowed_leverage;

		extended_market
	}
}

pub fn eth_usdc() -> ExtendedMarket {
	let mut metadata_url = BoundedVec::new();
	for &byte in b"https://x.com/zkxprotocol" {
		if let Err(_) = metadata_url.try_push(byte) {
			break // If we reach the bound, stop adding elements.
		}
	}

	ExtendedMarket {
		market: Market {
			id: 3,
			version: 1,
			asset: 4543560,
			asset_collateral: 1431520323,
			is_tradable: true,
			is_archived: false,
			ttl: 3600,
			tick_size: 1.into(),
			tick_precision: 1,
			step_size: 1.into(),
			step_precision: 1,
			minimum_order_size: FixedI128::from_inner(100000000000000000),
			minimum_leverage: 1.into(),
			maximum_leverage: 10.into(),
			currently_allowed_leverage: 8.into(),
			maintenance_margin_fraction: FixedI128::from_inner(75000000000000000),
			initial_margin_fraction: 1.into(),
			incremental_initial_margin_fraction: 1.into(),
			incremental_position_size: 100.into(),
			baseline_position_size: 1000.into(),
			maximum_position_size: 10000.into(),
		},
		metadata_url: metadata_url.clone(),
	}
}

pub fn link_usdc() -> ExtendedMarket {
	let mut metadata_url = BoundedVec::new();
	for &byte in b"https://x.com/zkxprotocol" {
		if let Err(_) = metadata_url.try_push(byte) {
			break // If we reach the bound, stop adding elements.
		}
	}

	ExtendedMarket {
		market: Market {
			id: 2,
			version: 1,
			asset: 1279872587,
			asset_collateral: 1431520323,
			is_tradable: false,
			is_archived: false,
			ttl: 360,
			tick_size: 1.into(),
			tick_precision: 1,
			step_size: 1.into(),
			step_precision: 1,
			minimum_order_size: 1.into(),
			minimum_leverage: 1.into(),
			maximum_leverage: 10.into(),
			currently_allowed_leverage: 8.into(),
			maintenance_margin_fraction: 1.into(),
			initial_margin_fraction: 1.into(),
			incremental_initial_margin_fraction: 1.into(),
			incremental_position_size: 1.into(),
			baseline_position_size: 1.into(),
			maximum_position_size: 1.into(),
		},
		metadata_url: metadata_url.clone(),
	}
}

pub fn btc_usdc() -> ExtendedMarket {
	let mut metadata_url = BoundedVec::new();
	for &byte in b"https://x.com/zkxprotocol" {
		if let Err(_) = metadata_url.try_push(byte) {
			break // If we reach the bound, stop adding elements.
		}
	}

	ExtendedMarket {
		market: Market {
			id: 1,
			version: 1,
			asset: 4346947,
			asset_collateral: 1431520323,
			is_tradable: true,
			is_archived: false,
			ttl: 3600,
			tick_size: 1.into(),
			tick_precision: 1,
			step_size: 1.into(),
			step_precision: 1,
			minimum_order_size: 1.into(),
			minimum_leverage: 1.into(),
			maximum_leverage: 10.into(),
			currently_allowed_leverage: 8.into(),
			maintenance_margin_fraction: FixedI128::from_inner(75000000000000000),
			initial_margin_fraction: 1.into(),
			incremental_initial_margin_fraction: 1.into(),
			incremental_position_size: 1.into(),
			baseline_position_size: 1.into(),
			maximum_position_size: 1.into(),
		},
		metadata_url: metadata_url.clone(),
	}
}
