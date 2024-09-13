import { getDefaultConfig } from '@rainbow-me/rainbowkit';
import {
  sepolia,
  baseSepolia,
  optimismSepolia,
  mantleSepoliaTestnet,
  mainnet,
  base,
  optimism,
  mantle,
  arbitrum,
  polygon,
} from 'wagmi/chains';

// const isTelegramWebView = () => typeof window.Telegram !== 'undefined';

const walletConnectProjectId = '31b7a6907dcc1be39c4d4ca7e4ed20b1';

const testChains = [
  sepolia,
  baseSepolia,
  optimismSepolia,
  mantleSepoliaTestnet,
];
const prodChains = [mainnet, base, optimism, mantle, arbitrum, polygon];

const getChains = () => {
  if (process.env.FRONTEND_EVM_ENV === 'production') {
    return prodChains;
  }
  return testChains;
};

export const config = getDefaultConfig({
  appName: 'ic2P2ramp',
  projectId: walletConnectProjectId,
  chains: getChains() as any,
});
