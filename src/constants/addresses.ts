interface AddressMapping {
  native: string;
  usdt: string;
}

const addresses: { [chainId: number]: AddressMapping } = {
  // Sepolia
  11155111: {
    native: '0x42ad57ab757ea55960f7d9805d82fa818683096b',
    usdt: '0x878bfCfbB8EAFA8A2189fd616F282E1637E06bcF',
  },
  // Base Sepolia
  84532: {
    native: '0xfa29381958DD8a2dD86246FC0Ab2932972640580',
    usdt: '0x67d2d3a45457b69259FB1F8d8178bAE4F6B11b4d',
  },
  // Polyzon zkEVM Cardona
  2442: {
    native: '0x9025e74D23384f664CfEB07F1d8ABd19570758B5',
    usdt: '',
  },
  // OP Sepolia
  11155420: {
    native: '0x9025e74D23384f664CfEB07F1d8ABd19570758B5',
    usdt: '',
  },
};

export default addresses;
