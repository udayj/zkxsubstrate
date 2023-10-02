export class AssetEntity {
  id: string;
  name: string;
  isTradable: boolean;
  isCollateral: boolean;
  tokenDecimal: number;

  constructor(data: Partial<AssetEntity>) {
    Object.assign(this, data);
  }
}
