import { createHash } from 'crypto';
import { setTimeout } from 'timers/promises'
import { ApiPromise, WsProvider } from "@polkadot/api";
import { cryptoWaitReady, randomAsHex } from "@polkadot/util-crypto";
import { Keyring } from "@polkadot/keyring";

(async () => {
  await cryptoWaitReady();

  const {
    NODE_ACCOUNT = "upon spice cloth armed bitter fiction despair tide rate spice ten spend",
    ZKX_NODE_ACCOUNT = 'upon spice cloth armed bitter fiction despair tide rate spice ten spend',
    // SUBSTRATE_URL = "ws://127.0.0.1:9944",
    // SUBSTRATE_URL = "wss://l3.stand-1.k8s.ntwrkx.com:443",
    // SUBSTRATE_URL = "wss://l3.stand-2.k8s.ntwrkx.com:443",
    // SUBSTRATE_URL = "wss://l3.stand-4.k8s.ntwrkx.com:443",
    SUBSTRATE_URL = "wss://l3.stage.zkx.fi",
    // SUBSTRATE_URL = "wss://l3.sandbox-2.zkx.fi",
    NUMBER_OF_ACCOUNTS = "100",
    // should not be less than 500000000
    AMOUNT_OF_MONEY = "500000000000000",
  } = process.env;

  const numberOfAccounts = parseInt(NUMBER_OF_ACCOUNTS, 10);
  const amountOfMoney = parseInt(AMOUNT_OF_MONEY, 10);

  const keyring = new Keyring({ type: "sr25519" });
  const nodeAccountKeyring = keyring.addFromUri(NODE_ACCOUNT);

  const wsProvider = new WsProvider(SUBSTRATE_URL);
  const api = await ApiPromise.create({
    provider: wsProvider,
    noInitWarn: true,
  });

  const results = [];

  for (let i = 0; i < numberOfAccounts; i++) {
    try {
      const seedPhraseHash = createHash('sha256').update(`${ZKX_NODE_ACCOUNT}${i}`).digest('hex');
      const seedPhrase = `0x${seedPhraseHash}`;
      await createAccount({ seedPhrase });
      await setTimeout(2500);
      results.push(seedPhrase);
      console.log(seedPhrase);
    } catch (error) {
      console.error(error);
    }
  }

  console.log(results.join(","));

  await api.disconnect();

  function getError(dispatchError: any) {
    if (dispatchError.isModule) {
      const decoded = api.registry.findMetaError(dispatchError.asModule);

      return JSON.parse(JSON.stringify(decoded));
    } else {
      return dispatchError.toPrimitive();
    }
  }

  async function createAccount(params: { seedPhrase: string }) {
    const { seedPhrase } = params;
    const keyring = new Keyring({ type: "sr25519" });
    const accountKeyring = keyring.addFromUri(seedPhrase);

    return new Promise<void>(async (resolve, reject) => {
      try {
        await api.tx.balances
          .transfer(accountKeyring.address, amountOfMoney)
          .signAndSend(
            nodeAccountKeyring,
            {
              nonce: -1,
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
