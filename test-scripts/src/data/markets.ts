import { MarketEntity } from '../entities';

export const data: MarketEntity[] = [
  {
    id: 'ETH-USDC',
    asset: 'ETH',
    assetCollateral: 'USDC',
    isTradable: true,
    isArchived: false,
    ttl: 3600,
    tickSize: 0.1,
    tickPrecision: 0,
    stepSize: 0.01,
    stepPrecision: 0,
    minimumOrderSize: 0.01,
    minimumLeverage: 1,
    maximumLeverage: 20,
    currentlyAllowedLeverage: 20,
    maintenanceMarginFraction: 0.03,
    initialMarginFraction: 0.05,
    incrementalInitialMarginFraction: 0.01,
    incrementalPositionSize: 100,
    baselinePositionSize: 500,
    maximumPositionSize: 10000,
    metadataUrl: 'https://zkxprotocol-deploy.s3.eu-central-1.amazonaws.com/eth-usdc.metadata.json',
  },
];
