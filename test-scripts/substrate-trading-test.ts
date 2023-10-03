
import { nanoid } from 'nanoid';
import { randomUUID } from 'crypto';
import { BigNumber } from 'bignumber.js';

BigNumber.set({
  EXPONENTIAL_AT: 25,
});

function generateHexId (): string {
  const uuid = randomUUID().split('-').join('');
  return `0x${uuid}`;
}

import { TypeRegistry } from '@polkadot/types';
import { ApiPromise, WsProvider } from '@polkadot/api';
import { blake2AsU8a, cryptoWaitReady } from '@polkadot/util-crypto';
import { Keyring } from '@polkadot/keyring';

import {
  I128,
  U128,
  U256,
} from '@polkadot/types-codec';

import {
  BN,
  hexToBn,
  hexToU8a,
  hexToString,
  stringToU8a,
  stringToHex,
  u8aToHex,
  u8aToString,
  u8aToNumber,
  u8aToFloat,
  u8aToBn,
  numberToHex,
  numberToU8a,
  compactStripLength,
  bnToHex,
  bnToU8a,
} from '@polkadot/util';

import * as baseStarknet from 'starknet';
const { encodeShortString, decodeShortString } = baseStarknet.shortString;

import { StarknetHelper } from './starknet.helper';
import { StarknetTestHelper } from './starknet-test.helper';
import { SubstrateHelper } from './substrate.helper';

const NODE_ACCOUNT = '//Alice';

function convertHexToU256 (value: string): U256 {
  BigNumber.set({
    EXPONENTIAL_AT: 100,
    DECIMAL_PLACES: 100,
  });

  const registry = new TypeRegistry();
  const valueAsBigNumber = hexToBn(value);
  const valueAsBigNumberString = valueAsBigNumber.toString();
  return registry.createType('u256', valueAsBigNumberString);
}

function convertStringToU256 (value: string): U256 {
  BigNumber.set({
    EXPONENTIAL_AT: 100,
    DECIMAL_PLACES: 100,
  });

  const registry = new TypeRegistry();
  const valueAsHex = stringToHex(value);
  const valueAsBigNumber = hexToBn(valueAsHex);
  const valueAsBigNumberString = valueAsBigNumber.toString();
  return registry.createType('u256', valueAsBigNumberString);
}

function convertU256ToString (value: any): string {
  const valueAsHex = bnToHex(value);
  const valueAsString = hexToString(valueAsHex);
  return valueAsString;
}

function convertHexToU128(value: string): U128 {
  const registry = new TypeRegistry();
  const valueAsBn = hexToBn(value);
  const valueAsBnString = valueAsBn.toString();

  return registry.createType('u128', valueAsBnString);
}

function convertStringToU128(value: string): U128 {
  const registry = new TypeRegistry();
  const valueAsHex = stringToHex(value);
  const valueAsBn = hexToBn(valueAsHex);
  const valueAsBnString = valueAsBn.toString();

  return registry.createType('u128', valueAsBnString);
}

function convertToFixedI128 (value: number): I128 {
  const registry = new TypeRegistry();
  const multiplyBy = new BigNumber(10).pow(18);
  const result = new BigNumber(value).multipliedBy(multiplyBy);
  return registry.createType('i128', result.toString());
}

function convertFromFixedI128 (value: any): number {
  const multiplyBy = new BigNumber(10).pow(18);
  return new BigNumber(value).dividedBy(multiplyBy).toNumber();
}

