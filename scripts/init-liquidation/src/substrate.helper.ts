import { Decimal } from 'decimal.js';
import { blake2AsU8a } from '@polkadot/util-crypto';
import { TypeRegistry } from '@polkadot/types';
import { I128, U128, U256 } from '@polkadot/types-codec';

import {
  bnToHex,
  hexToBn,
  hexToU8a,
  hexToString,
  numberToU8a,
  stringToHex,
  stringToU8a,
  u8aToHex,
} from '@polkadot/util';

// This temporary registry doesn't require
// a connection to a Substrate node and can be used
const typeRegistry = new TypeRegistry();
const BN_10_POW_18 = Decimal.pow(10, 18);

export class SubstrateHelper {
  // ================
  // i128 converters
  // ================

  static convertHexToI128(value: string): I128 {
    const valueAsBn = hexToBn(value);
    const valueAsBnString = valueAsBn.toString();

    return typeRegistry.createType('i128', valueAsBnString);
  }

  static convertStringToI128(value: string): I128 {
    const valueAsHex = stringToHex(value);

    return typeRegistry.createType('i128', valueAsHex);
  }

  static convertNumberToI128(value: number | string): I128 {
    const valueAsBn = new Decimal(value).mul(BN_10_POW_18);
    const valueAsBnString = valueAsBn.toFixed();

    return typeRegistry.createType('i128', valueAsBnString);
  }

  static convertI128ToNumber(value: any): number {
    const valueAsBn = new Decimal(value).dividedBy(BN_10_POW_18);
    const valueAsNumber = valueAsBn.toNumber();

    return valueAsNumber;
  }

  // ================
  // u128 converters
  // ================

  static convertHexToU128(value: string): U128 {
    const valueAsBn = hexToBn(value);
    const valueAsBnString = valueAsBn.toString();

    return typeRegistry.createType('u128', valueAsBnString);
  }

  static convertNumberToU128(value: number): U128 {
    return typeRegistry.createType('u128', value);
  }

  static convertStringToU128(value: string): U128 {
    const valueAsHex = stringToHex(value);
    const valueAsBn = hexToBn(valueAsHex);
    const valueAsBnString = valueAsBn.toString();

    return typeRegistry.createType('u128', valueAsBnString);
  }

  static convertU128ToHex(value: any): string {
    const valueAsBn = value.toString();
    const valueAsHex = bnToHex(valueAsBn);

    return valueAsHex;
  }

  static convertU128ToString(value: any): string {
    const valueAsBn = value.toString();
    const valueAsHex = bnToHex(valueAsBn);
    const valueAsString = hexToString(valueAsHex);

    return valueAsString;
  }

  // ================
  // u256 converters
  // ================

  static convertHexToU256(value: string): U256 {
    const valueAsBn = hexToBn(value);
    const valueAsBnString = valueAsBn.toString();

    return typeRegistry.createType('u256', valueAsBnString);
  }

  static convertStringToU256(value: string): U256 {
    const valueAsHex = stringToHex(value);
    const valueAsBn = hexToBn(valueAsHex);
    const valueAsBnString = valueAsBn.toString();
    return typeRegistry.createType('u256', valueAsBnString);
  }

  static convertU256ToHex(value: any): string {
    const valueAsBn = value.toString();
    const valueAsHex = bnToHex(valueAsBn);

    return valueAsHex;
  }

  static convertU256ToString(value: any): string {
    const valueAsBn = value.toString();
    const valueAsHex = bnToHex(valueAsBn);
    const valueAsString = hexToString(valueAsHex);

    return valueAsString;
  }

  static calculateAccountId(params: {
    index: number; // must be between [0, 255]
    address: string; // must be in hex
  }): string {
    let address = params.address;

    if (!address.includes('0x')) {
      throw new Error('Invalid account address');
    } else {
      address = address.substring(2);
    }

    // ex: "0x1ab" should be "0x01ab"
    if (address.length % 2 !== 0) {
      address = `0${address}`;
    }

    const indexLeBytes = numberToU8a(params.index);
    const addressLeBytes = hexToU8a(address).reverse();

    // just to be sure
    if (addressLeBytes.length > 32) {
      throw new Error('account address should be <= 32 bytes');
    }

    const concatenatedBytes = new Uint8Array(33);

    concatenatedBytes.set(addressLeBytes, 0);
    concatenatedBytes.set(indexLeBytes, 32);

    const hashAsBytes = blake2AsU8a(concatenatedBytes, 256);
    const hashAsString = u8aToHex(hashAsBytes);

    return hashAsString;
  }
}
