use crate::{mock::*, Event};
use frame_support::assert_ok;
use primitive_types::U256;
use sp_arithmetic::FixedI128;
use sp_io::hashing::blake2_256;
use starknet_crypto::{sign, verify, FieldElement};
use zkx_support::traits::{FieldElementExt, Hashable, U256Ext};
use zkx_support::types::{
	Asset, BalanceUpdate, Direction, HashType, LiquidatablePosition, Market, MarketPrice,
	MultipleMarketPrices, Order, OrderType, Position, Side, TimeInForce, TradingAccountWithoutId,
};

fn setup() -> (Vec<Market>, Vec<TradingAccountWithoutId>, Vec<U256>) {
	let ETH_ID: U256 = 4543560.into();
	let USDC_ID: U256 = 1431520323.into();
	let LINK_ID: U256 = 1279872587.into();
	let BTC_ID: U256 = 4346947.into();
	let name1: Vec<u8> = "ETH".into();
	let asset1: Asset = Asset {
		id: ETH_ID,
		name: name1.try_into().unwrap(),
		is_tradable: true,
		is_collateral: false,
		token_decimal: 18,
	};
	let name2: Vec<u8> = "USDC".into();
	let asset2: Asset = Asset {
		id: USDC_ID,
		name: name2.try_into().unwrap(),
		is_tradable: false,
		is_collateral: true,
		token_decimal: 6,
	};
	let name3: Vec<u8> = "LINK".into();
	let asset3: Asset = Asset {
		id: LINK_ID,
		name: name3.try_into().unwrap(),
		is_tradable: true,
		is_collateral: false,
		token_decimal: 6,
	};

	let assets: Vec<Asset> = vec![asset1.clone(), asset2.clone(), asset3.clone()];
	assert_ok!(Assets::replace_all_assets(RuntimeOrigin::signed(1), assets));

	let market1: Market = Market {
		id: 1.into(),
		asset: ETH_ID,
		asset_collateral: USDC_ID,
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
		incremental_position_size: 100.into(),
		baseline_position_size: 1000.into(),
		maximum_position_size: 10000.into(),
	};
	let market2: Market = Market {
		id: 2.into(),
		asset: LINK_ID,
		asset_collateral: USDC_ID,
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
		maintenance_margin_fraction: FixedI128::from_inner(75000000000000000),
		initial_margin_fraction: 1.into(),
		incremental_initial_margin_fraction: 1.into(),
		incremental_position_size: 100.into(),
		baseline_position_size: 1000.into(),
		maximum_position_size: 10000.into(),
	};

	let markets: Vec<Market> = vec![market1.clone(), market2.clone()];
	assert_ok!(Markets::replace_all_markets(RuntimeOrigin::signed(1), markets.clone()));

	let user_pub_key_1: U256 = U256::from_dec_str(
		"454932787469224290468444410084879070088819078827906347654495047407276534283",
	)
	.unwrap();
	let user_pri_key_1: U256 = U256::from_dec_str(
		"217039137810971208563823259722717297948702641410765313684702872265493782699",
	)
	.unwrap();
	let user_address_1: U256 = U256::from(100_u8);

	let user_pub_key_2: U256 = U256::from_dec_str(
		"2101677845476848141002376837472833021659088026888369432434421980160153750090",
	)
	.unwrap();
	let user_pri_key_2: U256 = U256::from_dec_str(
		"2835524789612495000294332407161775540542356260492319813526822636942276039073",
	)
	.unwrap();
	let user_address_2: U256 = U256::from(101_u8);

	let user_1 = TradingAccountWithoutId {
		account_address: user_address_1,
		index: 0,
		pub_key: user_pub_key_1,
	};
	let user_2 = TradingAccountWithoutId {
		account_address: user_address_2,
		index: 0,
		pub_key: user_pub_key_2,
	};
	let accounts: Vec<TradingAccountWithoutId> = vec![user_1, user_2];
	assert_ok!(TradingAccounts::add_accounts(RuntimeOrigin::signed(1), accounts.clone()));

	let private_keys: Vec<U256> = vec![user_pri_key_1, user_pri_key_2];

	(markets, accounts, private_keys)
}

fn get_trading_account_id(trading_accounts: Vec<TradingAccountWithoutId>, index: usize) -> U256 {
	let account_address = U256::from(trading_accounts[index].account_address);
	let mut account_array: [u8; 32] = [0; 32];
	account_address.to_little_endian(&mut account_array);

	let mut concatenated_bytes: Vec<u8> = account_array.to_vec();
	concatenated_bytes.push(trading_accounts.get(index).unwrap().index);
	let result: [u8; 33] = concatenated_bytes.try_into().unwrap();

	let trading_account_id: U256 = blake2_256(&result).into();
	trading_account_id
}

fn sign_order(order: Order, private_key: U256) -> Order {
	let order_hash = order.hash(&order.hash_type).unwrap();
	let private_key = private_key.try_to_felt().unwrap();
	let signature = sign(&private_key, &order_hash, &FieldElement::ONE).unwrap();

	let sig_r = signature.r.to_u256();
	let sig_s = signature.s.to_u256();
	Order { sig_r, sig_s, ..order }
}

