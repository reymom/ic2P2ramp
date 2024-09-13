import { addresses, tokenCanisters, TokenMapping } from './addresses';

export const defaultCommitEvmGas = BigInt(80000);
export const defaultReleaseEvmGas = BigInt(90000);

export interface TokenOption {
  name: string;
  address: string;
  decimals: number;
  isNative: boolean;
  rateSymbol: string;
}

export const getEvmTokenOptions = (chainId: number): TokenOption[] => {
  const mapping = addresses[chainId];
  if (!mapping) {
    throw new Error(`No address mapping found for chainId ${chainId}`);
  }

  const tokens: TokenMapping[] = [mapping.native, mapping.usdt, mapping.usdc];
  const options = tokens.map((token) => ({
    name: token.name,
    address: token.address === 'native' ? '' : token.address,
    decimals: token.decimals,
    isNative: token.address === 'native',
    rateSymbol: token.name,
  }));

  return options.filter((token) => token.address !== '' || token.isNative);
};

export const getIcpTokenOptions = (): TokenOption[] => {
  return [
    {
      name: 'ICP',
      address: tokenCanisters.ICP,
      decimals: 8,
      isNative: true,
      rateSymbol: 'ICP',
    },
    {
      name: 'ckBTC',
      address: tokenCanisters.ckBTC,
      decimals: 8,
      isNative: false,
      rateSymbol: 'BTC',
    },
  ];
};
