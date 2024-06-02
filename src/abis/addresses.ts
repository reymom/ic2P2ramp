interface AddressMapping {
  ZK2Ramp: string;
  USDT: string;
}

const addresses: { [chainId: number]: AddressMapping } = {
  5003: {
    // Mantle Sepolia Testnet
    ZK2Ramp: '0x8B1b90637F188541401DeeA100718ca618927E52',
    USDT: '0x67d2d3a45457b69259FB1F8d8178bAE4F6B11b4d',
  },
  11155111: {
    // Sepolia
    ZK2Ramp: '0xdaE80C0f07Bc847840f7342a8EC9AD78e695c5a3',
    USDT: '0x878bfCfbB8EAFA8A2189fd616F282E1637E06bcF',
  },
  1442: {
    // Polyzon zkEVM Testnet
    ZK2Ramp: '',
    USDT: '',
  },
  11155420: {
    // OP Sepolia Testnet
    ZK2Ramp: '',
    USDT: '',
  },
};

export default addresses;
