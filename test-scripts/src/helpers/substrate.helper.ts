import { Decimal } from 'decimal.js';
import { blake2AsU8a } from '@polkadot/util-crypto';
import { TypeRegistry } from '@polkadot/types';
import { I128, U128, U256 } from '@polkadot/types-codec';
import { randomUUID } from 'crypto';

import {
  bnToHex,
  hexToBn,
  hexToU8a,
  hexToString,
  numberToU8a,
  stringToHex,
  u8aToHex,
} from '@polkadot/util';
import { TradingAccountEntity } from '../entities';

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
    accountIndex: number; // must be between [0, 255]
    accountAddress: string; // must be in hex
  }): string {
    let accountAddress = params.accountAddress;

    if (!accountAddress.includes('0x')) {
      throw new Error('Invalid account address');
    } else {
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
      throw new Error('account address should be equal 32 bytes');
    }

    const concatenatedBytes = new Uint8Array(33);

    concatenatedBytes.set(accountAddressLeBytes, 0);
    concatenatedBytes.set(accountIndexLeBytes, 32);

    const hashAsBytes = blake2AsU8a(concatenatedBytes, 256);
    const hashAsString = u8aToHex(hashAsBytes);

    return hashAsString;
  }

  static generateHexId(): string {
    const uuid = randomUUID().split('-').join('');
    return `0x${uuid}`;
  }

  static generateTradingAccount(params: {
    accountIndex?: number;
    accountAddress?: string;
    starkKey: string, // it is pub_key in Substrate
  }): TradingAccountEntity {
    const accountIndex = params.accountIndex ?? 0;
    const accountAddress = params.accountAddress ?? SubstrateHelper.generateHexId();

    const accountId = SubstrateHelper.calculateAccountId({
      accountIndex,
      accountAddress,
    });

    const tradingAccount = new TradingAccountEntity({
      id: accountId,
      index: accountIndex,
      address: accountAddress,
      publicKey: params.starkKey,
    });

    return tradingAccount;
  }
}
