import { AssetEntity } from '../entities';

export const data: AssetEntity[] = [
  {
    id: 'ETH',
    version: 1,
    shortName: 'ETH',
    isTradable: true,
    isCollateral: false,
    l2Address: '0x0123',
    decimals: 6,
    metadataUrl: 'https://zkxprotocol-deploy.s3.eu-central-1.amazonaws.com/eth.metadata.json',
  },
  {
    id: 'USDC',
    version: 2,
    shortName: 'USDC',
    isTradable: false,
    isCollateral: true,
    l2Address: '0x0123',
    decimals: 6,
    metadataUrl: 'https://zkxprotocol-deploy.s3.eu-central-1.amazonaws.com/usdc.metadata.json',
  },
];
