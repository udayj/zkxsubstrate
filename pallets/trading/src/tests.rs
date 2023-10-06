use crate::{mock::*, Event};
use frame_support::assert_ok;
use primitive_types::U256;
use sp_arithmetic::FixedI128;
use sp_io::hashing::blake2_256;
use starknet_crypto::{sign, FieldElement};
use zkx_support::test_helpers::asset_helper::{btc, eth, link, usdc};
use zkx_support::traits::{FieldElementExt, Hashable, U256Ext};
use zkx_support::types::{
	Asset, BaseFee, Direction, Discount, HashType, Market, Order, OrderType, Position, Side,
	TimeInForce, TradingAccountMinimal,
};

const ORDER_ID_1: u128 = 200_u128;
const ORDER_ID_2: u128 = 201_u128;
const ORDER_ID_3: u128 = 202_u128;
const ORDER_ID_4: u128 = 203_u128;
const ORDER_ID_5: u128 = 204_u128;
const ORDER_ID_6: u128 = 205_u128;

fn setup() -> (Vec<Market>, Vec<TradingAccountMinimal>, Vec<U256>) {
	assert_ok!(Timestamp::set(None.into(), 100));

	let assets: Vec<Asset> = vec![eth(), usdc(), link(), btc()];
	assert_ok!(Assets::replace_all_assets(RuntimeOrigin::signed(1), assets));

	let market1: Market = Market {
		id: 1,
		version: 1,
		asset: 1163151370,
		asset_collateral: 93816115890698,
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
		maintenance_margin_fraction: 1.into(),
		initial_margin_fraction: 1.into(),
		incremental_initial_margin_fraction: 1.into(),
		incremental_position_size: 1.into(),
		baseline_position_size: 1.into(),
		maximum_position_size: 1.into(),
	};

	let market2: Market = Market {
		id: 2,
		version: 1,
		asset: 1279872587,
		asset_collateral: 93816115890698,
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

fn setup_fee() -> (Vec<u8>, Vec<BaseFee>, Vec<u8>, Vec<Discount>) {
	let fee_tiers: Vec<u8> = vec![1, 2, 3];
	let mut fee_details: Vec<BaseFee> = Vec::new();
	let base_fee1 = BaseFee {
		number_of_tokens: 0.into(),
		maker_fee: FixedI128::from_inner(20000000000000000),
		taker_fee: FixedI128::from_inner(50000000000000000),
	};
	let base_fee2 = BaseFee {
		number_of_tokens: 1000.into(),
		maker_fee: FixedI128::from_inner(15000000000000000),
		taker_fee: FixedI128::from_inner(40000000000000000),
	};
	let base_fee3 = BaseFee {
		number_of_tokens: 5000.into(),
		maker_fee: FixedI128::from_inner(10000000000000000),
		taker_fee: FixedI128::from_inner(35000000000000000),
	};
	fee_details.push(base_fee1);
	fee_details.push(base_fee2);
	fee_details.push(base_fee3);

	let discount_tiers: Vec<u8> = vec![1, 2, 3, 4];
	let mut discount_details: Vec<Discount> = Vec::new();
	let discount1 =
		Discount { number_of_tokens: 0.into(), discount: FixedI128::from_inner(30000000000000000) };
	let discount2 = Discount {
		number_of_tokens: 1000.into(),
		discount: FixedI128::from_inner(50000000000000000),
	};
	let discount3 = Discount {
		number_of_tokens: 4000.into(),
		discount: FixedI128::from_inner(75000000000000000),
	};
	let discount4 = Discount {
		number_of_tokens: 7500.into(),
		discount: FixedI128::from_inner(100000000000000000),
	};
	discount_details.push(discount1);
	discount_details.push(discount2);
	discount_details.push(discount3);
	discount_details.push(discount4);

	(fee_tiers, fee_details, discount_tiers, discount_details)
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
// basic open trade without any leverage
fn it_works_for_open_trade_simple() {
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
			price: 100.into(),
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
			price: 100.into(),
			size: 1.into(),
			leverage: 1.into(),
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
			1.into(),
			markets[0].id,
			100.into(),
			orders
		));

		// System::assert_has_event(Event::OrderError { order_id: 12, error_code: 25 }.into());

		let position1 = Trading::positions(account_id_1, (markets[0].id, Direction::Long));
		let expected_position: Position = Position {
			market_id: markets[0].id,
			avg_execution_price: 100.into(),
			size: 1.into(),
			direction: Direction::Long,
			side: Side::Buy,
			margin_amount: 100.into(),
			borrowed_amount: 0.into(),
			leverage: 1.into(),
			realized_pnl: 0.into(),
		};
		assert_eq!(expected_position, position1);

		let position2 = Trading::positions(account_id_2, (markets[0].id, Direction::Short));
		let expected_position: Position = Position {
			market_id: markets[0].id,
			avg_execution_price: 100.into(),
			size: 1.into(),
			direction: Direction::Short,
			side: Side::Buy,
			margin_amount: 100.into(),
			borrowed_amount: 0.into(),
			leverage: 1.into(),
			realized_pnl: 0.into(),
		};
		assert_eq!(expected_position, position2);
	});
}

