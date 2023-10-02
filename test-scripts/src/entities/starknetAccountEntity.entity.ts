export class StarknetAccountEntity {
  privateKey: string;
  publicKey: string;
  starkKey: string;

  constructor(data: Partial<StarknetAccountEntity>) {
    Object.assign(this, data);
  }
}
