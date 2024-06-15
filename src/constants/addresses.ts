interface AddressMapping {
  native: string;
  usdt: string;
}

const addresses: { [chainId: number]: AddressMapping } = {
  // Mantle Sepolia Testnet
  5003: {
    native: '0x8B1b90637F188541401DeeA100718ca618927E52',
    usdt: '0x67d2d3a45457b69259FB1F8d8178bAE4F6B11b4d',
  },
  // Sepolia
  11155111: {
    native: '0xdaE80C0f07Bc847840f7342a8EC9AD78e695c5a3',
    usdt: '0x878bfCfbB8EAFA8A2189fd616F282E1637E06bcF',
  },
  // Polyzon zkEVM Testnet
  1442: {
    native: '',
    usdt: '',
  },
  // OP Sepolia Testnet
  11155420: {
    native: '',
    usdt: '',
  },
};

export default addresses;
