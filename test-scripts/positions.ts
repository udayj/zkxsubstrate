import { nanoid } from 'nanoid';
import { randomUUID } from 'crypto';
import { BigNumber } from 'bignumber.js';

BigNumber.set({
  EXPONENTIAL_AT: 25,
});

function generateHexId (): string {
  const uuid = randomUUID().split('-').join('');
  return `0x${uuid}`;
}

import { TypeRegistry } from '@polkadot/types';
import { ApiPromise, WsProvider } from '@polkadot/api';
import { blake2AsU8a, cryptoWaitReady } from '@polkadot/util-crypto';
import { Keyring } from '@polkadot/keyring';

import {
  I128,
  U128,
  U256,
} from '@polkadot/types-codec';

import {
  BN,
  hexToBn,
  hexToU8a,
  hexToString,
  stringToU8a,
  stringToHex,
  u8aToHex,
  u8aToString,
  u8aToNumber,
  u8aToFloat,
  numberToHex,
  numberToU8a,
  compactStripLength,
  bnToHex,
} from '@polkadot/util';

import * as baseStarknet from 'starknet';
const { encodeShortString, decodeShortString } = baseStarknet.shortString;

const NODE_ACCOUNT = '//Alice';

function convertHexToU256 (value: string): U256 {
  BigNumber.set({
    EXPONENTIAL_AT: 100,
    DECIMAL_PLACES: 100,
  });

  const registry = new TypeRegistry();
  const valueAsBigNumber = hexToBn(value);
  const valueAsBigNumberString = valueAsBigNumber.toString();
  return registry.createType('u256', valueAsBigNumberString);
}

function convertStringToU256 (value: string): U256 {
  BigNumber.set({
    EXPONENTIAL_AT: 100,
    DECIMAL_PLACES: 100,
  });

  const registry = new TypeRegistry();
  const valueAsHex = stringToHex(value);
  const valueAsBigNumber = hexToBn(valueAsHex);
  const valueAsBigNumberString = valueAsBigNumber.toString();
  return registry.createType('u256', valueAsBigNumberString);
}

function convertU256ToString (value: any): string {
  const valueAsHex = bnToHex(value);
  const valueAsString = hexToString(valueAsHex);
  return valueAsString;
}

function convertToFixedI128 (value: number): I128 {
  const registry = new TypeRegistry();
  const multiplyBy = new BigNumber(10).pow(18);
  const result = new BigNumber(value).multipliedBy(multiplyBy);
  return registry.createType('i128', result.toString());
}

function convertFromFixedI128 (value: any): number {
  const multiplyBy = new BigNumber(10).pow(18);
  return new BigNumber(value).dividedBy(multiplyBy).toNumber();
}

function calculateAccountId(params: {
  accountIndex: number; // must be between [0, 255]
  accountAddress: string; // must be in hex
}): string {
  let accountAddress = params.accountAddress;

  if (accountAddress.includes('0x')) {
    accountAddress = accountAddress.substring(2);
  }

  // ex: "0x1ab" should be "0x01ab"
  if (accountAddress.length % 2 !== 0) {
    accountAddress = `0${accountAddress}`;
  }

  const accountIndexLeBytes = numberToU8a(params.accountIndex);
  const accountAddressLeBytes = hexToU8a(accountAddress).reverse();

  // just to be sure
  if (accountAddressLeBytes.length > 32) {
    throw new Error('accountAddress should be <= 32 bytes');
  }

  const concatenatedBytes = new Uint8Array(33);

  Buffer.from(accountAddressLeBytes).copy(concatenatedBytes, 0);
  Buffer.from(accountIndexLeBytes).copy(concatenatedBytes, 32);

  const hashAsBytes = blake2AsU8a(concatenatedBytes, 256);
  const hashAsString = u8aToHex(hashAsBytes);

  return hashAsString;
}

process.nextTick(async () => {
  await cryptoWaitReady();

  // Construct the keyring after the API (crypto has an async init)
  const keyring = new Keyring({ type: 'sr25519' });

  // Add Alice to our keyring with a hard-derivation path (empty phrase, so uses dev)
  const nodeAccountKeyring = keyring.addFromUri(NODE_ACCOUNT);

  const wsProvider = new WsProvider('ws://127.0.0.1:9944');
  const api = await ApiPromise.create({
    provider: wsProvider,
  });

  const positionsMap = await api.query.trading.positionsMap;
  const positionsMapEntries = (await positionsMap.entries()).map((kek) => {
    const obj0 = kek[0].toJSON() as any;
    const obj1 = kek[1].toPrimitive() as any;
    obj1.avgExecutionPrice = convertFromFixedI128(obj1.avgExecutionPrice);
    obj1.size = convertFromFixedI128(obj1.size_);
    return obj1;
  });

  console.log(positionsMapEntries);
});