#[test]
// basic open trade with leverage
fn it_works_for_open_trade_with_leverage() {
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
			price: 100.into(),
			size: 1.into(),
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
			price: 100.into(),
			size: 1.into(),
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
			1.into(),
			markets[0].id,
			100.into(),
			orders
		));

		let position1 = Trading::positions(account_id_1, (markets[0].id, Direction::Long));
		let expected_position: Position = Position {
			market_id: markets[0].id,
			avg_execution_price: 100.into(),
			size: 1.into(),
			direction: Direction::Long,
			side: Side::Buy,
			margin_amount: 20.into(),
			borrowed_amount: 80.into(),
			leverage: 5.into(),
			realized_pnl: 0.into(),
		};
		assert_eq!(expected_position, position1);

		let position2 = Trading::positions(account_id_2, (markets[0].id, Direction::Short));
		let expected_position: Position = Position {
			market_id: markets[0].id,
			avg_execution_price: 100.into(),
			size: 1.into(),
			direction: Direction::Short,
			side: Side::Buy,
			margin_amount: 20.into(),
			borrowed_amount: 80.into(),
			leverage: 5.into(),
			realized_pnl: 0.into(),
		};
		assert_eq!(expected_position, position2);
	});
}

#[test]
// basic open and close trade without any leverage
fn it_works_for_close_trade_simple() {
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
			price: 100.into(),
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
			price: 100.into(),
			size: 1.into(),
			leverage: 1.into(),
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
			1.into(),
			markets[0].id,
			100.into(),
			orders
		));

		// Close orders
		let order_3 = Order {
			account_id: account_id_1,
			order_id: ORDER_ID_3,
			market_id: markets[0].id,
			order_type: OrderType::Limit,
			direction: Direction::Long,
			side: Side::Sell,
			price: 105.into(),
			size: 1.into(),
			leverage: 1.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};
		let order_4 = Order {
			account_id: account_id_2,
			order_id: ORDER_ID_4,
			market_id: markets[0].id,
			order_type: OrderType::Market,
			direction: Direction::Short,
			side: Side::Sell,
			price: 100.into(),
			size: 1.into(),
			leverage: 1.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};

		let order_3 = sign_order(order_3, private_keys[0]);
		let order_4 = sign_order(order_4, private_keys[1]);
		let orders: Vec<Order> = vec![order_3, order_4];

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			U256::from(2_u8),
			1.into(),
			markets[0].id,
			105.into(),
			orders
		));

		let usdc_id: u128 = 93816115890698;
		let balance_1 = TradingAccounts::balances(account_id_1, usdc_id);
		assert_eq!(balance_1, 10005.into());
		let balance_2 = TradingAccounts::balances(account_id_2, usdc_id);
		assert_eq!(balance_2, 9995.into());
		let locked_1 = TradingAccounts::locked_margin(account_id_1, usdc_id);
		assert_eq!(locked_1, 0.into());

		// let event_record: frame_system::EventRecord<_, _> = System::events().pop().unwrap();
		// println!("Events: {:?}", event_record);
	});
}

#[test]
// partially open position by executing in different batches
fn it_works_for_open_trade_partial_open() {
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
			price: 100.into(),
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
			price: 100.into(),
			size: 2.into(),
			leverage: 1.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};

		let order_1 = sign_order(order_1, private_keys[0]);
		let order_2 = sign_order(order_2, private_keys[1]);
		let orders: Vec<Order> = vec![order_1, order_2.clone()];

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			U256::from(1_u8),
			1.into(),
			markets[0].id,
			100.into(),
			orders
		));

		let order_3 = Order {
			account_id: account_id_1,
			order_id: ORDER_ID_3,
			market_id: markets[0].id,
			order_type: OrderType::Limit,
			direction: Direction::Long,
			side: Side::Buy,
			price: 98.into(),
			size: 1.into(),
			leverage: 1.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};

		let order_3 = sign_order(order_3, private_keys[0]);
		let orders: Vec<Order> = vec![order_3, order_2];

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			U256::from(2_u8),
			1.into(),
			markets[0].id,
			98.into(),
			orders
		));

		let position1 = Trading::positions(account_id_2, (markets[0].id, Direction::Short));
		let expected_position: Position = Position {
			market_id: markets[0].id,
			avg_execution_price: 99.into(),
			size: 2.into(),
			direction: Direction::Short,
			side: Side::Buy,
			margin_amount: 198.into(),
			borrowed_amount: 0.into(),
			leverage: 1.into(),
			realized_pnl: 0.into(),
		};
		assert_eq!(expected_position, position1);
	});
}

