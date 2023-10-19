import * as baseStarknet from 'starknet';

const { Signature, keccak, verify } = baseStarknet.ec.starkCurve;
const { addHexPrefix, utf8ToArray } = baseStarknet.encode;
const { toHex, toBigInt } = baseStarknet.num;

export class StarknetHelper {
  static getHash(data: any): string {
    const ignoredValues = ['', undefined];
    const dataAsString = Object.keys(data)
      .map((key) => data[key])
      .filter((value) => !ignoredValues.includes(value))
      .map((value) => String(value))
      .sort()
      .join('|');

    const dataAsBuffer = utf8ToArray(dataAsString);
    const dataAsBn = keccak(dataAsBuffer);

    return toHex(dataAsBn);
  }

  static getStarkKeyFromPublicKey(publicKey: string): string {
    return addHexPrefix(publicKey.slice(-64));
  }

  static verify(params: {
    publicKey: string;
    signature: [string, string];
    data: any;
  }): boolean {
    const { publicKey, signature, data } = params;
    const signatureR = toBigInt(signature[0]);
    const signatureS = toBigInt(signature[1]);
    const signatureInstance = new Signature(signatureR, signatureS);
    const dataHash = StarknetHelper.getHash(data);

    return verify(signatureInstance, dataHash, publicKey);
  }
}
