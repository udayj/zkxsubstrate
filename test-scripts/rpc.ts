import { BigNumber } from 'bignumber.js';
BigNumber.set({
  EXPONENTIAL_AT: 25,
});
import { ApiPromise, WsProvider } from '@polkadot/api';
import { cryptoWaitReady } from '@polkadot/util-crypto';

process.nextTick(async () => {
  await cryptoWaitReady();
  // Construct the keyring after the API (crypto has an async init)
  // Add Alice to our keyring with a hard-derivation path (empty phrase, so uses dev)
  const wsProvider = new WsProvider('ws://127.0.0.1:9944');
  const api = await ApiPromise.create({
    provider: wsProvider,
    rpc: {
      trading: {
        get_positions: {
          description: 'Just a test method',
          type: 'Vec<Position>',
          params: [
            {
              name: 'account_id',
              type: 'u256',
            },
            {
              name: 'collateral_id',
              type: 'u256',
            },
          ],
        },
      },
    },
  });
  const res = await api.rpc.rpc.methods();
  console.log({ res: res.toPrimitive() });

  const response = await (api.rpc as any).trading.get_positions(
    // Please replace with actual values
    '0x19208b719cd133aed3e0cf6ac688becd831733a4bac44b1d78f4cf813f615c90',
    '0x55534443',
  );

  console.log({ response });
});