#[test]
// partially close position by executing in different batches
fn it_works_for_close_trade_partial_close() {
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
			price: 100.into(),
			size: 2.into(),
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
			price: 100.into(),
			size: 2.into(),
			leverage: 1.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};

		let order_1 = sign_order(order_1, private_keys[0]);
		let order_2 = sign_order(order_2, private_keys[1]);
		let orders: Vec<Order> = vec![order_1, order_2.clone()];

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			U256::from(1_u8),
			2.into(),
			markets[0].id,
			100.into(),
			orders
		));

		let order_3 = Order {
			account_id: account_id_1,
			order_id: ORDER_ID_3,
			market_id: markets[0].id,
			order_type: OrderType::Limit,
			direction: Direction::Long,
			side: Side::Sell,
			price: 104.into(),
			size: 1.into(),
			leverage: 1.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};
		let order_4 = Order {
			account_id: account_id_2,
			order_id: ORDER_ID_4,
			market_id: markets[0].id,
			order_type: OrderType::Market,
			direction: Direction::Short,
			side: Side::Sell,
			price: 100.into(),
			size: 2.into(),
			leverage: 1.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};

		let order_3 = sign_order(order_3, private_keys[0]);
		let order_4 = sign_order(order_4, private_keys[1]);
		let orders: Vec<Order> = vec![order_3, order_4.clone()];

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			U256::from(2_u8),
			1.into(),
			markets[0].id,
			105.into(),
			orders
		));

		let order_5 = Order {
			account_id: account_id_1,
			order_id: ORDER_ID_5,
			market_id: markets[0].id,
			order_type: OrderType::Limit,
			direction: Direction::Long,
			side: Side::Sell,
			price: 98.into(),
			size: 1.into(),
			leverage: 1.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};

		let order_5 = sign_order(order_5, private_keys[0]);
		let orders: Vec<Order> = vec![order_5, order_4];

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			U256::from(3_u8),
			1.into(),
			markets[0].id,
			100.into(),
			orders
		));

		let usdc_id: u128 = 93816115890698;
		let balance_1 = TradingAccounts::balances(account_id_2, usdc_id);
		assert_eq!(balance_1, 9998.into());
	});
}

#[test]
// trade batch with multiple makers and a taker
fn it_works_for_open_trade_multiple_makers() {
	new_test_ext().execute_with(|| {
		let (markets, accounts, private_keys) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		let account_id_1: U256 = get_trading_account_id(accounts.clone(), 0);
		let account_id_2: U256 = get_trading_account_id(accounts.clone(), 1);
		let account_id_3: U256 = get_trading_account_id(accounts.clone(), 2);
		let account_id_4: U256 = get_trading_account_id(accounts.clone(), 3);

		let order_1 = Order {
			account_id: account_id_1,
			order_id: ORDER_ID_1,
			market_id: markets[0].id,
			order_type: OrderType::Limit,
			direction: Direction::Short,
			side: Side::Buy,
			price: 105.into(),
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
			order_type: OrderType::Limit,
			direction: Direction::Short,
			side: Side::Buy,
			price: 99.into(),
			size: 1.into(),
			leverage: 1.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};
		let order_3 = Order {
			account_id: account_id_3,
			order_id: ORDER_ID_3,
			market_id: markets[0].id,
			order_type: OrderType::Limit,
			direction: Direction::Short,
			side: Side::Buy,
			price: 104.into(),
			size: 2.into(),
			leverage: 1.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};
		let order_4 = Order {
			account_id: account_id_4,
			order_id: ORDER_ID_4,
			market_id: markets[0].id,
			order_type: OrderType::Market,
			direction: Direction::Long,
			side: Side::Buy,
			price: 100.into(),
			size: 3.into(),
			leverage: 1.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};

		let order_1 = sign_order(order_1, private_keys[0]);
		let order_2 = sign_order(order_2, private_keys[1]);
		let order_3 = sign_order(order_3, private_keys[2]);
		let order_4 = sign_order(order_4, private_keys[3]);
		let orders: Vec<Order> = vec![order_1, order_2, order_3, order_4];

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			U256::from(1_u8),
			3.into(),
			markets[0].id,
			100.into(),
			orders
		));

		let event_record: frame_system::EventRecord<_, _> = System::events().pop().unwrap();
		println!("Events: {:?}", event_record);
	});
}

