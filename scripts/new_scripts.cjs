import * as pkg from 'bignumber.js';
const { BigNumber } = pkg;
BigNumber.set({
  EXPONENTIAL_AT: 25,
});

import { ApiPromise, WsProvider } from '@polkadot/api';
import { cryptoWaitReady } from '@polkadot/util-crypto';
import { Keyring } from '@polkadot/keyring';
import { TypeRegistry } from '@polkadot/types';
import {
    hexToBn,
  } from '@polkadot/util';

const typeRegistry = new TypeRegistry();

function convertHexToU256(value) {
    const valueAsBn = hexToBn(value);
    const valueAsBnString = valueAsBn.toString();
    return typeRegistry.createType('u256', valueAsBnString);
}

process.nextTick(async () => {
  await cryptoWaitReady();

  const seedphrase_hex = ""; // Add here

  const NODE_ACCOUNT = "upon spice cloth armed bitter fiction despair tide rate spice ten spend";
  const keyring = new Keyring({ type: 'sr25519' });
  // Create a key pair from the given public and private keys
  const adminPair = keyring.addFromUri(seedphrase_hex);
  // const adminPair = keyring.addFromUri(NODE_ACCOUNT);
  // const wsProvider = new WsProvider('wss://l3.ntwrkx.com/');
  //const wsProvider = new WsProvider('wss://l3.stand-3.k8s.ntwrkx.com/');
  const wsProvider = new WsProvider("wss://l3.sandbox-2.zkx.fi");
  
  const api = await ApiPromise.create({
    provider: wsProvider,
  });

  const count = await api.query.tradingAccount.accountsCount();
  const accountsCount = count.toString(10);
  console.log("Count: ", accountsCount);

  let monetaryMap = new Map();

  for(let i = 0; i < accountsCount; i++) {
    const tradingAccount = await api.query.tradingAccount.accountsListMap(i);
    const accountDetails = await api.query.tradingAccount.accountMap(tradingAccount.toPrimitive());
    const monetaryAddress = accountDetails.toJSON().accountAddress;
    if (monetaryMap.get(monetaryAddress) === undefined) {
        monetaryMap.set(monetaryAddress, [convertHexToU256(tradingAccount.toJSON())]);
    } else {
        let array = monetaryMap.get(monetaryAddress);
        array.push(convertHexToU256(tradingAccount.toJSON()));
        monetaryMap.set(monetaryAddress, array);
    }
  }

  let input = [];

  for (let key of monetaryMap.keys()) {
    input.push({"monetary_account": convertHexToU256(key), "trading_accounts": monetaryMap.get(key)});
  }

  console.log("Input array: ", input);

  for(let i = 0; i <= input.length; i+=10) {
    const nonce = await api.rpc.system.accountNextIndex(adminPair.address);

    let limit = i + 10;
    if (limit > input.length) { limit = input.length; }
    let subarray = input.slice(i, limit);

    const result = await api.tx.sudo
      .sudo(api.tx.tradingAccount.updateMonetaryToTradingAccounts(subarray))
      .signAndSend(adminPair, {
        nonce,
      });
  }

  console.log('....Completed....');
  process.exit(0);
}); 