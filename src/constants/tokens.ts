import { addresses, tokenCanisters } from './addresses';

export interface TokenOption {
  name: string;
  address: string;
  isNative: boolean;
  rateSymbol: string;
}

export const getEvmTokenOptions = (chainId: number): TokenOption[] => {
  const mapping = addresses[chainId];
  if (!mapping) {
    throw new Error(`No address mapping found for chainId ${chainId}`);
  }

  return [
    {
      name: mapping.native,
      address: '',
      isNative: true,
      rateSymbol: mapping.native,
    },
    {
      name: mapping.usdt[0],
      address: mapping.usdt[1],
      isNative: false,
      rateSymbol: mapping.usdt[0],
    },
  ];
};

export const getIcpTokenOptions = (): TokenOption[] => {
  return [
    {
      name: 'ICP',
      address: tokenCanisters.ICP,
      isNative: true,
      rateSymbol: 'ICP',
    },
    {
      name: 'ckBTC',
      address: tokenCanisters.ckBTC,
      isNative: false,
      rateSymbol: 'BTC',
    },
  ];
};