#[test]
#[should_panic(expected = "TradeBatchError")]
// trade batch with previously executed batch_id
fn it_reverts_for_trade_with_same_batch_id() {
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
			price: 100.into(),
			size: 2.into(),
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
			price: 100.into(),
			size: 2.into(),
			leverage: 1.into(),
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
			2.into(),
			markets[0].id,
			100.into(),
			orders
		));

		let order_3 = Order {
			account_id: account_id_1,
			order_id: ORDER_ID_3,
			market_id: markets[0].id,
			order_type: OrderType::Limit,
			direction: Direction::Long,
			side: Side::Buy,
			price: 100.into(),
			size: 1.into(),
			leverage: 1.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};
		let order_4 = Order {
			account_id: account_id_2,
			order_id: ORDER_ID_4,
			market_id: markets[0].id,
			order_type: OrderType::Market,
			direction: Direction::Short,
			side: Side::Buy,
			price: 100.into(),
			size: 1.into(),
			leverage: 1.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};

		let order_3 = sign_order(order_3, private_keys[0]);
		let order_4 = sign_order(order_4, private_keys[1]);
		let orders: Vec<Order> = vec![order_3, order_4];

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			U256::from(1_u8),
			1.into(),
			markets[0].id,
			100.into(),
			orders
		));
	});
}

#[test]
#[should_panic(expected = "TradeBatchError")]
// trade batch with invalid market_id
fn it_reverts_for_trade_with_invalid_market() {
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
			price: 100.into(),
			size: 2.into(),
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
			price: 100.into(),
			size: 2.into(),
			leverage: 1.into(),
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
			2.into(),
			123,
			100.into(),
			orders
		));
	});
}

#[test]
#[should_panic(expected = "TradeBatchError")]
// trade batch with quantity_locked as 0
fn it_reverts_for_trade_with_quantity_locked_zero() {
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
			price: 100.into(),
			size: 2.into(),
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
			price: 100.into(),
			size: 2.into(),
			leverage: 1.into(),
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
			0.into(),
			markets[0].id,
			100.into(),
			orders
		));
	});
}

#[test]
#[should_panic(expected = "TradeBatchError")]
// Taker tries to close a position which is already completely closed
fn it_reverts_when_taker_tries_to_close_already_closed_position() {
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
			price: 100.into(),
			size: 2.into(),
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
			price: 100.into(),
			size: 2.into(),
			leverage: 1.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};

		let order_1 = sign_order(order_1, private_keys[0]);
		let order_2 = sign_order(order_2, private_keys[1]);
		let orders: Vec<Order> = vec![order_1, order_2.clone()];

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			U256::from(1_u8),
			2.into(),
			markets[0].id,
			100.into(),
			orders
		));

		let order_3 = Order {
			account_id: account_id_1,
			order_id: ORDER_ID_3,
			market_id: markets[0].id,
			order_type: OrderType::Limit,
			direction: Direction::Long,
			side: Side::Sell,
			price: 104.into(),
			size: 2.into(),
			leverage: 1.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};
		let order_4 = Order {
			account_id: account_id_2,
			order_id: ORDER_ID_4,
			market_id: markets[0].id,
			order_type: OrderType::Market,
			direction: Direction::Short,
			side: Side::Sell,
			price: 100.into(),
			size: 2.into(),
			leverage: 1.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};

		let order_3 = sign_order(order_3, private_keys[0]);
		let order_4 = sign_order(order_4, private_keys[1]);
		let orders: Vec<Order> = vec![order_3, order_4.clone()];

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			U256::from(2_u8),
			2.into(),
			markets[0].id,
			105.into(),
			orders
		));

		let order_5 = Order {
			account_id: account_id_1,
			order_id: ORDER_ID_5,
			market_id: markets[0].id,
			order_type: OrderType::Limit,
			direction: Direction::Short,
			side: Side::Buy,
			price: 98.into(),
			size: 1.into(),
			leverage: 1.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};
		let order_6 = Order {
			account_id: account_id_2,
			order_id: ORDER_ID_6,
			market_id: markets[0].id,
			order_type: OrderType::Limit,
			direction: Direction::Short,
			side: Side::Sell,
			price: 98.into(),
			size: 1.into(),
			leverage: 1.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};

		let order_5 = sign_order(order_5, private_keys[0]);
		let order_6 = sign_order(order_6, private_keys[1]);
		let orders: Vec<Order> = vec![order_5, order_6];

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			U256::from(3_u8),
			1.into(),
			markets[0].id,
			100.into(),
			orders
		));
	});
}

