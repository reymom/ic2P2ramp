import { getDefaultConfig } from '@rainbow-me/rainbowkit';
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

const walletConnectProjectId = '31b7a6907dcc1be39c4d4ca7e4ed20b1';

const testChains = [sepolia, polygonZkEvmTestnet, optimismSepolia, baseSepolia];
const prodChains = [mainnet, polygon, optimism, arbitrum];

const getChains = () => {
  if (process.env.FRONTEND_ENV === 'production') {
    return prodChains;
  }
  return testChains;
};

export const config = getDefaultConfig({
  appName: 'ic2P2ramp',
  projectId: walletConnectProjectId,
  chains: getChains() as any,
});
