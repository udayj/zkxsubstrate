export class AssetEntity {
  id: string;
  version: number;
  shortName: string;
  isTradable: boolean;
  isCollateral: boolean;
  l2Address: string;
  decimals: number;

  constructor(data: Partial<AssetEntity>) {
    Object.assign(this, data);
  }
}