#[test]
// Non registered user tries to open a position
fn it_produces_error_when_user_not_registered() {
	new_test_ext().execute_with(|| {
		let (markets, accounts, private_keys) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		let account_id_2: U256 = get_trading_account_id(accounts.clone(), 1);

		let order_1 = Order {
			account_id: 1.into(),
			order_id: ORDER_ID_1,
			market_id: markets[0].id,
			order_type: OrderType::Limit,
			direction: Direction::Long,
			side: Side::Buy,
			price: 100.into(),
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
			price: 100.into(),
			size: 1.into(),
			leverage: 1.into(),
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
			1.into(),
			markets[0].id,
			100.into(),
			orders
		));

		System::assert_has_event(Event::OrderError { order_id: 200, error_code: 510 }.into());
	});
}

#[test]
// Tries to open a position with size lesser than allowed minimum order size
fn it_produces_error_when_size_too_small() {
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
			price: 100.into(),
			size: FixedI128::from_inner(500000000000000000),
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
			price: 100.into(),
			size: 1.into(),
			leverage: 1.into(),
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
			1.into(),
			markets[0].id,
			100.into(),
			orders
		));

		System::assert_has_event(Event::OrderError { order_id: 200, error_code: 505 }.into());
	});
}

#[test]
// Tries to open a position with different market_id compared to the one passed in argument
fn it_produces_error_when_market_id_is_different() {
	new_test_ext().execute_with(|| {
		let (markets, accounts, private_keys) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		let account_id_1: U256 = get_trading_account_id(accounts.clone(), 0);
		let account_id_2: U256 = get_trading_account_id(accounts.clone(), 1);

		let order_1 = Order {
			account_id: account_id_1,
			order_id: ORDER_ID_1,
			market_id: 789,
			order_type: OrderType::Limit,
			direction: Direction::Long,
			side: Side::Buy,
			price: 100.into(),
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
			price: 100.into(),
			size: 1.into(),
			leverage: 1.into(),
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
			1.into(),
			markets[0].id,
			100.into(),
			orders
		));

		System::assert_has_event(Event::OrderError { order_id: 200, error_code: 504 }.into());
	});
}

#[test]
// Tries to open a position leverage more than currently allowed leverage
fn it_produces_error_when_leverage_is_invalid() {
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
			price: 100.into(),
			size: 1.into(),
			leverage: 9.into(),
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
			price: 100.into(),
			size: 1.into(),
			leverage: 1.into(),
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
			1.into(),
			markets[0].id,
			100.into(),
			orders
		));

		System::assert_has_event(Event::OrderError { order_id: 200, error_code: 502 }.into());
	});
}

#[test]
// Tries to open a position with invalid signature
fn it_produces_error_when_signature_is_invalid() {
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
			price: 100.into(),
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
			price: 100.into(),
			size: 1.into(),
			leverage: 1.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 123.into(),
			sig_s: 456.into(),
			hash_type: HashType::Pedersen,
		};

		// let order_1 = sign_order(order_1, private_keys[0]);
		let order_2 = sign_order(order_2, private_keys[1]);
		let orders: Vec<Order> = vec![order_1, order_2];

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			U256::from(1_u8),
			1.into(),
			markets[0].id,
			100.into(),
			orders
		));

		System::assert_has_event(Event::OrderError { order_id: 200, error_code: 536 }.into());
	});
}

#[test]
// 2nd maker order with side and direction that does not match with the first maker
fn it_produces_error_for_maker_when_side_and_direction_is_invalid() {
	new_test_ext().execute_with(|| {
		let (markets, accounts, private_keys) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		let account_id_1: U256 = get_trading_account_id(accounts.clone(), 0);
		let account_id_2: U256 = get_trading_account_id(accounts.clone(), 1);
		let account_id_4: U256 = get_trading_account_id(accounts.clone(), 3);

		let order_1 = Order {
			account_id: account_id_1,
			order_id: ORDER_ID_1,
			market_id: markets[0].id,
			order_type: OrderType::Limit,
			direction: Direction::Short,
			side: Side::Buy,
			price: 105.into(),
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
			order_type: OrderType::Limit,
			direction: Direction::Long,
			side: Side::Buy,
			price: 99.into(),
			size: 1.into(),
			leverage: 1.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};
		let order_4 = Order {
			account_id: account_id_4,
			order_id: ORDER_ID_4,
			market_id: markets[0].id,
			order_type: OrderType::Market,
			direction: Direction::Long,
			side: Side::Buy,
			price: 100.into(),
			size: 3.into(),
			leverage: 1.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};

		let order_1 = sign_order(order_1, private_keys[0]);
		let order_2 = sign_order(order_2, private_keys[1]);
		let order_4 = sign_order(order_4, private_keys[3]);
		let orders: Vec<Order> = vec![order_1, order_2, order_4];

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			U256::from(1_u8),
			3.into(),
			markets[0].id,
			100.into(),
			orders
		));

		System::assert_has_event(Event::OrderError { order_id: 201, error_code: 512 }.into());
	});
}

