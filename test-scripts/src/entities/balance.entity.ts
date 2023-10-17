export class BalanceEntity {
  assetId: string;
  value: number;

  constructor(data: Partial<BalanceEntity>) {
    Object.assign(this, data);
  }
}
  