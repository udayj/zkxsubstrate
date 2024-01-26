import { ApiPromise } from "@polkadot/api";
import { KeyringPair } from "@polkadot/keyring/types";

export async function initAbrTimestamp(params: {
  api: ApiPromise;
  account: KeyringPair;
}) {
  const { api, account } = params;

  const currentTime = Date.now();

  const lastTimestamp = await getLastTimestamp();

  if (lastTimestamp !== 0) {
    console.log("Timestamp already initialized");
    process.exit(0);
  }

  const nextTimestamp = (await getNextTimestamp()) * 1000;

  const timestampMs = currentTime - (currentTime % nextTimestamp);

  console.dir({
    lastTimestamp,
    nextTimestamp,
    timestampMs,
  });

  await setInitializationTimestamp({
    timestampMs,
  });

  function getError(dispatchError: any) {
    if (dispatchError.isModule) {
      const decoded = api.registry.findMetaError(dispatchError.asModule);

      return JSON.parse(JSON.stringify(decoded));
    } else {
      return dispatchError.toPrimitive();
    }
  }

  async function getNextTimestamp(): Promise<number> {
    const rpc = api.rpc as any;

    const raw = await rpc.abr["get_next_timestamp"]();

    return raw.toPrimitive();
  }

  async function getLastTimestamp(): Promise<number> {
    const rpc = api.rpc as any;

    const raw = await rpc.abr["get_last_timestamp"]();

    return raw.toPrimitive();
  }

  async function setInitializationTimestamp(params: { timestampMs: number }) {
    const { timestampMs } = params;

    const nonce = await api.rpc.system.accountNextIndex(account.address);

    console.dir({ nonce: nonce.toPrimitive() });

    return new Promise<void>((resolve, reject) => {
      api.tx.sudo
        .sudo(api.tx.prices.setInitialisationTimestamp(timestampMs))
        .signAndSend(
          account,
          {
            nonce,
          },
          (result) => {
            const { dispatchError, isInBlock } = result;

            if (dispatchError) {
              reject(getError(dispatchError));
            }

            if (isInBlock) {
              resolve();
            }
          }
        );
    });
  }
}
