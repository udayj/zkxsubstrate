use crate::mock::*;
use frame_support::assert_ok;
use primitive_types::U256;
use sp_arithmetic::FixedI128;
use sp_io::hashing::blake2_256;
use starknet_crypto::{sign, FieldElement};
use zkx_support::test_helpers::asset_helper::{eth, link, usdc};
use zkx_support::traits::{FieldElementExt, Hashable, U256Ext};
use zkx_support::types::{
	Asset, Direction, HashType, LiquidatablePosition, Market, MarketPrice, MultipleMarketPrices,
	Order, OrderType, Position, Side, TimeInForce, TradingAccountMinimal,
};

const ORDER_ID_1: u128 = 200_u128;
const ORDER_ID_2: u128 = 201_u128;
const ORDER_ID_3: u128 = 202_u128;
const ORDER_ID_4: u128 = 203_u128;
const ORDER_ID_5: u128 = 204_u128;
const ORDER_ID_6: u128 = 205_u128;

fn setup() -> (Vec<Market>, Vec<TradingAccountMinimal>, Vec<U256>) {
	let assets: Vec<Asset> = vec![eth(), usdc(), link()];
	assert_ok!(Assets::replace_all_assets(RuntimeOrigin::signed(1), assets));

	let market1: Market = Market {
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
	};
	let market2: Market = Market {
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

	let user_pub_key_3: U256 = U256::from_dec_str(
		"1927799101328918885926814969993421873905724180750168745093131010179897850144",
	)
	.unwrap();
	let user_pri_key_3: U256 = U256::from_dec_str(
		"3388506857955987752046415916181604993164423072000548640801744803879383940670",
	)
	.unwrap();
	let user_address_3: U256 = U256::from(102_u8);

	let user_pub_key_4: U256 = U256::from_dec_str(
		"824120678599933675767871867465569325984720238047137957464936400424120564339",
	)
	.unwrap();
	let user_pri_key_4: U256 = U256::from_dec_str(
		"84035867551811388210596922086133550045728262314839423570645036080104955628",
	)
	.unwrap();
	let user_address_4: U256 = U256::from(103_u8);

	let user_1 = TradingAccountMinimal {
		account_address: user_address_1,
		index: 0,
		pub_key: user_pub_key_1,
	};
	let user_2 = TradingAccountMinimal {
		account_address: user_address_2,
		index: 0,
		pub_key: user_pub_key_2,
	};
	let user_3 = TradingAccountMinimal {
		account_address: user_address_3,
		index: 0,
		pub_key: user_pub_key_3,
	};
	let user_4 = TradingAccountMinimal {
		account_address: user_address_4,
		index: 0,
		pub_key: user_pub_key_4,
	};
	let accounts: Vec<TradingAccountMinimal> = vec![user_1, user_2, user_3, user_4];
	assert_ok!(TradingAccounts::add_accounts(RuntimeOrigin::signed(1), accounts.clone()));

	let private_keys: Vec<U256> =
		vec![user_pri_key_1, user_pri_key_2, user_pri_key_3, user_pri_key_4];

	(markets, accounts, private_keys)
}

fn get_trading_account_id(trading_accounts: Vec<TradingAccountMinimal>, index: usize) -> U256 {
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
fn test_liquidation() {
	new_test_ext().execute_with(|| {
		let (markets, accounts, private_keys) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		let account_id_1: U256 = get_trading_account_id(accounts.clone(), 0);
		let account_id_2: U256 = get_trading_account_id(accounts.clone(), 1);

		let order_1 = Order {
			account_id: account_id_1,
			order_id: ORDER_ID_1,
			market_id: markets[0].id,
			order_type: OrderType::Limit,
			direction: Direction::Long,
			side: Side::Buy,
			price: 10000.into(),
			size: 5.into(),
			leverage: 5.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};
		let order_2 = Order {
			account_id: account_id_2,
			order_id: ORDER_ID_2,
			market_id: markets[0].id,
			order_type: OrderType::Market,
			direction: Direction::Short,
			side: Side::Buy,
			price: 10000.into(),
			size: 5.into(),
			leverage: 5.into(),
			slippage: FixedI128::from_inner(100000000000000000),
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
			market_id: markets[0].id,
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

		let position2 = Trading::positions(account_id_2, (markets[0].id, Direction::Short));
		let expected_position: Position = Position {
			market_id: markets[0].id,
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
			market_id: 0,
			direction: Direction::Long,
			amount_to_be_sold: 0.into(),
			liquidatable: false,
		};
		assert_eq!(expected_position, liquidatable_position);

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
			market_id: 0,
			direction: Direction::Long,
			amount_to_be_sold: 0.into(),
			liquidatable: false,
		};
		assert_eq!(expected_position, liquidatable_position);

		// Decrease the price of the asset
		let mut market_prices: Vec<MultipleMarketPrices> = Vec::new();
		let market_price1 = MultipleMarketPrices { market_id: markets[0].id, price: 8000.into() };
		market_prices.push(market_price1);
		assert_ok!(MarketPrices::update_multiple_market_prices(
			RuntimeOrigin::signed(1),
			market_prices.clone()
		));

		let market_price: MarketPrice = MarketPrices::market_price(markets[0].id);
		let expected_price: FixedI128 = 8000.into();
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

		let expected_position: LiquidatablePosition = LiquidatablePosition {
			market_id: 1,
			direction: Direction::Long,
			amount_to_be_sold: 5.into(),
			liquidatable: true,
		};
		assert_eq!(expected_position, liquidatable_position);

		// Place liquidation order
		let order_3 = Order {
			account_id: account_id_2,
			order_id: ORDER_ID_3,
			market_id: markets[0].id,
			order_type: OrderType::Limit,
			direction: Direction::Long,
			side: Side::Buy,
			price: 8000.into(),
			size: 5.into(),
			leverage: 5.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};
		let order_4 = Order {
			account_id: account_id_1,
			order_id: ORDER_ID_4,
			market_id: markets[0].id,
			order_type: OrderType::Liquidation,
			direction: Direction::Long,
			side: Side::Sell,
			price: 8000.into(),
			size: 5.into(),
			leverage: 1.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};

		let order_3 = sign_order(order_3, private_keys[1]);
		let order_4 = sign_order(order_4, private_keys[0]);
		let orders: Vec<Order> = vec![order_3, order_4];

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			U256::from(2_u8),
			5.into(),
			markets[0].id,
			8000.into(),
			orders
		));

		// Check liquidatable position of the user whose position is liquidated
		let liquidatable_position = Trading::deleveragable_or_liquidatable_position(
			account_id_1,
			markets[0].asset_collateral,
		);

		let expected_position: LiquidatablePosition = LiquidatablePosition {
			market_id: 0,
			direction: Direction::Long,
			amount_to_be_sold: 0.into(),
			liquidatable: false,
		};

		assert_eq!(expected_position, liquidatable_position);

		let balance_1 = TradingAccounts::balances(account_id_1, markets[0].asset_collateral);
		assert_eq!(balance_1, 0.into());
	});
}

