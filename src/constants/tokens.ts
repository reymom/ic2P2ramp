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

export const NetworkIds =
  process.env.FRONTEND_ENV === 'production'
    ? {
        MAINNET: {
          id: 1,
          name: 'Mainnet',
          explorer: 'https://etherscan.io/tx/',
        },
        POLYGON: {
          id: 137,
          name: 'Polygon',
          explorer: 'https://polygonscan.com/tx/',
        },
        OPTIMISM: {
          id: 10,
          name: 'Optimism',
          explorer: 'https://optimistic.etherscan.io/tx/',
        },
        ARBITRUM: {
          id: 42161,
          name: 'Arbitrum',
          explorer: 'https://arbiscan.io/tx/',
        },
      }
    : {
        SEPOLIA: {
          id: 11155111,
          name: 'Sepolia',
          explorer: 'https://sepolia.etherscan.io/tx/',
        },
        BASE_SEPOLIA: {
          id: 84532,
          name: 'Base Sepolia',
          explorer: 'https://sepolia.basescan.org/tx/',
        },
        OP_SEPOLIA: {
          id: 11155420,
          name: 'Optimism Sepolia',
          explorer: 'https://optimism.etherscan.io/tx/',
        },
        POLYGON_ZKEVM_TESTNET: {
          id: 2442,
          name: 'Polygon zkEVM Testnet',
          explorer: 'https://explorer.public.zkevm-test.net/tx/',
        },
      };

const getNativeTokenForChainId = (chainId: number): string => {
  switch (chainId) {
    case NetworkIds.SEPOLIA?.id:
      return 'ETH';
    case NetworkIds.BASE_SEPOLIA?.id:
      return 'ETH';
    case NetworkIds.POLYGON_ZKEVM_TESTNET?.id:
      return 'MATIC';
    case NetworkIds.OP_SEPOLIA?.id:
      return 'OP';
    case NetworkIds.MAINNET?.id:
      return 'ETH';
    case NetworkIds.POLYGON?.id:
      return 'MATIC';
    case NetworkIds.OPTIMISM?.id:
      return 'OP';
    case NetworkIds.ARBITRUM?.id:
      return 'ETH';
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
