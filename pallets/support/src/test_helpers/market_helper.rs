use crate::types::Market;
use sp_arithmetic::fixed_point::FixedI128;

pub fn eth_usdc() -> Market {
	Market {
		id: 1,
		version: 1,
		asset: 0x4554480A,
		asset_collateral: 0x555344430A0A,
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
		asset: 0x4C494E4B,
		asset_collateral: 0x555344430A0A,
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
