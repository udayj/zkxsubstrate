use frame_support::dispatch::Vec;
use pallet_support::types::{BaseFee, BaseFeeAggregate, FeeShareDetails};
use sp_arithmetic::fixed_point::FixedI128;

pub fn get_usdc_aggregate_fees() -> BaseFeeAggregate {
	BaseFeeAggregate {
		maker_buy: get_usdc_maker_open_fees(),
		maker_sell: get_usdc_maker_close_fees(),
		taker_buy: get_usdc_taker_open_fees(),
		taker_sell: get_usdc_taker_close_fees(),
	}
}

pub fn get_usdc_fee_shares() -> Vec<Vec<FeeShareDetails>> {
	vec![
		vec![
			FeeShareDetails {
				volume: FixedI128::from_u32(0),
				fee_share: FixedI128::from_float(0.0),
			},
			FeeShareDetails {
				volume: FixedI128::from_u32(200000),
				fee_share: FixedI128::from_float(0.05),
			},
			FeeShareDetails {
				volume: FixedI128::from_u32(5000000),
				fee_share: FixedI128::from_float(0.08),
			},
			FeeShareDetails {
				volume: FixedI128::from_u32(10000000),
				fee_share: FixedI128::from_float(0.1),
			},
			FeeShareDetails {
				volume: FixedI128::from_u32(25000000),
				fee_share: FixedI128::from_float(0.12),
			},
			FeeShareDetails {
				volume: FixedI128::from_u32(50000000),
				fee_share: FixedI128::from_float(0.15),
			},
		],
		vec![
			FeeShareDetails {
				volume: FixedI128::from_u32(0),
				fee_share: FixedI128::from_float(0.0),
			},
			FeeShareDetails {
				volume: FixedI128::from_u32(200000),
				fee_share: FixedI128::from_float(0.5),
			},
			FeeShareDetails {
				volume: FixedI128::from_u32(5000000),
				fee_share: FixedI128::from_float(0.5),
			},
			FeeShareDetails {
				volume: FixedI128::from_u32(10000000),
				fee_share: FixedI128::from_float(0.5),
			},
			FeeShareDetails {
				volume: FixedI128::from_u32(25000000),
				fee_share: FixedI128::from_float(0.5),
			},
			FeeShareDetails {
				volume: FixedI128::from_u32(50000000),
				fee_share: FixedI128::from_float(0.5),
			},
		],
	]
}

pub fn get_btc_usdc_aggregate_fees() -> BaseFeeAggregate {
	BaseFeeAggregate {
		maker_buy: get_btc_usdc_maker_open_fees(),
		maker_sell: get_btc_usdc_maker_close_fees(),
		taker_buy: get_btc_usdc_taker_open_fees(),
		taker_sell: get_btc_usdc_taker_close_fees(),
	}
}

pub fn get_0_aggregate_fees() -> BaseFeeAggregate {
	BaseFeeAggregate {
		maker_buy: vec![BaseFee {
			volume: FixedI128::from_u32(0),
			fee: FixedI128::from_float(0.0),
		}],
		maker_sell: vec![BaseFee {
			volume: FixedI128::from_u32(0),
			fee: FixedI128::from_float(0.0),
		}],
		taker_buy: vec![BaseFee {
			volume: FixedI128::from_u32(0),
			fee: FixedI128::from_float(0.0),
		}],
		taker_sell: vec![BaseFee {
			volume: FixedI128::from_u32(0),
			fee: FixedI128::from_float(0.0),
		}],
	}
}

pub fn get_invalid_aggregate_volume() -> BaseFeeAggregate {
	BaseFeeAggregate {
		maker_buy: vec![
			BaseFee { volume: FixedI128::from_u32(0), fee: FixedI128::from_float(5.0) },
			BaseFee { volume: FixedI128::from_u32(1000000), fee: FixedI128::from_float(2.0) },
			BaseFee { volume: FixedI128::from_u32(200000), fee: FixedI128::from_float(1.0) },
		],
		maker_sell: vec![
			BaseFee { volume: FixedI128::from_u32(0), fee: FixedI128::from_float(5.0) },
			BaseFee { volume: FixedI128::from_u32(1000000), fee: FixedI128::from_float(2.0) },
			BaseFee { volume: FixedI128::from_u32(2000000), fee: FixedI128::from_float(1.0) },
		],
		taker_buy: vec![BaseFee {
			volume: FixedI128::from_u32(0),
			fee: FixedI128::from_float(0.0),
		}],
		taker_sell: vec![BaseFee {
			volume: FixedI128::from_u32(0),
			fee: FixedI128::from_float(0.0),
		}],
	}
}

pub fn get_invalid_aggregate_fee() -> BaseFeeAggregate {
	BaseFeeAggregate {
		maker_buy: vec![
			BaseFee { volume: FixedI128::from_u32(0), fee: FixedI128::from_float(5.0) },
			BaseFee { volume: FixedI128::from_u32(1000000), fee: FixedI128::from_float(2.0) },
			BaseFee { volume: FixedI128::from_u32(2000000), fee: FixedI128::from_float(3.0) },
		],
		maker_sell: vec![
			BaseFee { volume: FixedI128::from_u32(0), fee: FixedI128::from_float(5.0) },
			BaseFee { volume: FixedI128::from_u32(1000000), fee: FixedI128::from_float(2.0) },
			BaseFee { volume: FixedI128::from_u32(2000000), fee: FixedI128::from_float(1.0) },
		],
		taker_buy: vec![BaseFee {
			volume: FixedI128::from_u32(0),
			fee: FixedI128::from_float(0.0),
		}],
		taker_sell: vec![BaseFee {
			volume: FixedI128::from_u32(0),
			fee: FixedI128::from_float(0.0),
		}],
	}
}

