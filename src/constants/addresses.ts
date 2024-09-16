import ethereumLogo from '../assets/blockchains/ethereum-logo.png';
import polygonMaticLogo from '../assets/blockchains/polygon-matic.svg';
import mantleLogo from '../assets/blockchains/mantle.png';
import usdcLogo from '../assets/blockchains/usdc-logo.png';
import usdtLogo from '../assets/blockchains/tether-usdt-seeklogo.svg';

import icpLogo from '../assets/blockchains/icp-logo.svg';
import ckBTCLogo from '../assets/blockchains/ckBTC-logo.svg';

export interface TokenMapping {
  name: string;
  address: string;
  decimals: number;
}

interface AddressMapping {
  vault: string;
  native: TokenMapping;
  usdt: TokenMapping;
  usdc: TokenMapping;
}

if (
  !process.env.CONTRACT_SEPOLIA ||
  !process.env.CONTRACT_BASE_SEPOLIA ||
  !process.env.CONTRACT_OP_SEPOLIA ||
  !process.env.CONTRACT_MANTLE_SEPOLIA
) {
  console.error('Contract addresses not defined');
}

const testAddresses: { [chainId: number]: AddressMapping } = {
  // Sepolia
  11155111: {
    vault: process.env.CONTRACT_SEPOLIA ? process.env.CONTRACT_SEPOLIA : '',
    native: { name: 'ETH', address: 'native', decimals: 18 },
    usdt: {
      name: 'USDT',
      address: '0x878bfCfbB8EAFA8A2189fd616F282E1637E06bcF',
      decimals: 18,
    },
    usdc: { name: 'USDC', address: '', decimals: 6 },
  },
  // Base Sepolia
  84532: {
    vault: process.env.CONTRACT_BASE_SEPOLIA
      ? process.env.CONTRACT_BASE_SEPOLIA
      : '',
    native: { name: 'ETH', address: 'native', decimals: 18 },
    usdt: {
      name: 'USDT',
      address: '',
      decimals: 18,
    },
    usdc: {
      name: 'USDC',
      address: '0x036CbD53842c5426634e7929541eC2318f3dCF7e',
      decimals: 6,
    },
  },
  // OP Sepolia
  11155420: {
    vault: process.env.CONTRACT_OP_SEPOLIA
      ? process.env.CONTRACT_OP_SEPOLIA
      : '',
    native: { name: 'ETH', address: 'native', decimals: 18 },
    usdt: { name: 'USDT', address: '', decimals: 6 },
    usdc: { name: 'USDC', address: '', decimals: 6 },
  },
  // Mantle Sepolia
  5003: {
    vault: process.env.CONTRACT_MANTLE_SEPOLIA
      ? process.env.CONTRACT_MANTLE_SEPOLIA
      : '',
    native: { name: 'MNT', address: 'native', decimals: 18 },
    usdt: { name: 'USDT', address: '', decimals: 6 },
    usdc: { name: 'USDC', address: '', decimals: 6 },
  },
};

const prodAddresses: { [chainId: number]: AddressMapping } = {
  1: {
    vault: '0x...',
    native: { name: 'ETH', address: 'native', decimals: 18 },
    usdt: { name: 'USDT', address: '', decimals: 6 },
    usdc: { name: 'USDC', address: '', decimals: 6 },
  },
  137: {
    vault: '0x...',
    native: { name: 'MATIC', address: 'native', decimals: 18 },
    usdt: { name: 'USDT', address: '', decimals: 6 },
    usdc: { name: 'USDC', address: '', decimals: 6 },
  },
};

export const addresses =
  process.env.FRONTEND_EVM_ENV === 'production' ? prodAddresses : testAddresses;

export const getVaultAddress = (chainId: number): string => {
  return addresses[chainId].vault;
};

export const tokenCanisters = {
  ICP: 'ryjl3-tyaaa-aaaaa-aaaba-cai',
  OpenChat: '',
  ckBTC:
    process.env.FRONTEND_ICP_ENV === 'production'
      ? 'mxzaz-hqaaa-aaaar-qaada-cai'
      : 'mc6ru-gyaaa-aaaar-qaaaq-cai',
};

export const tokenLogos: { [token_name: string]: string } = {
  ETH: ethereumLogo,
  MNT: mantleLogo,
  MATIC: polygonMaticLogo,
  USDT: usdtLogo,
  USDC: usdcLogo,
  ICP: icpLogo,
  ckBTC: ckBTCLogo,
};
