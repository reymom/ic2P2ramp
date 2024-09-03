interface AddressMapping {
  vault: string;
  native: string;
  usdt: [string, string];
}

const testAddresses: { [chainId: number]: AddressMapping } = {
  // Sepolia
  11155111: {
    vault: '0x42ad57ab757ea55960f7d9805d82fa818683096b',
    native: 'ETH',
    usdt: ['USDT', '0x878bfCfbB8EAFA8A2189fd616F282E1637E06bcF'],
  },
  // Base Sepolia
  84532: {
    vault: '0xfa29381958DD8a2dD86246FC0Ab2932972640580',
    native: 'ETH',
    usdt: ['USDT', '0x67d2d3a45457b69259FB1F8d8178bAE4F6B11b4d'],
  },
  // Polyzon zkEVM Cardona
  2442: {
    vault: '0x9025e74D23384f664CfEB07F1d8ABd19570758B5',
    native: 'MATIC',
    usdt: ['USDT', ''],
  },
  // OP Sepolia
  11155420: {
    vault: '0x9025e74D23384f664CfEB07F1d8ABd19570758B5',
    native: 'OP',
    usdt: ['USDT', ''],
  },
};

const prodAddresses: { [chainId: number]: AddressMapping } = {
  1: {
    vault: '0x...',
    native: 'ETH',
    usdt: ['USDT', '0x...'],
  },
  137: {
    vault: '0x...',
    native: 'MATIC',
    usdt: ['USDT', '0x...'],
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
