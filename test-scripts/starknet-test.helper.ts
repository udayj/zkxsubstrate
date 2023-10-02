import * as baseStarknet from 'starknet';
import { StarknetHelper } from './starknet.helper';

const { getPublicKey, getStarkKey, sign } = baseStarknet.ec.starkCurve;
const { randomPrivateKey } = baseStarknet.ec.starkCurve.utils;
const { addHexPrefix, buf2hex, removeHexPrefix, padLeft } = baseStarknet.encode;
const { toHex } = baseStarknet.num;

export class StarknetTestHelper {
  static generateKeys(): {
    privateKey: string;
    publicKey: string;
    starkKey: string; // (on Substrate it is TradingAccount.pub_key)
  } {
    const privateKeyAsBuffer = randomPrivateKey();
    const privateKey = addHexPrefix(buf2hex(privateKeyAsBuffer));

    const publicKeyAsBuffer = getPublicKey(privateKeyAsBuffer, true);
    const publicKey = addHexPrefix(buf2hex(publicKeyAsBuffer));

    const starkKeyNotPadded = getStarkKey(privateKey);
    const starkKeyWithoutPrefix = removeHexPrefix(starkKeyNotPadded);
    const starkKeyPadded = padLeft(starkKeyWithoutPrefix, 64);
    const starkKey = addHexPrefix(starkKeyPadded);

    return {
      privateKey,
      publicKey,
      starkKey,
    };
  }

  static sign(params: { privateKey: string; data: any }): [string, string] {
    const { privateKey, data } = params;
    const dataHash = StarknetHelper.getHash(data);
    const signature = sign(dataHash, privateKey);
    const signatureR = toHex(signature.r);
    const signatureS = toHex(signature.s);

    return [signatureR, signatureS];
  }
}