fn get_usdc_maker_open_fees() -> Vec<BaseFee> {
	vec![
		BaseFee { volume: FixedI128::from_u32(0), fee: FixedI128::from_float(0.02) },
		BaseFee { volume: FixedI128::from_u32(1000000), fee: FixedI128::from_float(0.015) },
		BaseFee { volume: FixedI128::from_u32(5000000), fee: FixedI128::from_float(0.010) },
		BaseFee { volume: FixedI128::from_u32(10000000), fee: FixedI128::from_float(0.005) },
		BaseFee { volume: FixedI128::from_u32(50000000), fee: FixedI128::from_float(0.0) },
	]
}

fn get_btc_usdc_maker_open_fees() -> Vec<BaseFee> {
	vec![
		BaseFee { volume: FixedI128::from_u32(0), fee: FixedI128::from_float(0.002) },
		BaseFee { volume: FixedI128::from_u32(10000), fee: FixedI128::from_float(0.001) },
		BaseFee { volume: FixedI128::from_u32(1000000), fee: FixedI128::from_float(0.0) },
	]
}

fn get_usdc_maker_close_fees() -> Vec<BaseFee> {
	vec![
		BaseFee { volume: FixedI128::from_u32(0), fee: FixedI128::from_float(0.02) },
		BaseFee { volume: FixedI128::from_u32(1000000), fee: FixedI128::from_float(0.015) },
		BaseFee { volume: FixedI128::from_u32(5000000), fee: FixedI128::from_float(0.010) },
		BaseFee { volume: FixedI128::from_u32(10000000), fee: FixedI128::from_float(0.005) },
		BaseFee { volume: FixedI128::from_u32(50000000), fee: FixedI128::from_float(0.0) },
	]
}

fn get_btc_usdc_maker_close_fees() -> Vec<BaseFee> {
	vec![
		BaseFee { volume: FixedI128::from_u32(0), fee: FixedI128::from_float(0.002) },
		BaseFee { volume: FixedI128::from_u32(10000), fee: FixedI128::from_float(0.001) },
		BaseFee { volume: FixedI128::from_u32(1000000), fee: FixedI128::from_float(0.0) },
	]
}

fn get_usdc_taker_open_fees() -> Vec<BaseFee> {
	vec![
		BaseFee { volume: FixedI128::from_u32(0), fee: FixedI128::from_float(0.050) },
		BaseFee { volume: FixedI128::from_u32(1000000), fee: FixedI128::from_float(0.040) },
		BaseFee { volume: FixedI128::from_u32(5000000), fee: FixedI128::from_float(0.035) },
		BaseFee { volume: FixedI128::from_u32(10000000), fee: FixedI128::from_float(0.030) },
		BaseFee { volume: FixedI128::from_u32(50000000), fee: FixedI128::from_float(0.025) },
		BaseFee { volume: FixedI128::from_u32(200000000), fee: FixedI128::from_float(0.020) },
	]
}

fn get_btc_usdc_taker_open_fees() -> Vec<BaseFee> {
	vec![
		BaseFee { volume: FixedI128::from_u32(0), fee: FixedI128::from_float(0.005) },
		BaseFee { volume: FixedI128::from_u32(10000), fee: FixedI128::from_float(0.0045) },
		BaseFee { volume: FixedI128::from_u32(1000000), fee: FixedI128::from_float(0.004) },
		BaseFee { volume: FixedI128::from_u32(5000000), fee: FixedI128::from_float(0.002) },
	]
}

fn get_usdc_taker_close_fees() -> Vec<BaseFee> {
	vec![
		BaseFee { volume: FixedI128::from_u32(0), fee: FixedI128::from_float(0.050) },
		BaseFee { volume: FixedI128::from_u32(1000000), fee: FixedI128::from_float(0.040) },
		BaseFee { volume: FixedI128::from_u32(5000000), fee: FixedI128::from_float(0.035) },
		BaseFee { volume: FixedI128::from_u32(10000000), fee: FixedI128::from_float(0.030) },
		BaseFee { volume: FixedI128::from_u32(50000000), fee: FixedI128::from_float(0.025) },
		BaseFee { volume: FixedI128::from_u32(200000000), fee: FixedI128::from_float(0.020) },
	]
}

fn get_btc_usdc_taker_close_fees() -> Vec<BaseFee> {
	vec![
		BaseFee { volume: FixedI128::from_u32(0), fee: FixedI128::from_float(0.005) },
		BaseFee { volume: FixedI128::from_u32(10000), fee: FixedI128::from_float(0.0045) },
		BaseFee { volume: FixedI128::from_u32(1000000), fee: FixedI128::from_float(0.004) },
		BaseFee { volume: FixedI128::from_u32(5000000), fee: FixedI128::from_float(0.002) },
	]
}
