import ethereumLogo from '../assets/blockchains/ethereum-logo.png';
import coinBaseLogo from '../assets/blockchains/coinbase-logo.svg';
import polygonMaticLogo from '../assets/blockchains/polygon-matic.svg';
import mantleLogo from '../assets/blockchains/mantle.png';
import optimismLogo from '../assets/blockchains/optimism-logo.svg';

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
        POLYGON: {
          id: 137,
          name: 'Polygon',
          explorer: 'https://polygonscan.com',
          logo: polygonMaticLogo,
        },
        OPTIMISM: {
          id: 10,
          name: 'Optimism',
          explorer: 'https://optimistic.etherscan.io',
          logo: optimismLogo,
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
      };
