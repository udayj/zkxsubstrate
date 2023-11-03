use crate::mock::*;
use frame_support::assert_ok;
use primitive_types::U256;
use sp_arithmetic::FixedI128;
use zkx_support::{
	test_helpers::{
		accounts_helper::{
			alice, bob, charlie, dave, eduard, get_private_key, get_trading_account_id,
		},
		asset_helper::{btc, eth, link, usdc},
		market_helper::{btc_usdc, link_usdc},
	},
	types::{Direction, MultiplePrices, Order, OrderType, Position, Side},
};

fn setup() -> sp_io::TestExternalities {
	// Create a new test environment
	let mut env = new_test_ext();

	// Set the block number in the environment
	env.execute_with(|| {
		// Set the block number
		System::set_block_number(1);
		assert_ok!(Timestamp::set(None.into(), 100));

		// Set the assets in the system
		assert_ok!(Assets::replace_all_assets(
			RuntimeOrigin::signed(1),
			vec![eth(), usdc(), link(), btc()]
		));
		assert_ok!(Markets::replace_all_markets(
			RuntimeOrigin::signed(1),
			vec![btc_usdc(), link_usdc()]
		));

		// Add accounts to the system
		assert_ok!(TradingAccounts::add_accounts(
			RuntimeOrigin::signed(1),
			vec![alice(), bob(), charlie(), dave()]
		));

		// Add liquidator
		Trading::add_liquidator_signer(RuntimeOrigin::signed(1), eduard().pub_key)
			.expect("error while adding signer");
	});

	env
}

#[test]
// basic open trade with leverage
fn it_works_for_open_trade_with_leverage() {
	let mut env = setup();

	env.execute_with(|| {
		// Generate account_ids
		let alice_id: U256 = get_trading_account_id(alice());
		let bob_id: U256 = get_trading_account_id(bob());
		let charlie_id: U256 = get_trading_account_id(charlie());

		// market id
		let market_id = btc_usdc().market.id;

		// Create orders
		let alice_order = Order::new(201_u128, alice_id)
			.set_size(5.into())
			.set_leverage(5.into())
			.set_price(10000.into())
			.sign_order(get_private_key(alice().pub_key));

		let bob_order = Order::new(202_u128, bob_id)
			.set_size(5.into())
			.set_order_type(OrderType::Market)
			.set_direction(Direction::Short)
			.set_leverage(5.into())
			.set_price(10000.into())
			.sign_order(get_private_key(bob().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch id
			U256::from(1_u8),
			// size
			5.into(),
			// market
			market_id,
			// price
			10000.into(),
			// orders
			vec![alice_order.clone(), bob_order.clone()]
		));

		// Decrease the price of the asset
		let mut index_prices: Vec<MultiplePrices> = Vec::new();
		let index_price1 = MultiplePrices { market_id, price: 8000.into() };
		index_prices.push(index_price1);

		// Place Forced order for liquidation
		let charlie_order = Order::new(204_u128, charlie_id)
			.set_size(5.into())
			.set_price(8000.into())
			.sign_order(get_private_key(charlie().pub_key));

		let alice_forced_order = Order::new(203_u128, alice_id)
			.set_size(5.into())
			.set_price(8000.into())
			.set_order_type(OrderType::Forced)
			.set_direction(Direction::Long)
			.set_side(Side::Sell)
			.sign_order(get_private_key(eduard().pub_key));

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			// batch id
			U256::from(2_u8),
			// size
			5.into(),
			// market
			market_id,
			// price
			8000.into(),
			// orders
			vec![charlie_order, alice_forced_order]
		));
	});
}
