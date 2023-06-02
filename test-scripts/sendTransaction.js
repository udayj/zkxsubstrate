const { ApiPromise, WsProvider } = require( '@polkadot/api');
const { cryptoWaitReady } = require('@polkadot/util-crypto');

const { Keyring } = require('@polkadot/keyring');

const keyring = new Keyring({ type: 'sr25519' });

async function main(){

    await cryptoWaitReady();

    const wsProvider = new WsProvider('ws://localhost:9944');
    const api = await ApiPromise.create({ provider: wsProvider });

    //default developer account
    const alice = keyring.addFromUri('//Alice', { name: 'Alice default' });
    console.log(`alice address #${alice.address}`);

    const { nonce, data: balance } = await api.query.system.account(alice.address);
    console.log(` balance of ${balance.free} and a nonce of ${nonce}`);

    const result = await api.tx.zkxTradingAccount.recordAccount({ account_id: [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32]})
        .signAndSend(alice);


    let count = 0;
    const unsubHeads = await api.rpc.chain.subscribeNewHeads(async (lastHeader) => {

        const signedBlock = await api.rpc.chain.getBlock(lastHeader.parentHash);

        const apiAt = await api.at(signedBlock.block.header.hash);
        const allRecords = await apiAt.query.system.events();


        signedBlock.block.extrinsics.forEach((ex, index) => {
            // the extrinsics are decoded by the API, human-like view
            console.log(index, ex.toHuman());

            const { isSigned, meta, method: { args, method, section } } = ex;

            // explicit display of name, args & documentation
            console.log(`${section}.${method}(${args.map((a) => a.toString()).join(', ')})`);
//            console.log(meta.documentation.map((d) => d.toString()).join('\n'));

            if(method === "recordAccount"){

                const events = allRecords
                    .filter(({ phase }) =>
                        phase.isApplyExtrinsic &&
                        phase.asApplyExtrinsic.eq(index)
                    )
                    .map(({ event }) => `${event.section}.${event.method}`);

                console.log(`${section}.${method}:: ${events.join(', ') || 'no events'}`);
                return;
            }

            // signer/nonce info
            if (isSigned) {
                console.log(`signer=${ex.signer.toString()}, nonce=${ex.nonce.toString()}`);
            }
        });
    });
    /*, async ({ status, events }) => {

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
*/
    console.log(`Result ${result.toHuman()}`);

}

main();

/*
console.log('Generated Account:');
console.log('Address:', address);
console.log('Seed Phrase:', seedPhrase);*/