#[test]
// basic open trade with leverage
fn it_works_for_open_trade_2() {
	new_test_ext().execute_with(|| {
		let order_id_1 = 200_u128;
		let order_id_2 = 201_u128;

		let (markets, accounts, private_keys) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		let account_id_1: U256 = get_trading_account_id(accounts.clone(), 0);
		let account_id_2: U256 = get_trading_account_id(accounts.clone(), 1);

		let order_1 = Order {
			account_id: account_id_1,
			order_id: order_id_1,
			market_id: markets[0].id,
			order_type: OrderType::Limit,
			direction: Direction::Long,
			side: Side::Buy,
			price: 10000.into(),
			size: 5.into(),
			leverage: 5.into(),
			slippage: 10.into(),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};
		let order_2 = Order {
			account_id: account_id_2,
			order_id: order_id_2,
			market_id: markets[0].id,
			order_type: OrderType::Market,
			direction: Direction::Short,
			side: Side::Buy,
			price: 10000.into(),
			size: 5.into(),
			leverage: 5.into(),
			slippage: 10.into(),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};

		let order_1 = sign_order(order_1, private_keys[0]);
		let order_2 = sign_order(order_2, private_keys[1]);
		let orders: Vec<Order> = vec![order_1, order_2];

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			U256::from(1_u8),
			5.into(),
			markets[0].id,
			10000.into(),
			orders
		));

		let position1 = Trading::positions(account_id_1, (markets[0].id, Direction::Long));
		let expected_position: Position = Position {
			avg_execution_price: 10000.into(),
			size: 5.into(),
			direction: Direction::Long,
			side: Side::Buy,
			margin_amount: 10000.into(),
			borrowed_amount: 40000.into(),
			leverage: 5.into(),
			realized_pnl: 0.into(),
		};
		assert_eq!(expected_position, position1);
		println!("{:?}", position1);

		let position2 = Trading::positions(account_id_2, (markets[0].id, Direction::Short));
		let expected_position: Position = Position {
			avg_execution_price: 10000.into(),
			size: 5.into(),
			direction: Direction::Short,
			side: Side::Buy,
			margin_amount: 10000.into(),
			borrowed_amount: 40000.into(),
			leverage: 5.into(),
			realized_pnl: 0.into(),
		};
		assert_eq!(expected_position, position2);
		println!("{:?}", position2);

		// As the price of an asset didn't change. Liquidatable or deleverabale position would be zero for account_id_1
		assert_ok!(RiskManagement::mark_under_collateralized_position(
			RuntimeOrigin::signed(1),
			account_id_1,
			markets[0].asset_collateral,
		));

		let liquidatable_position = Trading::deleveragable_or_liquidatable_position(
			account_id_1,
			markets[0].asset_collateral,
		);

		let expected_position: LiquidatablePosition = LiquidatablePosition {
			market_id: 0.into(),
			direction: Direction::Long,
			amount_to_be_sold: 0.into(),
			liquidatable: false,
		};
		assert_eq!(expected_position, liquidatable_position);
		println!("{:?}", liquidatable_position);

		// As the price of an asset didn't change. Liquidatable or deleverabale position would be zero for account_id_2
		assert_ok!(RiskManagement::mark_under_collateralized_position(
			RuntimeOrigin::signed(1),
			account_id_2,
			markets[0].asset_collateral,
		));

		let liquidatable_position = Trading::deleveragable_or_liquidatable_position(
			account_id_2,
			markets[0].asset_collateral,
		);

		let expected_position: LiquidatablePosition = LiquidatablePosition {
			market_id: 0.into(),
			direction: Direction::Long,
			amount_to_be_sold: 0.into(),
			liquidatable: false,
		};
		assert_eq!(expected_position, liquidatable_position);
		println!("{:?}", liquidatable_position);

		// Decrease the price of the asset
		let mut market_prices: Vec<MultipleMarketPrices> = Vec::new();
		let market_price1 = MultipleMarketPrices { market_id: markets[0].id, price: 5000.into() };
		market_prices.push(market_price1);
		assert_ok!(MarketPrices::update_multiple_market_prices(
			RuntimeOrigin::signed(1),
			market_prices.clone()
		));

		let mut market_price: MarketPrice = MarketPrices::market_price(markets[0].id);
		let mut expected_price: FixedI128 = 5000.into();
		assert_eq!(expected_price, market_price.price);

		// Call mark_under_collateralized_position for the account_id_1
		assert_ok!(RiskManagement::mark_under_collateralized_position(
			RuntimeOrigin::signed(1),
			account_id_1,
			markets[0].asset_collateral,
		));

		let liquidatable_position = Trading::deleveragable_or_liquidatable_position(
			account_id_1,
			markets[0].asset_collateral,
		);

		// let expected_position: LiquidatablePosition = LiquidatablePosition {
		// 	market_id: 0.into(),
		// 	direction: Direction::Long,
		// 	amount_to_be_sold: 0.into(),
		// 	liquidatable: false,
		// };
		// assert_eq!(expected_position, liquidatable_position);
		println!("{:?}", liquidatable_position);
	});
}