function getOrderSignature(params: {
  privateKey: string;
  order: any;
}): [string, string] {
  // params.privateKey = '0x02460f89d7bfd056055946ec8eed5420fde518a4829d0bd4cd6f25dfea2b4fc8';
  const { privateKey, order } = params;
  // params.order = {
  //   account_id: convertHexToU256(account1.id),
  //   order_id: encodeShortString(nanoid(16)),
  //   market_id: ethUsdcMarket.id,
  //   order_type: 0, // LIMIT
  //   direction: 0, // LONG
  //   side: 0, // BUY
  //   price: convertToFixedI128(1830),
  //   size: ethUsdcMarket.step_size,
  //   leverage: convertToFixedI128(1),
  //   slippage: convertToFixedI128(.1),
  //   postOnly: false,
  //   timeInForce: 0,
  // };

  const accountIdAsBytes = bnToU8a(order.account_id.toPrimitive(), { isLe: false });
  // const accountIdAsBytes = bnToU8a(BigInt('48051126807621704249374145463538098412152070912329342346721017172303341278455'), { isLe: false });
  const accountIdAsLowBytes = accountIdAsBytes.slice(16);
  const accountIdAsHighBytes = accountIdAsBytes.slice(0, 16);

  // accountIdAsLowBytes.set(accountIdAsBeBytes, 16);
  // accountIdAsHighBytes.set(accountIdAsBeBytes, 0);

  const orderHashElements = [
    u8aToBn(accountIdAsLowBytes, { isLe: false }).toString(),
    u8aToBn(accountIdAsHighBytes, { isLe: false }).toString(),
    order.order_id.toPrimitive(),
    // '105525528769446872544116910375562659185',
    order.market_id.toPrimitive(),
    order.order_type,
    order.direction,
    order.side,
    order.price.toPrimitive(),
    order.size.toPrimitive(),
    order.leverage.toPrimitive(),
    order.slippage.toPrimitive(),
    order.post_only ? 1 : 0,
    order.time_in_force,

    // '100',
    // '200',
    // '300',
    // 1,
    // 0,
    // 0,
    // 10000000,
    // 1,
    // '100',
    // '200',
    // 1,
    // 0,

    // orderDirectionToL2Hex[orderRequest.direction],
    // `0x${to64x61(orderRequest.price, marketTickPrecision).toString(16)}`,
    // `0x${to64x61(orderRequest.quantity, marketStepPrecision).toString(16)}`,
    // `0x${to64x61(orderRequest.leverage ?? 0, 2).toString(16)}`, // [0.00, 10.00]
    // `0x${to64x61(orderRequest.slippage ?? 0, 2).toString(16)}`, // [0.00, 1.00]
    // orderTypeToL2Hex[orderRequest.type],
    // orderTimeInForceToL2Hex[orderRequest.timeInForce],
    // mapOrderPostOnlyToL2Hex(orderRequest.postOnly),
    // orderSideToL2Hex[orderRequest.side],
  ];

  const orderHash = baseStarknet.hash.computeHashOnElements(orderHashElements);

  // const keyPair = baseStarknet.ec..getKeyPair(options.privateKeyL2);
  const signature = baseStarknet.ec.starkCurve.sign(orderHash, privateKey);

  // const publicKey = baseStarknet.ec.starkCurve.getPublicKey(privateKey);
  // const isVerified = baseStarknet.ec.starkCurve.verify(signature, orderHash, publicKey);

  const key1 = baseStarknet.num.toHex(signature.r);
  const key2 = baseStarknet.num.toHex(signature.s);

  return [key1, key2];
}

function calculateAccountId(params: {
  accountIndex: number; // must be between [0, 255]
  accountAddress: string; // must be in hex
}): string {
  let accountAddress = params.accountAddress;

  if (accountAddress.includes('0x')) {
    accountAddress = accountAddress.substring(2);
  }

  // ex: "0x1ab" should be "0x01ab"
  if (accountAddress.length % 2 !== 0) {
    accountAddress = `0${accountAddress}`;
  }

  const accountIndexLeBytes = numberToU8a(params.accountIndex);
  const accountAddressLeBytes = hexToU8a(accountAddress).reverse();

  // just to be sure
  if (accountAddressLeBytes.length > 32) {
    throw new Error('accountAddress should be <= 32 bytes');
  }

  const concatenatedBytes = new Uint8Array(33);

  Buffer.from(accountAddressLeBytes).copy(concatenatedBytes, 0);
  Buffer.from(accountIndexLeBytes).copy(concatenatedBytes, 32);

  const hashAsBytes = blake2AsU8a(concatenatedBytes, 256);
  const hashAsString = u8aToHex(hashAsBytes);

  return hashAsString;
}