#[test]
// Maker order type is not limit
fn it_produces_error_when_maker_is_market_order() {
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
			order_type: OrderType::Market,
			direction: Direction::Long,
			side: Side::Buy,
			price: 100.into(),
			size: 1.into(),
			leverage: 8.into(),
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
			price: 100.into(),
			size: 1.into(),
			leverage: 1.into(),
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
			1.into(),
			markets[0].id,
			100.into(),
			orders
		));

		System::assert_has_event(Event::OrderError { order_id: 200, error_code: 518 }.into());
	});
}

#[test]
// Maker tries to close a position which is already completely closed
fn it_reverts_when_maker_tries_to_close_already_closed_position() {
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
			price: 100.into(),
			size: 2.into(),
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
			price: 100.into(),
			size: 2.into(),
			leverage: 1.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};

		let order_1 = sign_order(order_1, private_keys[0]);
		let order_2 = sign_order(order_2, private_keys[1]);
		let orders: Vec<Order> = vec![order_1, order_2.clone()];

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			U256::from(1_u8),
			2.into(),
			markets[0].id,
			100.into(),
			orders
		));

		let order_3 = Order {
			account_id: account_id_1,
			order_id: ORDER_ID_3,
			market_id: markets[0].id,
			order_type: OrderType::Limit,
			direction: Direction::Long,
			side: Side::Sell,
			price: 104.into(),
			size: 2.into(),
			leverage: 1.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};
		let order_4 = Order {
			account_id: account_id_2,
			order_id: ORDER_ID_4,
			market_id: markets[0].id,
			order_type: OrderType::Market,
			direction: Direction::Short,
			side: Side::Sell,
			price: 100.into(),
			size: 2.into(),
			leverage: 1.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};

		let order_3 = sign_order(order_3, private_keys[0]);
		let order_4 = sign_order(order_4, private_keys[1]);
		let orders: Vec<Order> = vec![order_3, order_4.clone()];

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			U256::from(2_u8),
			2.into(),
			markets[0].id,
			105.into(),
			orders
		));

		let order_5 = Order {
			account_id: account_id_1,
			order_id: ORDER_ID_5,
			market_id: markets[0].id,
			order_type: OrderType::Limit,
			direction: Direction::Long,
			side: Side::Sell,
			price: 98.into(),
			size: 1.into(),
			leverage: 1.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};
		let order_6 = Order {
			account_id: account_id_2,
			order_id: ORDER_ID_6,
			market_id: markets[0].id,
			order_type: OrderType::Limit,
			direction: Direction::Long,
			side: Side::Buy,
			price: 98.into(),
			size: 1.into(),
			leverage: 1.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};

		let order_5 = sign_order(order_5, private_keys[0]);
		let order_6 = sign_order(order_6, private_keys[1]);
		let orders: Vec<Order> = vec![order_5, order_6];

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			U256::from(3_u8),
			1.into(),
			markets[0].id,
			100.into(),
			orders
		));

		let event_record: frame_system::EventRecord<_, _> = System::events().pop().unwrap();
		println!("Events: {:?}", event_record);

		System::assert_has_event(Event::OrderError { order_id: 204, error_code: 524 }.into());
	});
}

#[test]
#[should_panic(expected = "TradeBatchError")]
// taker order with side and direction that does not match with the maker
fn it_produces_error_for_taker_when_side_and_direction_is_invalid() {
	new_test_ext().execute_with(|| {
		let (markets, accounts, private_keys) = setup();
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		let account_id_1: U256 = get_trading_account_id(accounts.clone(), 0);
		let account_id_4: U256 = get_trading_account_id(accounts.clone(), 3);

		let order_1 = Order {
			account_id: account_id_1,
			order_id: ORDER_ID_1,
			market_id: markets[0].id,
			order_type: OrderType::Limit,
			direction: Direction::Short,
			side: Side::Buy,
			price: 105.into(),
			size: 1.into(),
			leverage: 1.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};
		let order_4 = Order {
			account_id: account_id_4,
			order_id: ORDER_ID_4,
			market_id: markets[0].id,
			order_type: OrderType::Market,
			direction: Direction::Long,
			side: Side::Buy,
			price: 100.into(),
			size: 3.into(),
			leverage: 1.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};

		let order_1 = sign_order(order_1, private_keys[0]);
		let order_4 = sign_order(order_4, private_keys[3]);
		let orders: Vec<Order> = vec![order_1, order_4];

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			U256::from(1_u8),
			3.into(),
			markets[0].id,
			100.into(),
			orders
		));

		let order_1 = Order {
			account_id: account_id_1,
			order_id: ORDER_ID_1,
			market_id: markets[0].id,
			order_type: OrderType::Limit,
			direction: Direction::Short,
			side: Side::Buy,
			price: 105.into(),
			size: 1.into(),
			leverage: 1.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};
		let order_4 = Order {
			account_id: account_id_4,
			order_id: ORDER_ID_4,
			market_id: markets[0].id,
			order_type: OrderType::Market,
			direction: Direction::Long,
			side: Side::Sell,
			price: 100.into(),
			size: 3.into(),
			leverage: 1.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};

		let order_1 = sign_order(order_1, private_keys[0]);
		let order_4 = sign_order(order_4, private_keys[3]);
		let orders: Vec<Order> = vec![order_1, order_4];

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			U256::from(2_u8),
			3.into(),
			markets[0].id,
			100.into(),
			orders
		));

		let event_record: frame_system::EventRecord<_, _> = System::events().pop().unwrap();
		println!("Events: {:?}", event_record);

		System::assert_has_event(Event::OrderError { order_id: 203, error_code: 511 }.into());
	});
}

