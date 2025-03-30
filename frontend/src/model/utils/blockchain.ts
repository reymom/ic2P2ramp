import { NetworkIds } from '@/constants/networks';

const FRONTEND_ICP_ENV = process.env.FRONTEND_ICP_ENV || 'test';
const FRONTEND_BTC_ENV = process.env.FRONTEND_BTC_ENV || 'mainnet';

interface ExplorerUrls {
  address: string;
  transaction?: string;
}

/**
 * Get the blockchain explorer URL for a given user login.
 */
export const getExplorerUrls = (
  blockchainType: string,
  address: string,
  chain_id?: bigint,
  txHash?: string,
): ExplorerUrls | null => {
  switch (blockchainType) {
    case 'EVM': {
      if (!chain_id) return null;
      const network = Object.values(NetworkIds).find(
        (n) => n.id === Number(chain_id),
      );
      if (!network) return null;
      return {
        address: `${network.explorer}/address/${address}`,
        transaction: txHash ? `${network.explorer}/tx/${txHash}` : undefined,
      };
    }

    case 'ICP': {
      if (FRONTEND_ICP_ENV === 'production') {
        return {
          address: `https://www.icpexplorer.org/#/acct/${address}`,
          transaction: txHash
            ? `https://www.icpexplorer.org/#/tx/${txHash}`
            : undefined,
        };
      }
      return null;
    }

    case 'Bitcoin': {
      const baseExplorer =
        FRONTEND_BTC_ENV === 'test'
          ? 'https://mempool.space/testnet4'
          : 'https://mempool.space';
      return {
        address: `${baseExplorer}/address/${address}`,
        transaction: txHash ? `${baseExplorer}/tx/${txHash}` : undefined,
      };
    }

    case 'Solana': {
      return {
        address: `https://solscan.io/address/${address}`,
        transaction: txHash ? `https://solscan.io/tx/${txHash}` : undefined,
      };
    }
    default:
      return null;
  }
};
