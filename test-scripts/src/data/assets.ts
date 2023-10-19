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
  },
  {
    id: 'USDC',
    version: 2,
    shortName: 'USDC',
    isTradable: false,
    isCollateral: true,
    l2Address: '0x0123',
    decimals: 6,
  },
];