#[test]
fn test_liquidation_after_deleveraging() {
	new_test_ext().execute_with(|| {
		let (markets, accounts, private_keys) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		let account_id_1: U256 = get_trading_account_id(accounts.clone(), 0);
		let account_id_2: U256 = get_trading_account_id(accounts.clone(), 1);

		let order_1 = Order {
			account_id: account_id_1,
			order_id: ORDER_ID_1,
			market_id: markets[0].id,
			order_type: OrderType::Limit,
			direction: Direction::Long,
			side: Side::Buy,
			price: 10000.into(),
			size: 5.into(),
			leverage: 5.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};
		let order_2 = Order {
			account_id: account_id_2,
			order_id: ORDER_ID_2,
			market_id: markets[0].id,
			order_type: OrderType::Market,
			direction: Direction::Short,
			side: Side::Buy,
			price: 10000.into(),
			size: 5.into(),
			leverage: 5.into(),
			slippage: FixedI128::from_inner(100000000000000000),
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
			market_id: markets[0].id,
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

		let position2 = Trading::positions(account_id_2, (markets[0].id, Direction::Short));
		let expected_position: Position = Position {
			market_id: markets[0].id,
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

		// Decrease the price of the asset
		let mut market_prices: Vec<MultipleMarketPrices> = Vec::new();
		let market_price1 = MultipleMarketPrices { market_id: markets[0].id, price: 8500.into() };
		market_prices.push(market_price1);
		assert_ok!(MarketPrices::update_multiple_market_prices(
			RuntimeOrigin::signed(1),
			market_prices.clone()
		));

		let market_price: MarketPrice = MarketPrices::market_price(markets[0].id);
		let expected_price: FixedI128 = 8500.into();
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

		let expected_position: LiquidatablePosition = LiquidatablePosition {
			market_id: 1,
			direction: Direction::Long,
			amount_to_be_sold: FixedI128::from_inner(321637426900584796),
			liquidatable: false,
		};

		assert_eq!(expected_position, liquidatable_position);

		// Place Deleveraging order
		let order_3 = Order {
			account_id: account_id_2,
			order_id: ORDER_ID_3,
			market_id: markets[0].id,
			order_type: OrderType::Limit,
			direction: Direction::Long,
			side: Side::Buy,
			price: 8500.into(),
			size: 5.into(),
			leverage: 5.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};
		let order_4 = Order {
			account_id: account_id_1,
			order_id: ORDER_ID_4,
			market_id: markets[0].id,
			order_type: OrderType::Deleveraging,
			direction: Direction::Long,
			side: Side::Sell,
			price: 8500.into(),
			size: FixedI128::from_inner(321637426900584796),
			leverage: 1.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};

		let order_3 = sign_order(order_3, private_keys[1]);
		let order_4 = sign_order(order_4, private_keys[0]);
		let orders: Vec<Order> = vec![order_3, order_4];

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			U256::from(2_u8),
			FixedI128::from_inner(321637426900584796),
			markets[0].id,
			8500.into(),
			orders
		));

		// Check liquidatable position of the user whose position is liquidated
		let liquidatable_position = Trading::deleveragable_or_liquidatable_position(
			account_id_1,
			markets[0].asset_collateral,
		);

		let expected_position: LiquidatablePosition = LiquidatablePosition {
			market_id: 0,
			direction: Direction::Long,
			amount_to_be_sold: 0.into(),
			liquidatable: false,
		};
		assert_eq!(expected_position, liquidatable_position);

		let balance_1 = TradingAccounts::balances(account_id_1, markets[0].asset_collateral);
		assert_eq!(balance_1, 10000.into());

		// Decrease the price of the asset again
		let mut market_prices: Vec<MultipleMarketPrices> = Vec::new();
		let market_price1 = MultipleMarketPrices { market_id: markets[0].id, price: 7000.into() };
		market_prices.push(market_price1);
		assert_ok!(MarketPrices::update_multiple_market_prices(
			RuntimeOrigin::signed(1),
			market_prices.clone()
		));

		let market_price: MarketPrice = MarketPrices::market_price(markets[0].id);
		let expected_price: FixedI128 = 7000.into();
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

		let expected_position: LiquidatablePosition = LiquidatablePosition {
			market_id: 1,
			direction: Direction::Long,
			amount_to_be_sold: FixedI128::from_inner(4678362573099415204),
			liquidatable: true,
		};
		assert_eq!(expected_position, liquidatable_position);
		println!("{:?}", liquidatable_position);

		// Place liquidation order
		let order_5 = Order {
			account_id: account_id_2,
			order_id: ORDER_ID_5,
			market_id: markets[0].id,
			order_type: OrderType::Limit,
			direction: Direction::Long,
			side: Side::Buy,
			price: 7000.into(),
			size: 5.into(),
			leverage: 5.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};
		let order_6 = Order {
			account_id: account_id_1,
			order_id: ORDER_ID_6,
			market_id: markets[0].id,
			order_type: OrderType::Liquidation,
			direction: Direction::Long,
			side: Side::Sell,
			price: 7000.into(),
			size: FixedI128::from_inner(4678362573099415204),
			leverage: 1.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};

		let order_5 = sign_order(order_5, private_keys[1]);
		let order_6 = sign_order(order_6, private_keys[0]);
		let orders: Vec<Order> = vec![order_5, order_6];

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			U256::from(3_u8),
			FixedI128::from_inner(4678362573099415204),
			markets[0].id,
			7000.into(),
			orders
		));

		// Check liquidatable position of the user whose position is liquidated
		let liquidatable_position = Trading::deleveragable_or_liquidatable_position(
			account_id_1,
			markets[0].asset_collateral,
		);

		let expected_position: LiquidatablePosition = LiquidatablePosition {
			market_id: 0,
			direction: Direction::Long,
			amount_to_be_sold: 0.into(),
			liquidatable: false,
		};

		assert_eq!(expected_position, liquidatable_position);

		let balance_1 = TradingAccounts::balances(account_id_1, markets[0].asset_collateral);
		assert_eq!(balance_1, FixedI128::from_inner(-4035087719298245612000));
	});
}

