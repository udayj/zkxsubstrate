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

console.log('Generated Account:');
console.log('Address:', address);
console.log('Seed Phrase:', seedPhrase);