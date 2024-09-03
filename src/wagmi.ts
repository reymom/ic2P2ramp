import { getDefaultConfig, connectorsForWallets } from '@rainbow-me/rainbowkit';
import { createConfig } from 'wagmi';
import {
  rainbowWallet,
  walletConnectWallet,
  coinbaseWallet,
  metaMaskWallet,
  injectedWallet,
} from '@rainbow-me/rainbowkit/wallets';
import {
  polygonZkEvmTestnet,
  optimismSepolia,
  baseSepolia,
  sepolia,
  mainnet,
  polygon,
  optimism,
  arbitrum,
} from 'wagmi/chains';

const isTelegramWebView = () => typeof window.Telegram !== 'undefined';

const walletConnectProjectId = '31b7a6907dcc1be39c4d4ca7e4ed20b1';

const testChains = [sepolia, polygonZkEvmTestnet, optimismSepolia, baseSepolia];
const prodChains = [mainnet, polygon, optimism, arbitrum];

const getChains = () => {
  if (process.env.FRONTEND_EVM_ENV === 'production') {
    return prodChains;
  }
  return testChains;
};

const connectors = connectorsForWallets(
  [
    {
      groupName: 'Recommended',
      wallets: [rainbowWallet, walletConnectWallet],
    },
  ],
  {
    appName: 'ic2P2ramp',
    projectId: walletConnectProjectId,
  },
);

export const config = getDefaultConfig({
  appName: 'ic2P2ramp',
  projectId: walletConnectProjectId,
  chains: getChains() as any,
});