#[test]
#[should_panic(expected = "TradeBatchError")]
fn test_liquidation_with_invalid_order_type() {
	new_test_ext().execute_with(|| {
		let (markets, accounts, private_keys) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		let account_id_1: U256 = get_trading_account_id(accounts.clone(), 0);
		let account_id_2: U256 = get_trading_account_id(accounts.clone(), 1);

		let order_1 = Order {
			account_id: account_id_1,
			order_id: ORDER_ID_1,
			market_id: markets[0].id,
			order_type: OrderType::Limit,
			direction: Direction::Long,
			side: Side::Buy,
			price: 10000.into(),
			size: 5.into(),
			leverage: 5.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};
		let order_2 = Order {
			account_id: account_id_2,
			order_id: ORDER_ID_2,
			market_id: markets[0].id,
			order_type: OrderType::Market,
			direction: Direction::Short,
			side: Side::Buy,
			price: 10000.into(),
			size: 5.into(),
			leverage: 5.into(),
			slippage: FixedI128::from_inner(100000000000000000),
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
			market_id: markets[0].id,
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

		let position2 = Trading::positions(account_id_2, (markets[0].id, Direction::Short));
		let expected_position: Position = Position {
			market_id: markets[0].id,
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

		// Decrease the price of the asset
		let mut market_prices: Vec<MultipleMarketPrices> = Vec::new();
		let market_price1 = MultipleMarketPrices { market_id: markets[0].id, price: 8000.into() };
		market_prices.push(market_price1);
		assert_ok!(MarketPrices::update_multiple_market_prices(
			RuntimeOrigin::signed(1),
			market_prices.clone()
		));

		let market_price: MarketPrice = MarketPrices::market_price(markets[0].id);
		let expected_price: FixedI128 = 8000.into();
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

		let expected_position: LiquidatablePosition = LiquidatablePosition {
			market_id: 1,
			direction: Direction::Long,
			amount_to_be_sold: 5.into(),
			liquidatable: true,
		};
		assert_eq!(expected_position, liquidatable_position);

		// Place deleveraging order instead of liquidation order
		let order_3 = Order {
			account_id: account_id_2,
			order_id: ORDER_ID_3,
			market_id: markets[0].id,
			order_type: OrderType::Limit,
			direction: Direction::Long,
			side: Side::Buy,
			price: 8000.into(),
			size: 5.into(),
			leverage: 5.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};
		let order_4 = Order {
			account_id: account_id_1,
			order_id: ORDER_ID_4,
			market_id: markets[0].id,
			order_type: OrderType::Deleveraging,
			direction: Direction::Long,
			side: Side::Sell,
			price: 8000.into(),
			size: 5.into(),
			leverage: 1.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};

		let order_3 = sign_order(order_3, private_keys[1]);
		let order_4 = sign_order(order_4, private_keys[0]);
		let orders: Vec<Order> = vec![order_3, order_4];

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			U256::from(2_u8),
			5.into(),
			markets[0].id,
			8000.into(),
			orders
		));
	});
}

