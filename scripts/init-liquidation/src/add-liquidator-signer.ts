import { ApiPromise } from "@polkadot/api";
import { KeyringPair } from "@polkadot/keyring/types";
import { SubstrateHelper } from "./substrate.helper";

export async function addLiquidatorSigner(params: {
  api: ApiPromise;
  account: KeyringPair;
  signersPubKeys: string[];
}) {
  const { api, account, signersPubKeys } = params;

  if (!signersPubKeys.length) {
    return;
  }

  try {
    for (const pubKey of signersPubKeys) {
      try {
        const nonce = await api.rpc.system.accountNextIndex(account.address);

        await new Promise<void>((resolve, reject) => {
          api.tx.sudo
            .sudo(
              api.tx.trading.addLiquidatorSigner(
                SubstrateHelper.convertStringToU256(pubKey)
              )
            )
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
      } catch (error: any) {
        if (error.name === "DuplicateSigner") {
          console.warn(`Signer pub key already added: ${pubKey}`);
        } else {
          throw error;
        }
      }
    }
  } catch (error) {
    console.error("Error: %s", error);
    process.exit(1);
  }

  function getError(dispatchError: any) {
    if (dispatchError.isModule) {
      const decoded = api.registry.findMetaError(dispatchError.asModule);

      return JSON.parse(JSON.stringify(decoded));
    } else {
      return dispatchError.toPrimitive();
    }
  }
}
