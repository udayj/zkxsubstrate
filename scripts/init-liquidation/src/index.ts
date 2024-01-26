import { ApiPromise, WsProvider } from "@polkadot/api";
import Keyring from "@polkadot/keyring";
import { cryptoWaitReady } from "@polkadot/util-crypto";

import { rpc } from "./rpc";
import { initAbrTimestamp } from "./init-abr-timestamp";
import { addLiquidatorSigner } from "./add-liquidator-signer";

(async () => {
  await cryptoWaitReady();

  const {
    SUBSTRATE_WS_URL = "ws://127.0.0.1:9944",
    NODE_ACCOUNT = "//Alice",
    SIGNERS_PUB_KEYS = "",
  } = process.env;

  const signersPubKeys = SIGNERS_PUB_KEYS.split(",").map((item) => item.trim());

  const keyring = new Keyring({ type: "sr25519" });
  const account = keyring.addFromUri(NODE_ACCOUNT);
  const provider = new WsProvider(SUBSTRATE_WS_URL);

  const api = await ApiPromise.create({
    provider,
    rpc,
  });

  try {
    await initAbrTimestamp({ account, api });
    console.log("Successful init timestamp");

    await addLiquidatorSigner({
      account,
      api,
      signersPubKeys,
    });
    console.log("Successful add signers pub keys");
  } catch (error) {
    console.log(error);
    await api.disconnect();
    process.exit(1);
  }
  await api.disconnect();
})().catch(console.error);
