const { ApiPromise, WsProvider } = require( '@polkadot/api');
const { cryptoWaitReady } = require('@polkadot/util-crypto');
const { Keyring } = require('@polkadot/keyring');

const keyring = new Keyring({ type: 'sr25519' });


async function main(){

    await cryptoWaitReady();

    const wsProvider = new WsProvider('ws://localhost:9944');
    const api = await ApiPromise.create({ provider: wsProvider });

    console.log(`Genesis hash ${api.genesisHash.toHex()}`);

    const chain = await api.rpc.system.chain();
    const lastHeader = await api.rpc.chain.getHeader();
    console.log(`${chain}: last block #${lastHeader.number} has hash ${lastHeader.hash}`);

    const signedBlock = await api.rpc.chain.getBlock(lastHeader.hash);
    console.log(`Last block hash ${signedBlock.hash}`);

    let count = 0;
    const unsubHeads = await api.rpc.chain.subscribeNewHeads((lastHeader) => {
        console.log(`${chain}: last block #${lastHeader.number} has hash ${lastHeader.hash}, author ${lastHeader.author}`);

        if (++count === 10) {
            unsubHeads();
            api.disconnect();

        }
    });

}

main();
