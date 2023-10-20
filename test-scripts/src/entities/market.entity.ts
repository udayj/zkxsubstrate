export class MarketEntity {
  id: string;
  asset: string;
  assetCollateral: string;
  isTradable: boolean;
  isArchived: boolean;
  ttl: number;
  tickSize: number;
  tickPrecision: number;
  stepSize: number;
  stepPrecision: number;
  minimumOrderSize: number;
  minimumLeverage: number;
  maximumLeverage: number;
  currentlyAllowedLeverage: number;
  maintenanceMarginFraction: number;
  initialMarginFraction: number;
  incrementalInitialMarginFraction: number;
  incrementalPositionSize: number;
  baselinePositionSize: number;
  maximumPositionSize: number;
  metadataUrl: string;

  constructor(data: Partial<MarketEntity>) {
    Object.assign(this, data);
  }
}
