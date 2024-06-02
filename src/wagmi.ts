import { getDefaultConfig } from '@rainbow-me/rainbowkit';
import {
  polygonZkEvmTestnet,
  optimismSepolia,
  mantleSepoliaTestnet,
  sepolia,
} from 'wagmi/chains';

const walletConnectProjectId = '31b7a6907dcc1be39c4d4ca7e4ed20b1';

export const config = getDefaultConfig({
  appName: 'p2Ploan',
  projectId: walletConnectProjectId,
  chains: [polygonZkEvmTestnet, optimismSepolia, mantleSepoliaTestnet, sepolia],
});
