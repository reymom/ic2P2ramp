import ethereumLogo from '../assets/blockchains/ethereum-logo.png';
import coinBaseLogo from '../assets/blockchains/coinbase-logo.svg';
import mantleLogo from '../assets/blockchains/mantle.png';
import optimismLogo from '../assets/blockchains/optimism-logo.svg';
import arbitrumLogo from '../assets/blockchains/arbitrum-logo.svg';

export interface NetworkProps {
  id: number;
  name: string;
  explorer: string;
  logo: string;
}

export const NetworkIds: { [chainId: string]: NetworkProps } =
  process.env.FRONTEND_EVM_ENV === 'production'
    ? {
        MAINNET: {
          id: 1,
          name: 'Mainnet',
          explorer: 'https://etherscan.io',
          logo: ethereumLogo,
        },
        BASE: {
          id: 8453,
          name: 'Base',
          explorer: 'https://basescan.org',
          logo: coinBaseLogo,
        },
        OPTIMISM: {
          id: 10,
          name: 'Optimism',
          explorer: 'https://optimistic.etherscan.io',
          logo: optimismLogo,
        },
        ARBITRUM: {
          id: 42161,
          name: 'Arbitrum',
          explorer: 'https://arbiscan.io',
          logo: arbitrumLogo,
        },
      }
    : {
        SEPOLIA: {
          id: 11155111,
          name: 'Sepolia',
          explorer: 'https://sepolia.etherscan.io',
          logo: ethereumLogo,
        },
        BASE_SEPOLIA: {
          id: 84532,
          name: 'Base Sepolia',
          explorer: 'https://sepolia.basescan.org',
          logo: coinBaseLogo,
        },
        OP_SEPOLIA: {
          id: 11155420,
          name: 'Optimism Sepolia',
          explorer: 'https://sepolia-optimism.etherscan.io',
          logo: optimismLogo,
        },
        MANTLE_SEPOLIA: {
          id: 5003,
          name: 'Mantle Sepolia',
          explorer: 'https://explorer.sepolia.mantle.xyz',
          logo: mantleLogo,
        },
        ARBITRUM_SEPOLIA: {
          id: 421614,
          name: 'Arbitrum',
          explorer: 'https://sepolia.arbiscan.io',
          logo: arbitrumLogo,
        },
      };
