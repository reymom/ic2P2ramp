import { getTokenOptions } from './addresses';

export enum SepoliaTokens {
  USDT = 'USDT',
  ETH = 'ETH',
}

export enum BaseSepoliaTokens {
  USDT = 'USDT',
  ETH = 'ETH',
}

export enum PolygonZkEvmTokens {
  USDT = 'USDT',
  MATIC = 'MATIC',
}

export enum OptimismSepoliaTokens {
  USDT = 'USDT',
  OP = 'OP',
}

export const NetworkIds = {
  SEPOLIA: 11155111,
  BASE_SEPOLIA: 84532,
  OP_SEPOLIA: 11155420,
  POLYGON_ZKEVM_TESTNET: 2442,
};

const getNativeTokenForChainId = (chainId: number): string => {
  switch (chainId) {
    case NetworkIds.SEPOLIA:
      return 'ETH';
    case NetworkIds.BASE_SEPOLIA:
      return 'ETH';
    case NetworkIds.POLYGON_ZKEVM_TESTNET:
      return 'MATIC';
    case NetworkIds.OP_SEPOLIA:
      return 'OP';
    default:
      return 'Unknown Token';
  }
};

export const getTokenMapping = (
  chainId: number,
): { [address: string]: string } => {
  const tokenOptions = getTokenOptions(chainId);
  const nativeToken = getNativeTokenForChainId(chainId);
  const mapping = tokenOptions.reduce((map, token) => {
    map[token.address] = token.name;
    return map;
  }, {} as { [address: string]: string });
  mapping[''] = nativeToken; // Handle native token case
  return mapping;
};