#[test]
#[should_panic(expected = "TradeBatchError")]
// Taker long buy limit order execution price is invalid
fn it_produces_error_when_taker_long_buy_limit_price_invalid() {
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
			direction: Direction::Short,
			side: Side::Buy,
			price: 100.into(),
			size: 1.into(),
			leverage: 8.into(),
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
			order_type: OrderType::Limit,
			direction: Direction::Long,
			side: Side::Buy,
			price: 99.into(),
			size: 1.into(),
			leverage: 1.into(),
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
			1.into(),
			markets[0].id,
			100.into(),
			orders
		));

		System::assert_has_event(Event::OrderError { order_id: 201, error_code: 508 }.into());
	});
}

#[test]
#[should_panic(expected = "TradeBatchError")]
// Taker short buy limit order execution price is invalid
fn it_produces_error_when_taker_short_buy_limit_price_invalid() {
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
			price: 100.into(),
			size: 1.into(),
			leverage: 8.into(),
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
			order_type: OrderType::Limit,
			direction: Direction::Short,
			side: Side::Buy,
			price: 101.into(),
			size: 1.into(),
			leverage: 1.into(),
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
			1.into(),
			markets[0].id,
			100.into(),
			orders
		));

		System::assert_has_event(Event::OrderError { order_id: 201, error_code: 507 }.into());
	});
}

#[test]
#[should_panic(expected = "TradeBatchError")]
// Taker long buy slippage check
fn it_produces_error_when_taker_long_buy_price_not_within_slippage() {
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
			direction: Direction::Short,
			side: Side::Buy,
			price: 111.into(),
			size: 1.into(),
			leverage: 8.into(),
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
			direction: Direction::Long,
			side: Side::Buy,
			price: 100.into(),
			size: 1.into(),
			leverage: 1.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};

		let order_1 = sign_order(order_1, private_keys[0]);
		let order_2 = sign_order(order_2, private_keys[1]);
		let orders: Vec<Order> = vec![order_1, order_2.clone()];

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			U256::from(1_u8),
			1.into(),
			markets[0].id,
			100.into(),
			orders
		));
	});
}

#[test]
// Taker long buy slippage check when execution price very low
fn it_works_when_taker_long_buy_price_very_low() {
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
			direction: Direction::Short,
			side: Side::Buy,
			price: 80.into(),
			size: 1.into(),
			leverage: 8.into(),
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
			direction: Direction::Long,
			side: Side::Buy,
			price: 100.into(),
			size: 1.into(),
			leverage: 1.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};

		let order_1 = sign_order(order_1, private_keys[0]);
		let order_2 = sign_order(order_2, private_keys[1]);
		let orders: Vec<Order> = vec![order_1, order_2.clone()];

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			U256::from(1_u8),
			1.into(),
			markets[0].id,
			100.into(),
			orders
		));
	});
}

