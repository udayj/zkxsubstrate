import { AssetEntity } from '../entities';

export const data: AssetEntity[] = [
  {
    id: 'ETH',
    name: 'ETH',
    isTradable: true,
    isCollateral: false,
    tokenDecimal: 6,
  },
  {
    id: 'USDC',
    name: 'USDC',
    isTradable: false,
    isCollateral: true,
    tokenDecimal: 6,
  },
];