#[test]
#[should_panic(expected = "TradeBatchError")]
fn test_deleveraging_with_invalid_order_type() {
	new_test_ext().execute_with(|| {
		let (markets, accounts, private_keys) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		let account_id_1: U256 = get_trading_account_id(accounts.clone(), 0);
		let account_id_2: U256 = get_trading_account_id(accounts.clone(), 1);

		let order_1 = Order {
			account_id: account_id_1,
			order_id: ORDER_ID_1,
			market_id: markets[0].id,
			order_type: OrderType::Limit,
			direction: Direction::Long,
			side: Side::Buy,
			price: 10000.into(),
			size: 5.into(),
			leverage: 5.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};
		let order_2 = Order {
			account_id: account_id_2,
			order_id: ORDER_ID_2,
			market_id: markets[0].id,
			order_type: OrderType::Market,
			direction: Direction::Short,
			side: Side::Buy,
			price: 10000.into(),
			size: 5.into(),
			leverage: 5.into(),
			slippage: FixedI128::from_inner(100000000000000000),
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
			market_id: markets[0].id,
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

		let position2 = Trading::positions(account_id_2, (markets[0].id, Direction::Short));
		let expected_position: Position = Position {
			market_id: markets[0].id,
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

		// Decrease the price of the asset
		let mut market_prices: Vec<MultipleMarketPrices> = Vec::new();
		let market_price1 = MultipleMarketPrices { market_id: markets[0].id, price: 8500.into() };
		market_prices.push(market_price1);
		assert_ok!(MarketPrices::update_multiple_market_prices(
			RuntimeOrigin::signed(1),
			market_prices.clone()
		));

		let market_price: MarketPrice = MarketPrices::market_price(markets[0].id);
		let expected_price: FixedI128 = 8500.into();
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

		let expected_position: LiquidatablePosition = LiquidatablePosition {
			market_id: 1,
			direction: Direction::Long,
			amount_to_be_sold: FixedI128::from_inner(321637426900584796),
			liquidatable: false,
		};

		assert_eq!(expected_position, liquidatable_position);

		// Place Liquidation order instead of Deleveraging order
		let order_3 = Order {
			account_id: account_id_2,
			order_id: ORDER_ID_3,
			market_id: markets[0].id,
			order_type: OrderType::Limit,
			direction: Direction::Long,
			side: Side::Buy,
			price: 8500.into(),
			size: 5.into(),
			leverage: 5.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};
		let order_4 = Order {
			account_id: account_id_1,
			order_id: ORDER_ID_4,
			market_id: markets[0].id,
			order_type: OrderType::Liquidation,
			direction: Direction::Long,
			side: Side::Sell,
			price: 8500.into(),
			size: FixedI128::from_inner(321637426900584796),
			leverage: 1.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};

		let order_3 = sign_order(order_3, private_keys[1]);
		let order_4 = sign_order(order_4, private_keys[0]);
		let orders: Vec<Order> = vec![order_3, order_4];

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			U256::from(2_u8),
			FixedI128::from_inner(321637426900584796),
			markets[0].id,
			8500.into(),
			orders
		));
	});
}

