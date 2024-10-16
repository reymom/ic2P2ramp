import { TokenOption } from '../model/types';

import ethereumLogo from '../assets/blockchains/ethereum-logo.png';
import mantleLogo from '../assets/blockchains/mantle.png';
import usdcLogo from '../assets/blockchains/usdc-logo.png';
import usdtLogo from '../assets/blockchains/tether-usdt-seeklogo.svg';
import eurcLogo from '../assets/blockchains/eurc-logo.png';
import shibaLogo from '../assets/blockchains/shiba-token.png';
import opLogo from '../assets/blockchains/optimism-logo.svg';
import daiLogo from '../assets/blockchains/dai-logo.png';

export const defaultCommitEvmGas = BigInt(80000);
export const defaultReleaseEvmGas = BigInt(100000);

// ----------------------
// EVM Tokens Definitions
// ----------------------

interface AddressMapping {
  vault: string;
  tokens: TokenOption[];
}

console.log('FRONTEND_EVM_ENV = ', process.env.FRONTEND_EVM_ENV);

if (process.env.FRONTEND_EVM_ENV === 'production') {
  if (
    !process.env.CONTRACT_MAINNET ||
    !process.env.CONTRACT_BASE ||
    !process.env.CONTRACT_OP
  ) {
    console.error('Contract addresses not defined');
  }
} else {
  if (
    !process.env.CONTRACT_SEPOLIA ||
    !process.env.CONTRACT_BASE_SEPOLIA ||
    !process.env.CONTRACT_OP_SEPOLIA ||
    !process.env.CONTRACT_MANTLE_SEPOLIA
  ) {
    console.error('Contract addresses not defined');
  }
}

const ethereumToken: TokenOption = {
  name: 'ETH',
  address: '',
  decimals: 18,
  isNative: true,
  rateSymbol: 'ETH',
  logo: ethereumLogo,
};

const newUsdtToken = (address: string, decimals: number): TokenOption => {
  return {
    name: 'USDT',
    address: address,
    decimals: decimals,
    isNative: false,
    rateSymbol: 'USD',
    logo: usdtLogo,
  };
};

const newUsdcToken = (address: string, decimals: number): TokenOption => {
  return {
    name: 'USDC',
    address: address,
    decimals: decimals,
    isNative: false,
    rateSymbol: 'USD',
    logo: usdcLogo,
  };
};

const newEurcToken = (address: string): TokenOption => {
  return {
    name: 'EURC',
    address: address,
    decimals: 6,
    isNative: false,
    rateSymbol: 'EUR',
    logo: eurcLogo,
  };
};

const newDaiToken = (address: string): TokenOption => {
  return {
    name: 'DAI',
    address: address,
    decimals: 18,
    isNative: false,
    rateSymbol: 'USD',
    logo: daiLogo,
  };
};

const newShibaToken = (address: string): TokenOption => {
  return {
    name: 'SHIB',
    address: address,
    decimals: 18,
    isNative: false,
    rateSymbol: 'SHIB',
    logo: shibaLogo,
  };
};

const newOpToken = (address: string): TokenOption => {
  return {
    name: 'OP',
    address: address,
    decimals: 18,
    isNative: false,
    rateSymbol: 'OP',
    logo: opLogo,
  };
};

