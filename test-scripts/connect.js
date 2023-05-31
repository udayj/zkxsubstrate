const { ApiPromise, WsProvider } = require( '@polkadot/api');

const { Keyring } = require('@polkadot/keyring');
const { mnemonicGenerate } = require('@polkadot/util-crypto');

// Generate a new seed phrase
const seedPhrase = mnemonicGenerate();

// Create a new keyring instance
const keyring = new Keyring();

// Generate a new account
const account = keyring.createFromUri(`//${seedPhrase}`);

// Get the account address and seed phrase
const address = account.address;

async function main(){
    // Construct
    const wsProvider = new WsProvider('ws://localhost:9944');
    const api = await ApiPromise.create({ provider: wsProvider });

// Do something
    console.log(api.genesisHash.toHex());

   // const ADDR = '5DTestUPts3kjeXSTMyerHihn1uwMfLj8vU8sqF7qYrFabHE';
    const { nonce, data: balance } = await api.query.system.account(address);
    console.log(` balance of ${balance.free} and a nonce of ${nonce}`);

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
   /* const txHash = await api.tx.balances
        .transfer(address, 12345)
        .signAndSend(alice);
*/
// Show the hash
   // console.log(`Submitted with hash ${txHash}`);

    const nonce1 = await api.rpc.system.accountNextIndex(alice.address);

    console.log(`nonce ${nonce1}`);

}

main();

/*
console.log('Generated Account:');
console.log('Address:', address);
console.log('Seed Phrase:', seedPhrase);*/
