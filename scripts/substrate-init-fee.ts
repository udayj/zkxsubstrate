import { SubstrateHelper } from "./substrate.helper";
import { ApiPromise, WsProvider } from "@polkadot/api";
import { cryptoWaitReady } from "@polkadot/util-crypto";
import { Keyring } from "@polkadot/keyring";

enum Side {
  Buy,
  Sell,
}
enum OrderSide {
  Maker,
  Taker,
}
(async () => {
  await cryptoWaitReady();
  const NODE_ACCOUNT = "//Alice";

  const keyring = new Keyring({ type: "sr25519" });

  const nodeAccountKeyring = keyring.addFromUri(NODE_ACCOUNT);

  const wsProvider = new WsProvider("ws://127.0.0.1:9944");
  // const wsProvider = new WsProvider("wss://l3.stand-1.k8s.ntwrkx.com:443");
  // const wsProvider = new WsProvider("wss://l3.stand-2.k8s.ntwrkx.com:443");
  // const wsProvider = new WsProvider("wss://l3.sandbox.zkx.fi/");

  const api = await ApiPromise.create({
    provider: wsProvider,
  });

  await updateBaseFees({
    collateralId: "USDC",
    orderSide: OrderSide.Taker,
    side: Side.Buy,
    fee: [
      {
        volume: 0,
        fee: 0.001,
      },
      {
        volume: 1_000_000,
        fee: 0.0008,
      },
      {
        volume: 5_000_000,
        fee: 0.0005,
      },
      {
        volume: 10_000_000,
        fee: 0.0004,
      },
      {
        volume: 50_000_000,
        fee: 0.0002,
      },
    ],
  });

  await updateBaseFees({
    collateralId: "USDC",
    orderSide: OrderSide.Taker,
    side: Side.Sell,
    fee: [
      {
        volume: 0,
        fee: 0.001,
      },
      {
        volume: 1_000_000,
        fee: 0.0008,
      },
      {
        volume: 5_000_000,
        fee: 0.0005,
      },
      {
        volume: 10_000_000,
        fee: 0.0004,
      },
      {
        volume: 50_000_000,
        fee: 0.0002,
      }
    ],
  });

  await updateBaseFees({
    collateralId: "USDC",
    orderSide: OrderSide.Maker,
    side: Side.Buy,
    fee: [
      {
        volume: 0,
        fee: 0.001,
      },
      {
        volume: 1_000_000,
        fee: 0.0005,
      },
      {
        volume: 5_000_000,
        fee: 0.0002,
      },
      {
        volume: 10_000_000,
        fee: 0.0001,
      },
      {
        volume: 50_000_000,
        fee: 0,
      },
    ],
  });

  await updateBaseFees({
    collateralId: "USDC",
    orderSide: OrderSide.Maker,
    side: Side.Sell,
    fee: [
      {
        volume: 0,
        fee: 0.001,
      },
      {
        volume: 1_000_000,
        fee: 0.0005,
      },
      {
        volume: 5_000_000,
        fee: 0.0002,
      },
      {
        volume: 10_000_000,
        fee: 0.0001,
      },
      {
        volume: 50_000_000,
        fee: 0,
      },
    ],
  });

  await api.disconnect();

  function getError(dispatchError: any) {
    if (dispatchError.isModule) {
      const decoded = api.registry.findMetaError(dispatchError.asModule);

      return JSON.parse(JSON.stringify(decoded));
    } else {
      return dispatchError.toPrimitive();
    }
  }

  async function updateBaseFees(params: {
    collateralId: string;
    side: Side;
    orderSide: OrderSide;
    fee: { volume: number; fee: number }[];
  }) {
    const { collateralId, side, orderSide, fee } = params;
    return new Promise<void>(async (resolve, reject) => {
      const nonce = await api.rpc.system.accountNextIndex(
        nodeAccountKeyring.address,
      );
      try {
        await api.tx.tradingFees
          .updateBaseFees(
            SubstrateHelper.convertStringToU128(collateralId),
            side,
            orderSide,
            fee.map(({ fee, volume }) => ({
              volume: SubstrateHelper.convertNumberToI128(volume),
              fee: SubstrateHelper.convertNumberToI128(fee),
            })),
          )
          .signAndSend(
            nodeAccountKeyring,
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
            },
          );
      } catch (e) {
        reject(e);
      }
    });
  }
})().catch(console.error);
