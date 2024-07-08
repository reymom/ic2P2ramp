interface AddressMapping {
  native: [string, string];
  usdt: [string, string];
}

const testAddresses: { [chainId: number]: AddressMapping } = {
  // Sepolia
  11155111: {
    native: ['ETH', '0x42ad57ab757ea55960f7d9805d82fa818683096b'],
    usdt: ['USDT', '0x878bfCfbB8EAFA8A2189fd616F282E1637E06bcF'],
  },
  // Base Sepolia
  84532: {
    native: ['ETH', '0xfa29381958DD8a2dD86246FC0Ab2932972640580'],
    usdt: ['USDT', '0x67d2d3a45457b69259FB1F8d8178bAE4F6B11b4d'],
  },
  // Polyzon zkEVM Cardona
  2442: {
    native: ['MATIC', '0x9025e74D23384f664CfEB07F1d8ABd19570758B5'],
    usdt: ['USDT', ''],
  },
  // OP Sepolia
  11155420: {
    native: ['OP', '0x9025e74D23384f664CfEB07F1d8ABd19570758B5'],
    usdt: ['USDT', ''],
  },
};

const prodAddresses: { [chainId: number]: AddressMapping } = {
  1: {
    native: ['ETH', '0x...'],
    usdt: ['USDT', '0x...'],
  },
  137: {
    native: ['MATIC', '0x...'],
    usdt: ['USDT', '0x...'],
  },
};

export const addresses =
  process.env.FRONTEND_ENV === 'production' ? prodAddresses : testAddresses;

export interface TokenOption {
  name: string;
  address: string;
  isNative: boolean;
}

export const getTokenOptions = (chainId: number): TokenOption[] => {
  const mapping = addresses[chainId];

  if (!mapping) {
    throw new Error(`No address mapping found for chainId ${chainId}`);
  }

  return [
    { name: mapping.native[0], address: mapping.native[1], isNative: true },
    { name: mapping.usdt[0], address: mapping.usdt[1], isNative: false },
  ];
};