#[test]
#[should_panic(expected = "TradeBatchError")]
fn test_deleveraging_with_invalid_market_id() {
	new_test_ext().execute_with(|| {
		let (markets, accounts, private_keys) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		let account_id_1: U256 = get_trading_account_id(accounts.clone(), 0);
		let account_id_2: U256 = get_trading_account_id(accounts.clone(), 1);

		let order_1 = Order {
			account_id: account_id_1,
			order_id: ORDER_ID_1,
			market_id: markets[0].id,
			order_type: OrderType::Limit,
			direction: Direction::Long,
			side: Side::Buy,
			price: 10000.into(),
			size: 5.into(),
			leverage: 5.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};
		let order_2 = Order {
			account_id: account_id_2,
			order_id: ORDER_ID_2,
			market_id: markets[0].id,
			order_type: OrderType::Market,
			direction: Direction::Short,
			side: Side::Buy,
			price: 10000.into(),
			size: 5.into(),
			leverage: 5.into(),
			slippage: FixedI128::from_inner(100000000000000000),
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
			market_id: markets[0].id,
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

		let position2 = Trading::positions(account_id_2, (markets[0].id, Direction::Short));
		let expected_position: Position = Position {
			market_id: markets[0].id,
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

		// Decrease the price of the asset
		let mut market_prices: Vec<MultipleMarketPrices> = Vec::new();
		let market_price1 = MultipleMarketPrices { market_id: markets[0].id, price: 8500.into() };
		market_prices.push(market_price1);
		assert_ok!(MarketPrices::update_multiple_market_prices(
			RuntimeOrigin::signed(1),
			market_prices.clone()
		));

		let market_price: MarketPrice = MarketPrices::market_price(markets[0].id);
		let expected_price: FixedI128 = 8500.into();
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

		let expected_position: LiquidatablePosition = LiquidatablePosition {
			market_id: 1,
			direction: Direction::Long,
			amount_to_be_sold: FixedI128::from_inner(321637426900584796),
			liquidatable: false,
		};

		assert_eq!(expected_position, liquidatable_position);

		// Place Deleveraging order
		let order_3 = Order {
			account_id: account_id_2,
			order_id: ORDER_ID_3,
			market_id: markets[1].id,
			order_type: OrderType::Limit,
			direction: Direction::Long,
			side: Side::Buy,
			price: 8500.into(),
			size: 5.into(),
			leverage: 5.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};
		let order_4 = Order {
			account_id: account_id_1,
			order_id: ORDER_ID_4,
			market_id: markets[1].id,
			order_type: OrderType::Deleveraging,
			direction: Direction::Long,
			side: Side::Sell,
			price: 8500.into(),
			size: FixedI128::from_inner(321637426900584796),
			leverage: 1.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};

		let order_3 = sign_order(order_3, private_keys[1]);
		let order_4 = sign_order(order_4, private_keys[0]);
		let orders: Vec<Order> = vec![order_3, order_4];

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			U256::from(2_u8),
			FixedI128::from_inner(321637426900584796),
			markets[0].id,
			8500.into(),
			orders
		));
	});
}

