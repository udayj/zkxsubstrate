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

  const wsProvider = new WsProvider('wss://l3.stand-1.k8s.ntwrkx.com:443');
  // const wsProvider = new WsProvider('ws://zkxl3-main-node.zkxl3-stand-2.svc.cluster.local:9944');
  const api = await ApiPromise.create({
    provider: wsProvider,
  });

  await new Promise((resolve) => setTimeout(resolve, 4000));

  const nonce = await api.rpc.system.accountNextIndex(nodeAccountKeyring.address);
  const current_timestamp = Date.now() / 1000;
  const setInitialisationTimestamp = await api.tx.sudo
    .sudo(api.tx.prices.setInitialisationTimestamp(current_timestamp))
    .signAndSend(nodeAccountKeyring, {
      nonce,
    });

  const setInitialisationTimestampResultAsHex = setInitialisationTimestamp.toHex();
  console.log('setInitialisationTimestampResultAsHex', setInitialisationTimestampResultAsHex);
  console.log('...');
});