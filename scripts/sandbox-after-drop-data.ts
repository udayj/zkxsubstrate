import { BigNumber } from 'bignumber.js';

BigNumber.set({
  EXPONENTIAL_AT: 25,
});

import { ApiPromise, WsProvider } from '@polkadot/api';
import { cryptoWaitReady } from '@polkadot/util-crypto';
import { Keyring } from '@polkadot/keyring';

const NODE_ACCOUNT = '//Alice';

process.nextTick(async () => {
  await cryptoWaitReady();

  const keyring = new Keyring({ type: 'sr25519' });
  const nodeAccountKeyring = keyring.addFromUri(NODE_ACCOUNT);
  const wsProvider = new WsProvider('wss://l3.sandbox.k8s.ntwrkx.com:443');
  
  const api = await ApiPromise.create({
    provider: wsProvider,
  });

  await new Promise((resolve) => setTimeout(resolve, 4000));

  const nonce = await api.rpc.system.accountNextIndex(nodeAccountKeyring.address);
  
  const setMatchingTimeLimit = await api.tx.sudo
    .sudo(api.tx.trading.setMatchingTimeLimit(2419200))
    .signAndSend(nodeAccountKeyring, {
      nonce,
    });

  const setMatchingTimeLimitResultAsHex = setMatchingTimeLimit.toHex();
  console.log('setMatchingTimeLimitResultAsHex', setMatchingTimeLimitResultAsHex);
  console.log('...');

  await new Promise((resolve) => setTimeout(resolve, 4000));
  
  process.exit(0);
});