#[test]
#[should_panic(expected = "TradeBatchError")]
fn test_deleveraging_with_invalid_order_direction() {
	new_test_ext().execute_with(|| {
		let (markets, accounts, private_keys) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		let account_id_1: U256 = get_trading_account_id(accounts.clone(), 0);
		let account_id_2: U256 = get_trading_account_id(accounts.clone(), 1);

		let order_1 = Order {
			account_id: account_id_1,
			order_id: ORDER_ID_1,
			market_id: markets[0].id,
			order_type: OrderType::Limit,
			direction: Direction::Long,
			side: Side::Buy,
			price: 10000.into(),
			size: 5.into(),
			leverage: 5.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};
		let order_2 = Order {
			account_id: account_id_2,
			order_id: ORDER_ID_2,
			market_id: markets[0].id,
			order_type: OrderType::Market,
			direction: Direction::Short,
			side: Side::Buy,
			price: 10000.into(),
			size: 5.into(),
			leverage: 5.into(),
			slippage: FixedI128::from_inner(100000000000000000),
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
			market_id: markets[0].id,
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

		let position2 = Trading::positions(account_id_2, (markets[0].id, Direction::Short));
		let expected_position: Position = Position {
			market_id: markets[0].id,
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

		// Decrease the price of the asset
		let mut market_prices: Vec<MultipleMarketPrices> = Vec::new();
		let market_price1 = MultipleMarketPrices { market_id: markets[0].id, price: 8500.into() };
		market_prices.push(market_price1);
		assert_ok!(MarketPrices::update_multiple_market_prices(
			RuntimeOrigin::signed(1),
			market_prices.clone()
		));

		let market_price: MarketPrice = MarketPrices::market_price(markets[0].id);
		let expected_price: FixedI128 = 8500.into();
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

		let expected_position: LiquidatablePosition = LiquidatablePosition {
			market_id: 1,
			direction: Direction::Long,
			amount_to_be_sold: FixedI128::from_inner(321637426900584796),
			liquidatable: false,
		};

		assert_eq!(expected_position, liquidatable_position);

		// Place Deleveraging order
		let order_3 = Order {
			account_id: account_id_2,
			order_id: ORDER_ID_3,
			market_id: markets[0].id,
			order_type: OrderType::Limit,
			direction: Direction::Long,
			side: Side::Buy,
			price: 8500.into(),
			size: 5.into(),
			leverage: 5.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};
		let order_4 = Order {
			account_id: account_id_1,
			order_id: ORDER_ID_4,
			market_id: markets[0].id,
			order_type: OrderType::Deleveraging,
			direction: Direction::Short,
			side: Side::Sell,
			price: 8500.into(),
			size: FixedI128::from_inner(321637426900584796),
			leverage: 1.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};

		let order_3 = sign_order(order_3, private_keys[1]);
		let order_4 = sign_order(order_4, private_keys[0]);
		let orders: Vec<Order> = vec![order_3, order_4];

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			U256::from(2_u8),
			FixedI128::from_inner(321637426900584796),
			markets[0].id,
			8500.into(),
			orders
		));
	});
}

