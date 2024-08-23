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