process.nextTick(async () => {
  await cryptoWaitReady();

  // Construct the keyring after the API (crypto has an async init)
  const keyring = new Keyring({ type: 'sr25519' });

  // Add Alice to our keyring with a hard-derivation path (empty phrase, so uses dev)
  const nodeAccountKeyring = keyring.addFromUri(NODE_ACCOUNT);

  const wsProvider = new WsProvider('ws://127.0.0.1:9944');
  const api = await ApiPromise.create({
    provider: wsProvider,
    // rpc: {
    //   trading: {
    //     get_positions: {
    //       description: 'Just a test method',
    //       type: 'Vec<Position>',
    //       params: [
    //         {
    //           name: 'at',
    //           type: 'Hash',
    //           isOptional: true,
    //         },
    //         {
    //           name: 'account_id',
    //           type: 'u256',
    //         },
    //         {
    //           name: 'collateral_id',
    //           type: 'u256',
    //         },
    //       ],
    //     },
    //   },
    // },
  });

  // const rpcAsAny = api.rpc as any;
  // const rpcTradingGetPositions = await rpcAsAny.trading.get_positions(
  //   '0xec320217bbc66a3418487febdceacb583f0fb51531013eb8176a51f169266f2a',
  //   convertHexToU256('0x01234'),
  //   convertHexToU256('0x01234'),
  // );









  // const tradingMethods = api.rpc?.['trading'] as any;
  // rpcMethods.methods.

  const assetsMap = await api.query.assets.assetMap;
  const assetsMapEntries = (await assetsMap.entries()).map((kek) => {
    const obj = kek[1].toPrimitive() as any;
    obj.id = convertU256ToString(obj.id);
    return obj;
  });

  const marketsMap = await api.query.markets.marketMap;
  const marketsMapEntries = (await marketsMap.entries()).map((kek) => {
    const obj = kek[1].toPrimitive() as any;
    obj.id = convertU256ToString(obj.id);
    obj.asset = convertU256ToString(obj.asset);
    obj.assetCollateral = convertU256ToString(obj.assetCollateral);
    obj.tickSize = convertFromFixedI128(obj.tickSize);
    obj.stepSize = convertFromFixedI128(obj.stepSize);
    obj.minimumOrderSize = convertFromFixedI128(obj.minimumOrderSize);
    obj.minimumLeverage = convertFromFixedI128(obj.minimumLeverage);
    return obj;
  });

  const positionsMap = await api.query.trading.positionsMap;
  const positionsMapEntries = (await positionsMap.entries()).map((kek) => {
    const obj0 = kek[0].toJSON() as any;
    const obj1 = kek[1].toPrimitive() as any;
    obj1.avgExecutionPrice = convertFromFixedI128(obj1.avgExecutionPrice);
    obj1.size = convertFromFixedI128(obj1.size_);
    return obj1;
  });

  const ethAsset = {
    id: convertStringToU128('ETH'),
    name: stringToHex('ETH'),
    is_tradable: true,
    is_collateral: false,
    token_decimal: 6,
  };

  const usdcAsset = {
    id: convertStringToU128('USDC'),
    name: stringToHex('USDC'),
    is_tradable: false,
    is_collateral: true,
    token_decimal: 6,
  };

  const ethUsdcMarket = {
    id: convertStringToU128('ETH-USDC'),
    asset: convertStringToU128('ETH'),
    asset_collateral: convertStringToU128('USDC'),
    is_tradable: 1,
    is_archived: false,
    ttl: 3600,
    tick_size: convertToFixedI128(0.1),
    tick_precision: 0,
    step_size: convertToFixedI128(0.01),
    step_precision: 0,
    minimum_order_size: convertToFixedI128(0.01),
    minimum_leverage: convertToFixedI128(1),
    maximum_leverage: convertToFixedI128(20),
    currently_allowed_leverage: convertToFixedI128(20),
    maintenance_margin_fraction: convertToFixedI128(0.03),
    initial_margin_fraction: convertToFixedI128(0.05),
    incremental_initial_margin_fraction: convertToFixedI128(0.01),
    incremental_position_size: convertToFixedI128(100),
    baseline_position_size: convertToFixedI128(500),
    maximum_position_size: convertToFixedI128(10000),
  };

  const nonce1 = await api.rpc.system.accountNextIndex(nodeAccountKeyring.address);
  const replaceAllAssetsResult = await api.tx.assets.replaceAllAssets([
    ethAsset,
    usdcAsset,
  ]).signAndSend(nodeAccountKeyring, {
    nonce: nonce1,
  });

  await new Promise((resolve) => setTimeout(resolve, 7000));
  const nonce2 = await api.rpc.system.accountNextIndex(nodeAccountKeyring.address);
  const replaceAllMarketsResult = await api.tx.markets.replaceAllMarkets([
    ethUsdcMarket,
  ]).signAndSend(nodeAccountKeyring, {
    nonce: nonce2,
  });

  const replaceAllAssetsResultAsHex = replaceAllAssetsResult.toHex();
  const replaceAllMarketsResultAsHex = replaceAllMarketsResult.toHex();
  console.log('replaceAllAssetsResultAsHex', replaceAllAssetsResultAsHex);
  console.log('replaceAllMarketsResultAsHex', replaceAllMarketsResultAsHex);

  const starknetAccount1 = StarknetTestHelper.generateKeys();
  const starknetAccount2 = StarknetTestHelper.generateKeys();

  const account1 = {
    id: '',
    index: 0,
    address: generateHexId(),
    pubKey: starknetAccount1.starkKey,
  };

  const account2 = {
    id: '',
    index: 0,
    address: generateHexId(),
    pubKey: starknetAccount2.starkKey,
  };

  const nonceForUpdatingBalance = await api.rpc.system.accountNextIndex(nodeAccountKeyring.address);
  
  const balanceUpdateResult = await api.tx.zkxTradingAccount.deposit(
    account2.address,// account_address: U256,
		account2.index,// 	index: u8,
		account2.pubKey,// 	pub_key: U256,
		usdcAsset.id, // 	collateral_id: u128,
		convertToFixedI128(100)// 	amount: FixedI128,
    // batchId,
    // quantityLocked,
    // marketId,
    // oraclePrice,
    // [
    //   order1,
    //   order2,
    // ]
    ).signAndSend(nodeAccountKeyring, {
      nonce: nonceForUpdatingBalance,
    });

  const balanceUpdateAsHexResult = balanceUpdateResult.toHex();
  console.log('balanceUpdateAsHexResult', balanceUpdateAsHexResult);

  // const account3 = {
  //   id: '',
  //   index: 0,
  //   address: generateHexId(),
  //   pubKey: generateHexId(),
  // };

  for await (const account of [account1, account2]) {
    account.id = calculateAccountId({
      accountIndex: account.index,
      accountAddress: account.address,
    });

    await new Promise((resolve) => setTimeout(resolve, 7000));

    const accountAddressAsHex = account.address;
    const accountPubKeyAsHex = account.pubKey;
    console.log('accountPubKeyAsHex:', accountPubKeyAsHex);
    const nonceToCreateAccount = await api.rpc.system.accountNextIndex(nodeAccountKeyring.address);
    const addAccountsResult = await api.tx.zkxTradingAccount.addAccounts([
      {
        index: account.index,
        account_address: convertHexToU256(accountAddressAsHex),
        pub_key: convertHexToU256(accountPubKeyAsHex),
      },
    ]).signAndSend(nodeAccountKeyring, {
      nonce: nonceToCreateAccount,
    });

    const addAccountsResultAsHex = addAccountsResult.toHex();
    console.log('addAccountsResultAsHex:', addAccountsResultAsHex);

    // await new Promise((resolve) => setTimeout(resolve, 10000));
    // const nonceToAddBalance = await api.rpc.system.accountNextIndex(nodeAccountKeyring.address);

    // let balanceValueI128 = convertToFixedI128(10000);

    // if (account.address !== account1.address) {
    //   balanceValueI128 = convertToFixedI128(-10000);
    // }

    // const addBalanceResult = await api.tx.zkxTradingAccount.addBalances(
    //   convertHexToU256(account.id),
    //   [
    //     {
    //       asset_id: encodeShortString('USDC'),
    //       balance_value: balanceValueI128,
    //     },
    //   ]).signAndSend(nodeAccountKeyring, {
    //   nonce: nonceToAddBalance,
    // });
    //
    // const addBalanceResultAsHex = addBalanceResult.toHex();
    // console.log('addBalanceResultAsHex', addBalanceResultAsHex);
  }

  // =============================================
  // =============================================
  // =============================================
  // =============================================

  const order1: any = {
    account_id: convertHexToU256(account1.id),
    order_id: SubstrateHelper.convertStringToU128(nanoid(16)),
    market_id: ethUsdcMarket.id,
    order_type: 0, // LIMIT
    direction: 0, // LONG
    side: 0, // BUY
    price: convertToFixedI128(1830),
    size: ethUsdcMarket.step_size,
    leverage: convertToFixedI128(1),
    slippage: convertToFixedI128(.1),
    post_only: false,
    time_in_force: 0,
    hash_type: 0,
    sig_r: null,
    sig_s: null,
  };

  const order2: any = {
    account_id: convertHexToU256(account2.id),
    order_id: SubstrateHelper.convertStringToU128(nanoid(16)),
    market_id: ethUsdcMarket.id,
    order_type: 1, // MARKET
    direction: 1, // SHORT
    side: 0, // BUY
    price: convertToFixedI128(1830),
    size: ethUsdcMarket.step_size,
    leverage: convertToFixedI128(1),
    slippage: convertToFixedI128(.1),
    post_only: false,
    time_in_force: 0,
    hash_type: 0,
    sig_r: null,
    sig_s: null,
  };

  // ==========
  // ==========
  // ==========

  // const order3 = {
  //   account_id: convertHexToU256(account2.id),
  //   order_id: encodeShortString(nanoid(16)),
  //   market_id: ethUsdcMarket.id,
  //   order_type: 1, // MARKET
  //   direction: 1, // SHORT
  //   side: 0, // BUY
  //   price: convertToFixedI128(1830),
  //   size: ethUsdcMarket.step_size,
  //   leverage: convertToFixedI128(1),
  //   slippage: convertToFixedI128(.1),
  //   postOnly: false,
  //   timeInForce: 0,
  // };

  // console.log(JSON.stringify({
  //   ...order1,
  //   price: ethUsdcMarket.tick_size.toPrimitive(),
  //   size: ethUsdcMarket.step_size.toPrimitive(),
  // }, null, 2));
  //
  // console.log(JSON.stringify({
  //   ...order2,
  //   price: ethUsdcMarket.tick_size.toPrimitive(),
  //   size: ethUsdcMarket.step_size.toPrimitive(),
  // }, null, 2));

  await new Promise((resolve) => setTimeout(resolve, 7000));

  const signature1 = getOrderSignature({
    privateKey: starknetAccount1.privateKey,
    order: order1,
  });

  const signature2 = getOrderSignature({
    privateKey: starknetAccount2.privateKey,
    order: order2,
  });

  order1.sig_r = convertHexToU256(signature1[0]);
  order1.sig_s = convertHexToU256(signature1[1]);

  order2.sig_r = convertHexToU256(signature2[0]);
  order2.sig_s = convertHexToU256(signature2[1]);

  const nonceForTrade = await api.rpc.system.accountNextIndex(nodeAccountKeyring.address);
  const batchId = encodeShortString(nanoid());
  // const batchId = encodeShortString('1234abcd');
  const quantityLocked = ethUsdcMarket.step_size;
  const marketId = ethUsdcMarket.id;
  const oraclePrice = convertToFixedI128(1830);
  const executeTradeResult = await api.tx.trading.executeTrade(
    batchId,
    quantityLocked,
    marketId,
    oraclePrice,
    [
      order1,
      order2,
    ]).signAndSend(nodeAccountKeyring, {
      nonce: nonceForTrade,
    });

  const executeTradeResultAsHex = executeTradeResult.toHex();
  console.log('account id: ', convertHexToU256(account1.id));
  console.log('executeTradeResultAsHex', executeTradeResultAsHex);

  console.log('...');
});
