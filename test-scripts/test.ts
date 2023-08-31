
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
  numberToHex,
  numberToU8a,
  compactStripLength,
  bnToHex,
} from '@polkadot/util';

import * as baseStarknet from 'starknet';
const { encodeShortString, decodeShortString } = baseStarknet.shortString;

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
  });

  // const account0 = {
  //   id: '',
  //   index: 0,
  //   address: '0x1234abcd',
  //   pubKey: '0x01cfa45e2808e8a2593bbe272ae23a5e175a5c2886d3edaececdbac724f5fd3b',
  // };

  // account0.id = calculateAccountId({
  //   accountIndex: account0.index,
  //   accountAddress: account0.address,
  // });

  // const account = await api.query.zkxTradingAccount.accountMap(
  //   convertHexToU256(account0.id)
  // );

  // const accountData = account.toPrimitive() as any;
  // accountData.accountId = convertU256ToString(accountData.accountId);
  // accountData.accountAddress = convertU256ToString(accountData.accountAddress);
  // accountData.pubKey = convertU256ToString(accountData.pubKey);

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
    id: convertStringToU256('ETH'),
    name: stringToHex('ETH'),
    is_tradable: true,
    is_collateral: false,
    token_decimal: 6,
  };

  const usdcAsset = {
    id: convertStringToU256('USDC'),
    name: stringToHex('USDC'),
    is_tradable: false,
    is_collateral: true,
    token_decimal: 6,
  };

  const ethUsdcMarket = {
    id: convertStringToU256('ETH-USDC'),
    asset: convertStringToU256('ETH'),
    asset_collateral: convertStringToU256('USDC'),
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

  await new Promise((resolve) => setTimeout(resolve, 10000));
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

  const account1 = {
    id: '',
    index: 0,
    address: generateHexId(),
    pubKey: generateHexId(),
  };

  const account2 = {
    id: '',
    index: 0,
    address: generateHexId(),
    pubKey: generateHexId(),
  };

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

    await new Promise((resolve) => setTimeout(resolve, 10000));

    const accountAddressAsHex = account.address;
    const accountPubKeyAsHex = account.pubKey;
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

  const order1 = {
    account_id: convertHexToU256(account1.id),
    order_id: encodeShortString(nanoid(16)),
    market_id: ethUsdcMarket.id,
    order_type: 0, // LIMIT
    direction: 0, // LONG
    side: 0, // BUY
    price: convertToFixedI128(1830),
    size: ethUsdcMarket.step_size,
    leverage: convertToFixedI128(1),
    slippage: convertToFixedI128(.1),
    postOnly: false,
    timeInForce: 0,
  };

  const order2 = {
    account_id: convertHexToU256(account2.id),
    order_id: encodeShortString(nanoid(16)),
    market_id: ethUsdcMarket.id,
    order_type: 1, // MARKET
    direction: 1, // SHORT
    side: 0, // BUY
    price: convertToFixedI128(1830),
    size: ethUsdcMarket.step_size,
    leverage: convertToFixedI128(1),
    slippage: convertToFixedI128(.1),
    postOnly: false,
    timeInForce: 0,
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

  console.log(JSON.stringify({
    ...order1,
    price: ethUsdcMarket.tick_size.toPrimitive(),
    size: ethUsdcMarket.step_size.toPrimitive(),
  }, null, 2));

  console.log(JSON.stringify({
    ...order2,
    price: ethUsdcMarket.tick_size.toPrimitive(),
    size: ethUsdcMarket.step_size.toPrimitive(),
  }, null, 2));

  await new Promise((resolve) => setTimeout(resolve, 10000));
  const nonceForTrade = await api.rpc.system.accountNextIndex(nodeAccountKeyring.address);
  const batchId = encodeShortString(nanoid());
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
  console.log('executeTradeResultAsHex', executeTradeResultAsHex);

  console.log('...');
  console.log(marketsMapEntries);
});