export class TradingAccountEntity {
  id: string;
  index: number;
  address: string;
  publicKey: string;

  constructor(data: Partial<TradingAccountEntity>) {
    Object.assign(this, data);
  }
}
