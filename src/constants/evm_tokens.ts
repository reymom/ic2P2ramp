import { TokenOption } from '../model/types';

import ethereumLogo from '../assets/blockchains/ethereum-logo.png';
import mantleLogo from '../assets/blockchains/mantle.png';
import usdcLogo from '../assets/blockchains/usdc-logo.png';
import usdtLogo from '../assets/blockchains/tether-usdt-seeklogo.svg';

export const defaultCommitEvmGas = BigInt(80000);
export const defaultReleaseEvmGas = BigInt(100000);

// ----------------------
// EVM Tokens Definitions
// ----------------------

interface AddressMapping {
  vault: string;
  tokens: TokenOption[];
}

if (
  !process.env.CONTRACT_SEPOLIA ||
  !process.env.CONTRACT_BASE_SEPOLIA ||
  !process.env.CONTRACT_OP_SEPOLIA ||
  !process.env.CONTRACT_MANTLE_SEPOLIA
) {
  console.error('Contract addresses not defined');
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
    rateSymbol: 'USDT',
    logo: usdtLogo,
  };
};

const newUsdcToken = (address: string, decimals: number): TokenOption => {
  return {
    name: 'USDC',
    address: address,
    decimals: decimals,
    isNative: false,
    rateSymbol: 'USDC',
    logo: usdcLogo,
  };
};

const testAddresses: { [chainId: number]: AddressMapping } = {
  // Sepolia
  11155111: {
    vault: process.env.CONTRACT_SEPOLIA ?? '',
    tokens: [
      ethereumToken,
      newUsdtToken('0x878bfCfbB8EAFA8A2189fd616F282E1637E06bcF', 18),
    ],
  },
  // Base Sepolia
  84532: {
    vault: process.env.CONTRACT_BASE_SEPOLIA
      ? process.env.CONTRACT_BASE_SEPOLIA
      : '',
    tokens: [
      ethereumToken,
      newUsdcToken('0x036CbD53842c5426634e7929541eC2318f3dCF7e', 6),
    ],
  },
  // OP Sepolia
  11155420: {
    vault: process.env.CONTRACT_OP_SEPOLIA
      ? process.env.CONTRACT_OP_SEPOLIA
      : '',
    tokens: [ethereumToken],
  },
  // Mantle Sepolia
  5003: {
    vault: process.env.CONTRACT_MANTLE_SEPOLIA
      ? process.env.CONTRACT_MANTLE_SEPOLIA
      : '',
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
  1: {
    vault: '0x...',
    tokens: [ethereumToken],
  },
  137: {
    vault: '0x...',
    tokens: [ethereumToken],
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