#[test]
fn test_fee_while_opening_order() {
	new_test_ext().execute_with(|| {
		let (fee_tiers, fee_details, discount_tiers, discount_details) = setup_fee();
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		let side: Side = Side::Buy;
		// Dispatch a signed extrinsic.
		assert_ok!(TradingFees::update_base_fees_and_discounts(
			RuntimeOrigin::signed(1),
			side,
			fee_tiers,
			fee_details.clone(),
			discount_tiers,
			discount_details.clone()
		));

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
			price: 100.into(),
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
			price: 100.into(),
			size: 1.into(),
			leverage: 1.into(),
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
			1.into(),
			markets[0].id,
			100.into(),
			orders
		));

		let usdc_id: u128 = 93816115890698;
		let balance_1 = TradingAccounts::balances(account_id_1, usdc_id);
		assert_eq!(balance_1, FixedI128::from_inner(9998060000000000000000));
		let balance_2 = TradingAccounts::balances(account_id_2, usdc_id);
		assert_eq!(balance_2, FixedI128::from_inner(9995150000000000000000));

		// Close orders
		// Since we are closing orders without setting the fee for close orders, fee won't be deducted from balance
		let order_3 = Order {
			account_id: account_id_1,
			order_id: ORDER_ID_3,
			market_id: markets[0].id,
			order_type: OrderType::Limit,
			direction: Direction::Long,
			side: Side::Sell,
			price: 105.into(),
			size: 1.into(),
			leverage: 1.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};
		let order_4 = Order {
			account_id: account_id_2,
			order_id: ORDER_ID_4,
			market_id: markets[0].id,
			order_type: OrderType::Market,
			direction: Direction::Short,
			side: Side::Sell,
			price: 100.into(),
			size: 1.into(),
			leverage: 1.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};

		let order_3 = sign_order(order_3, private_keys[0]);
		let order_4 = sign_order(order_4, private_keys[1]);
		let orders: Vec<Order> = vec![order_3, order_4];

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			U256::from(2_u8),
			1.into(),
			markets[0].id,
			105.into(),
			orders
		));

		let usdc_id: u128 = 93816115890698;
		let balance_1 = TradingAccounts::balances(account_id_1, usdc_id);
		assert_eq!(balance_1, FixedI128::from_inner(10003060000000000000000));
		let balance_2 = TradingAccounts::balances(account_id_2, usdc_id);
		assert_eq!(balance_2, FixedI128::from_inner(9990150000000000000000));
		let locked_1 = TradingAccounts::locked_margin(account_id_1, usdc_id);
		assert_eq!(locked_1, 0.into());
	});
}

#[test]
fn test_fee_while_closing_order() {
	new_test_ext().execute_with(|| {
		let (fee_tiers, fee_details, discount_tiers, discount_details) = setup_fee();
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		let side: Side = Side::Sell;
		// Dispatch a signed extrinsic.
		assert_ok!(TradingFees::update_base_fees_and_discounts(
			RuntimeOrigin::signed(1),
			side,
			fee_tiers,
			fee_details.clone(),
			discount_tiers,
			discount_details.clone()
		));

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
			price: 100.into(),
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
			price: 100.into(),
			size: 1.into(),
			leverage: 1.into(),
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
			1.into(),
			markets[0].id,
			100.into(),
			orders
		));

		// Since we are opening orders without setting the fee for open orders, fee won't be deducted from balance
		let usdc_id: u128 = 93816115890698;
		let balance_1 = TradingAccounts::balances(account_id_1, usdc_id);
		assert_eq!(balance_1, 10000.into());
		let balance_2 = TradingAccounts::balances(account_id_2, usdc_id);
		assert_eq!(balance_2, 10000.into());

		// Close orders
		let order_3 = Order {
			account_id: account_id_1,
			order_id: ORDER_ID_3,
			market_id: markets[0].id,
			order_type: OrderType::Limit,
			direction: Direction::Long,
			side: Side::Sell,
			price: 105.into(),
			size: 1.into(),
			leverage: 1.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};
		let order_4 = Order {
			account_id: account_id_2,
			order_id: ORDER_ID_4,
			market_id: markets[0].id,
			order_type: OrderType::Market,
			direction: Direction::Short,
			side: Side::Sell,
			price: 100.into(),
			size: 1.into(),
			leverage: 1.into(),
			slippage: FixedI128::from_inner(100000000000000000),
			post_only: false,
			time_in_force: TimeInForce::GTC,
			sig_r: 0.into(),
			sig_s: 0.into(),
			hash_type: HashType::Pedersen,
		};

		let order_3 = sign_order(order_3, private_keys[0]);
		let order_4 = sign_order(order_4, private_keys[1]);
		let orders: Vec<Order> = vec![order_3, order_4];

		assert_ok!(Trading::execute_trade(
			RuntimeOrigin::signed(1),
			U256::from(2_u8),
			1.into(),
			markets[0].id,
			105.into(),
			orders
		));

		let usdc_id: u128 = 93816115890698;
		let balance_1 = TradingAccounts::balances(account_id_1, usdc_id);
		assert_eq!(balance_1, FixedI128::from_inner(10002963000000000000000));
		let balance_2 = TradingAccounts::balances(account_id_2, usdc_id);
		assert_eq!(balance_2, FixedI128::from_inner(9990392500000000000000));
		let locked_1 = TradingAccounts::locked_margin(account_id_1, usdc_id);
		assert_eq!(locked_1, 0.into());
	});
}