#[test]
#[should_panic(expected = "TradeBatchError")]
fn test_shouldnt_liquidate_long_leverage_1() {
	new_test_ext().execute_with(|| {
		let (markets, accounts, private_keys) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		let account_id_1: U256 = get_trading_account_id(accounts.clone(), 0);
		let account_id_2: U256 = get_trading_account_id(accounts.clone(), 1);

		let order_1 = Order {
			account_id: account_id_1,
			order_id: ORDER_ID_1,
			market_id: markets[0].id,
			order_type: OrderType::Limit,
			direction: Direction::Long,
			side: Side::Buy,
			price: 10000.into(),
			size: 1.into(),
			leverage: 1.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};
		let order_2 = Order {
			account_id: account_id_2,
			order_id: ORDER_ID_2,
			market_id: markets[0].id,
			order_type: OrderType::Market,
			direction: Direction::Short,
			side: Side::Buy,
			price: 10000.into(),
			size: 5.into(),
			leverage: 5.into(),
			slippage: FixedI128::from_inner(100000000000000000),
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
			market_id: markets[0].id,
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

		let position2 = Trading::positions(account_id_2, (markets[0].id, Direction::Short));
		let expected_position: Position = Position {
			market_id: markets[0].id,
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

		// Decrease the price of the asset
		let mut market_prices: Vec<MultipleMarketPrices> = Vec::new();
		let market_price1 = MultipleMarketPrices { market_id: markets[0].id, price: 8000.into() };
		market_prices.push(market_price1);
		assert_ok!(MarketPrices::update_multiple_market_prices(
			RuntimeOrigin::signed(1),
			market_prices.clone()
		));

		let market_price: MarketPrice = MarketPrices::market_price(markets[0].id);
		let expected_price: FixedI128 = 8000.into();
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

		let expected_position: LiquidatablePosition = LiquidatablePosition {
			market_id: 1,
			direction: Direction::Long,
			amount_to_be_sold: 5.into(),
			liquidatable: true,
		};
		assert_eq!(expected_position, liquidatable_position);

		// Place deleveraging order instead of liquidation order
		let order_3 = Order {
			account_id: account_id_2,
			order_id: ORDER_ID_3,
			market_id: markets[0].id,
			order_type: OrderType::Limit,
			direction: Direction::Long,
			side: Side::Buy,
			price: 8000.into(),
			size: 5.into(),
			leverage: 5.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};
		let order_4 = Order {
			account_id: account_id_1,
			order_id: ORDER_ID_4,
			market_id: markets[0].id,
			order_type: OrderType::Deleveraging,
			direction: Direction::Long,
			side: Side::Sell,
			price: 8000.into(),
			size: 5.into(),
			leverage: 1.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};

		let order_3 = sign_order(order_3, private_keys[1]);
		let order_4 = sign_order(order_4, private_keys[0]);
		let orders: Vec<Order> = vec![order_3, order_4];

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			U256::from(2_u8),
			5.into(),
			markets[0].id,
			8000.into(),
			orders
		));
	});
}