const testAddresses: { [chainId: number]: AddressMapping } = {
  // Sepolia
  11155111: {
    vault: process.env.CONTRACT_SEPOLIA ?? '',
    tokens: [
      ethereumToken,
      newUsdtToken('0x878bfCfbB8EAFA8A2189fd616F282E1637E06bcF', 18),
      newUsdcToken('0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238', 6),
      newEurcToken('0x08210F9170F89Ab7658F0B5E3fF39b0E03C594D4'),
    ],
  },
  // Base Sepolia
  84532: {
    vault: process.env.CONTRACT_BASE_SEPOLIA ?? '',
    tokens: [
      ethereumToken,
      newUsdcToken('0x036CbD53842c5426634e7929541eC2318f3dCF7e', 6),
      newEurcToken('0x808456652fdb597867f38412077A9182bf77359F'),
    ],
  },
  // OP Sepolia
  11155420: {
    vault: process.env.CONTRACT_OP_SEPOLIA ?? '',
    tokens: [
      ethereumToken,
      newUsdcToken('0x5fd84259d66Cd46123540766Be93DFE6D43130D7', 6),
    ],
  },
  // Arbitrum Sepolia
  421614: {
    vault: process.env.CONTRACT_ARBITRUM_SEPOLIA ?? '',
    tokens: [
      ethereumToken,
      newUsdcToken('0x75faf114eafb1BDbe2F0316DF893fd58CE46AA4d', 6),
    ],
  },
  // Mantle Sepolia
  5003: {
    vault: process.env.CONTRACT_MANTLE_SEPOLIA ?? '',
    tokens: [
      {
        name: 'MNT',
        address: '',
        decimals: 18,
        isNative: true,
        rateSymbol: 'MNT',
        logo: mantleLogo,
      },
    ],
  },
};

const prodAddresses: { [chainId: number]: AddressMapping } = {
  // Mainnet
  1: {
    vault: process.env.CONTRACT_MAINNET ?? '',
    tokens: [
      ethereumToken,
      newUsdtToken('0xdAC17F958D2ee523a2206206994597C13D831ec7', 6),
      newUsdcToken('0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48', 6),
      newEurcToken('0x1aBaEA1f7C830bD89Acc67eC4af516284b1bC33c'),
      newDaiToken('0x6B175474E89094C44Da98b954EedeAC495271d0F'),
      newShibaToken('0x95aD61b0a150d79219dCF64E1E6Cc01f0B64C4cE'),
    ],
  },
  // Base
  8453: {
    vault: process.env.CONTRACT_BASE ?? '',
    tokens: [
      ethereumToken,
      newUsdtToken('0xfde4C96c8593536E31F229EA8f37b2ADa2699bb2', 6),
      newUsdcToken('0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913', 6),
      newEurcToken('0x60a3e35cc302bfa44cb288bc5a4f316fdb1adb42'),
      newDaiToken('0x50c5725949A6F0c72E6C4a641F24049A917DB0Cb'),
    ],
  },
  // Optimism
  10: {
    vault: process.env.CONTRACT_OP ?? '',
    tokens: [
      ethereumToken,
      newUsdtToken('0x94b008aA00579c1307B0EF2c499aD98a8ce58e58', 6),
      newUsdcToken('0x0b2c639c533813f4aa9d7837caf62653d097ff85', 6),
      newDaiToken('0xDA10009cBd5D07dd0CeCc66161FC93D7c9000da1'),
      newOpToken('0x4200000000000000000000000000000000000042'),
    ],
  },
  // Arbitrum
  42161: {
    vault: process.env.CONTRACT_ARBITRUM ?? '',
    tokens: [
      ethereumToken,
      newUsdtToken('0xFd086bC7CD5C481DCC9C85ebE478A1C0b69FCbb9', 6),
      newUsdcToken('0xaf88d065e77c8cC2239327C5EDb3A432268e5831', 6),
      newDaiToken('0xDA10009cBd5D07dd0CeCc66161FC93D7c9000da1'),
    ],
  },
};

const addresses =
  process.env.FRONTEND_EVM_ENV === 'production' ? prodAddresses : testAddresses;

export const getEvmTokens = (chainId: number): TokenOption[] => {
  const mapping = addresses[chainId];
  if (!mapping) {
    throw new Error(`No address mapping found for chainId ${chainId}`);
  }

  return mapping.tokens;
};

export const getVaultAddress = (chainId: number): string => {
  const mapping = addresses[chainId];
  if (!mapping || !mapping.vault) {
    throw new Error(`No vault address found for chainId ${chainId}`);
  }

  return addresses[chainId].vault;
};
