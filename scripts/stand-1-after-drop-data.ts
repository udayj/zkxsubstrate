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
  
  const api = await ApiPromise.create({
    provider: wsProvider,
  });

  const timestampCodec = await api.query.prices.initialisationTimestamp();
  const timestampBefore = timestampCodec.toPrimitive();
  console.log({ timestampBefore });

  await new Promise((resolve) => setTimeout(resolve, 4000));

  const nonce = await api.rpc.system.accountNextIndex(nodeAccountKeyring.address);
  
  const currentTimestamp = Date.now();
  console.log({ currentTimestamp });

  const setInitialisationTimestamp = await api.tx.sudo
    .sudo(api.tx.prices.setInitialisationTimestamp(currentTimestamp))
    .signAndSend(nodeAccountKeyring, {
      nonce,
    });

  await new Promise((resolve) => setTimeout(resolve, 4000));

  const setInitialisationTimestampResultAsHex = setInitialisationTimestamp.toHex();
  console.log('setInitialisationTimestampResultAsHex', setInitialisationTimestampResultAsHex);

  await new Promise((resolve) => setTimeout(resolve, 5000));

  const timestampAfterCodec = await api.query.prices.initialisationTimestamp();
  const timestampAfter = timestampAfterCodec.toPrimitive();
  console.log({ timestampAfter });
  console.log('...');
  
  process.exit(0);
});