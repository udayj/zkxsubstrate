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
const substrateTypes = {
  Side: {
    _enum: ["BUY", "SELL"],
  },
  Direction: {
    _enum: ["LONG", "SHORT"],
  },
  ForceClosureFlag: {
    _enum: ["DELEVERAGE", "LIQUIDATE"],
  },
  Position: {
    market_id: "u256",
    direction: "Direction",
    side: "Side",
    avg_execution_price: "FixedI128",
    size: "FixedI128",
    margin_amount: "FixedI128",
    borrowed_amount: "FixedI128",
    leverage: "FixedI128",
    realized_pnl: "FixedI128",
  },
  PositionExtended: {
    market_id: "u256",
    direction: "Direction",
    side: "Side",
    avg_execution_price: "FixedI128",
    size: "FixedI128",
    margin_amount: "FixedI128",
    borrowed_amount: "FixedI128",
    leverage: "FixedI128",
    realized_pnl: "FixedI128",
    maintenance_margin: "FixedI128",
    market_price: "FixedI128",
  },
  DeleveragablePosition: {
    market_id: "u256",
    direction: "Direction",
    amount_to_be_sold: "FixedI128",
  },
  AccountInfo: {
    available_margin: "FixedI128",
    total_margin: "FixedI128",
    collateral_balance: "FixedI128",
    force_closure_flag: "Option<ForceClosureFlag>",
    positions: "Vec<PositionExtended>",
    deleveragable_position: "Option<DeleveragablePosition>",
  },
  FeeRates: {
    maker_buy: "FixedI128",
    maker_sell: "FixedI128",
    taker_buy: "FixedI128",
    taker_sell: "FixedI128",
  },
};
const substrateRpcFunctions = {
  trading: {
    get_positions: {
      description: "Get positions rpc method",
      type: "Vec<Position>",
      params: [
        {
          name: "account_id",
          type: "u256",
        },
        {
          name: "collateral_id",
          type: "u128",
        },
        {
          name: "at",
          type: "Hash",
          isOptional: true,
        },
      ],
    },
    get_account_info: {
      description: "Get account info",
      type: "AccountInfo",
      params: [
        {
          name: "account_id",
          type: "u256",
        },
        {
          name: "collateral_id",
          type: "u128",
        },
        {
          name: "at",
          type: "Hash",
          isOptional: true,
        },
      ],
    },
    get_fee: {
      description: "Get fee",
      type: "(FeeRates, u64)",
      params: [
        {
          name: "account_id",
          type: "u256",
        },
        {
          name: "market_id",
          type: "u256",
        },
        {
          name: "at",
          type: "Hash",
          isOptional: true,
        },
      ],
    },
  },
};

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
    types: substrateTypes,
    rpc: substrateRpcFunctions,
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
