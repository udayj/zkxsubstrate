use crate::types::Market;
use sp_arithmetic::fixed_point::FixedI128;

impl Market {
	pub fn set_id(self: Market, id: u128) -> Market {
		let mut market = self;
		market.id = id;

		market
	}

	pub fn set_is_tradable(self: Market, is_tradable: bool) -> Market {
		let mut market = self;
		market.is_tradable = is_tradable;

		market
	}

	pub fn set_asset(self: Market, asset: u128) -> Market {
		let mut market = self;
		market.asset = asset;

		market
	}

	pub fn set_asset_collateral(self: Market, asset_collateral: u128) -> Market {
		let mut market = self;
		market.asset_collateral = asset_collateral;

		market
	}

	pub fn set_maximum_leverage(self: Market, maximum_leverage: FixedI128) -> Market {
		let mut market = self;
		market.maximum_leverage = maximum_leverage;

		market
	}

	pub fn set_minimum_leverage(self: Market, minimum_leverage: FixedI128) -> Market {
		let mut market = self;
		market.minimum_leverage = minimum_leverage;

		market
	}

	pub fn set_currently_allowed_leverage(
		self: Market,
		currently_allowed_leverage: FixedI128,
	) -> Market {
		let mut market = self;
		market.currently_allowed_leverage = currently_allowed_leverage;

		market
	}
}

pub fn eth_usdc() -> Market {
	Market {
		id: 1,
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
	}
}

pub fn link_usdc() -> Market {
	Market {
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
	}
}
