// Import the API & Provider and some utility functions
const { ApiPromise, WsProvider } = require("@polkadot/api");
const { Keyring } = require("@polkadot/keyring");

const fs = require("fs");

const seedphrase_hex = "";

async function main() {
  // Initialise the provider to connect to the local node
  const provider = new WsProvider("wss://127.0.0.1:9944");

  const keyring = new Keyring({ type: 'sr25519' });

  // Create the API and wait until ready (optional provider passed through)
  const api = await ApiPromise.create({ provider });
  
  // Create a key pair from the given public and private keys
  const adminPair = keyring.addFromUri(seedphrase_hex);

  // Retrieve the runtime to upgrade
  const code = fs
    .readFileSync("./node_template_runtime.compact.compressed.wasm")
    .toString("hex");
  const proposal =
    api.tx.system && api.tx.system.setCode
      ? api.tx.system.setCode(`0x${code}`) // For newer versions of Substrate
      : api.tx.consensus.setCode(`0x${code}`); // For previous versions

  console.log(`Upgrading from ${adminPair}, ${code.length / 2} bytes`);

  // Perform the actual chain upgrade via the sudo module
  const sudo = api.tx.sudo.sudoUncheckedWeight(proposal, { weight: 0 });
  sudo.signAndSend(adminPair, ({ events = [], status }) => {
    console.log("Proposal status:", status.type);

    if (status.isInBlock) {
      console.error("You have just upgraded your chain");

      console.log("Included at block hash", status.asInBlock.toHex());
      console.log("Events:");

      // console.log(JSON.stringify(events.toHuman(), null, 2));
      events.forEach(({ event: { data, method, section }, phase }) => {
        console.log(
          "\t",
          phase.toString(),
          `: ${section}.${method}`,
          data.toString()
        );
      });
    } else if (status.isFinalized) {
      console.log("Finalized block hash", status.asFinalized.toHex());

      process.exit(0);
    }
  });
}

main().catch((error) => {
  console.error(error);
  process.exit(-1);
});