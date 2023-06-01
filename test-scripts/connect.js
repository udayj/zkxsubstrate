const { ApiPromise, WsProvider } = require( '@polkadot/api');
const { cryptoWaitReady } = require('@polkadot/util-crypto');

const { Keyring } = require('@polkadot/keyring');
const { mnemonicGenerate } = require('@polkadot/util-crypto');

// Generate a new seed phrase
const seedPhrase = mnemonicGenerate();

// Create a new keyring instance
const keyring = new Keyring({ type: 'sr25519' });

// Generate a new account
//const account = keyring.createFromUri(`//${seedPhrase}`);

// Get the account address and seed phrase
//const address = account.address;

async function main(){

    // Wait for the crypto library to be ready
    await cryptoWaitReady();

    // Construct
    const wsProvider = new WsProvider('ws://localhost:9944');
    const api = await ApiPromise.create({ provider: wsProvider });

// Do something
    console.log(api.genesisHash.toHex());


// Retrieve the chain name
    const chain = await api.rpc.system.chain();

// Retrieve the latest header
    const lastHeader = await api.rpc.chain.getHeader();

// Log the information
    console.log(`${chain}: last block #${lastHeader.number} has hash ${lastHeader.hash}`);

    let count = 0;

    const signedBlock = await api.rpc.chain.getBlock(lastHeader.hash);



// Subscribe to the new headers
    const unsubHeads = await api.rpc.chain.subscribeNewHeads((lastHeader) => {
        console.log(`${chain}: last block #${lastHeader.number} has hash ${lastHeader.hash}`);



        if (++count === 10) {
            unsubHeads();
            api.disconnect();

        }
    });

    const alice = keyring.addFromUri('//Alice', { name: 'Alice default' });

    console.log(`${chain}: address #${alice.address}`);

    // const ADDR = '5DTestUPts3kjeXSTMyerHihn1uwMfLj8vU8sqF7qYrFabHE';
    const { nonce, data: balance } = await api.query.system.account("");
    console.log(` balance of ${balance.free} and a nonce of ${nonce}`);

    /* const txHash = await api.tx.balances
         .transfer(address, 12345)
         .signAndSend(alice);
 */
// Show the hash
   // console.log(`Submitted with hash ${txHash}`);

    const nonce1 = await api.rpc.system.accountNextIndex(alice.address);

    console.log(`nonce ${nonce1}`);

    api.tx.zkxTradingAccount.recordAccount({ account_id: [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32]}) .signAndSend(alice, async ({ status, events }) => {

        const account = await api.query.zkxTradingAccount.accounts([1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32]);

        const humanAccount = account.toHuman();

        console.log(`account json ${JSON.stringify(humanAccount)}`);


        if (status.isInBlock || status.isFinalized) {
            events
                // find/filter for failed events
                .filter(({ event }) =>
                    api.events.system.ExtrinsicFailed.is(event)
                )
                // we know that data for system.ExtrinsicFailed is
                // (DispatchError, DispatchInfo)
                .forEach(({ event: { data: [error, info] } }) => {
                    if (error.isModule) {
                        // for module errors, we have the section indexed, lookup
                        const decoded = api.registry.findMetaError(error.asModule);
                        const { docs, method, section } = decoded;

                        console.log(`${section}.${method}: ${docs.join(' ')}`);
                    } else {
                        // Other, CannotLookup, BadOrigin, no extra info
                        console.log(error.toString());
                    }
                });
        }
    });



}

main();

/*
console.log('Generated Account:');
console.log('Address:', address);
console.log('Seed Phrase:', seedPhrase);*/
