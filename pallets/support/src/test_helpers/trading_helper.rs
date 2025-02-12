use crate::{
	traits::{FieldElementExt, Hashable},
	types::{BaseFee, Direction, HashType, Order, OrderType, Side, SignatureInfo, TimeInForce},
};
use frame_support::dispatch::Vec;
use primitive_types::U256;
use sp_arithmetic::fixed_point::FixedI128;
use starknet_crypto::{sign, FieldElement};

use super::btc_usdc;

pub fn setup_fee() -> (Vec<BaseFee>, Vec<BaseFee>) {
	// TODO(merkle-groot): Using manual pushing because vec! has some issues in support pallet
	// fee tiers
	let mut fee_tiers = Vec::<u8>::new();
	fee_tiers.push(1_u8);
	fee_tiers.push(2_u8);
	fee_tiers.push(3_u8);

	// fee details
	let mut fee_details_maker = Vec::<BaseFee>::new();
	let base_fee1 = BaseFee { volume: 0.into(), fee: FixedI128::from_inner(20000000000000000) };
	fee_details_maker.push(base_fee1);
	let base_fee2 = BaseFee { volume: 1000.into(), fee: FixedI128::from_inner(15000000000000000) };
	fee_details_maker.push(base_fee2);
	let base_fee3 = BaseFee { volume: 5000.into(), fee: FixedI128::from_inner(10000000000000000) };
	fee_details_maker.push(base_fee3);

	let mut fee_details_taker: Vec<BaseFee> = Vec::new();
	let base_fee1 = BaseFee { volume: 0.into(), fee: FixedI128::from_inner(50000000000000000) };
	let base_fee2 = BaseFee { volume: 1000.into(), fee: FixedI128::from_inner(40000000000000000) };
	let base_fee3 = BaseFee { volume: 5000.into(), fee: FixedI128::from_inner(35000000000000000) };
	fee_details_taker.push(base_fee1);
	fee_details_taker.push(base_fee2);
	fee_details_taker.push(base_fee3);

	(fee_details_maker, fee_details_taker)
}

impl Order {
	pub fn new(order_id: U256, account_id: U256) -> Order {
		Order {
			account_id,
			order_id,
			market_id: btc_usdc().market.id,
			order_type: OrderType::Limit,
			direction: Direction::Long,
			side: Side::Buy,
			price: 100.into(),
			size: 1.into(),
			leverage: 1.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			signature_info: SignatureInfo {
				liquidator_pub_key: U256::zero(),
				hash_type: HashType::Pedersen,
				sig_r: U256::zero(),
				sig_s: U256::zero(),
			},
			timestamp: 1699940278000,
		}
	}

	pub fn set_account_id(self: Order, account_id: U256) -> Order {
		Order { account_id, ..self }
	}

	pub fn set_order_id(self: Order, order_id: U256) -> Order {
		Order { order_id, ..self }
	}

	pub fn set_market_id(self: Order, market_id: u128) -> Order {
		Order { market_id, ..self }
	}

	pub fn set_order_type(self: Order, order_type: OrderType) -> Order {
		Order { order_type, ..self }
	}

	pub fn set_direction(self: Order, direction: Direction) -> Order {
		Order { direction, ..self }
	}

	pub fn set_side(self: Order, side: Side) -> Order {
		Order { side, ..self }
	}

	pub fn set_price(self: Order, price: FixedI128) -> Order {
		Order { price, ..self }
	}

	pub fn set_size(self: Order, size: FixedI128) -> Order {
		Order { size, ..self }
	}

	pub fn set_leverage(self: Order, leverage: FixedI128) -> Order {
		Order { leverage, ..self }
	}

	pub fn set_slippage(self: Order, slippage: FixedI128) -> Order {
		Order { slippage, ..self }
	}

	pub fn set_post_only(self: Order, post_only: bool) -> Order {
		Order { post_only, ..self }
	}

	pub fn set_time_in_force(self: Order, time_in_force: TimeInForce) -> Order {
		Order { time_in_force, ..self }
	}

	pub fn set_timestamp(self: Order, timestamp: u64) -> Order {
		Order { timestamp, ..self }
	}

	pub fn sign_order(self: Order, private_key: FieldElement) -> Order {
		let order_hash = self.hash(&self.signature_info.hash_type).unwrap();
		let signature = sign(&private_key, &order_hash, &FieldElement::ONE).unwrap();

		let sig_r = signature.r.to_u256();
		let sig_s = signature.s.to_u256();
		let signature_info = SignatureInfo { sig_r, sig_s, ..self.signature_info };
		Order { signature_info, ..self }
	}

	pub fn sign_order_liquidator(
		self: Order,
		private_key: FieldElement,
		liquidator_pub_key: U256,
	) -> Order {
		let order_hash = self.hash(&self.signature_info.hash_type).unwrap();
		let signature = sign(&private_key, &order_hash, &FieldElement::ONE).unwrap();

		let sig_r = signature.r.to_u256();
		let sig_s = signature.s.to_u256();
		let signature_info =
			SignatureInfo { sig_r, sig_s, liquidator_pub_key, ..self.signature_info };
		Order { signature_info, ..self }
	}
}
