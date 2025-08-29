import { TokenInfo } from '@/declarations/solana_backend/solana_backend.did';
import { solanaBackend } from '../solanaBackendProxy';

const RPC_API_BASE =
  process.env.FRONTEND_SOL_ENV === 'devnet'
    ? 'https://solana-devnet.g.alchemy.com/v2/'
    : 'https://solana-mainnet.g.alchemy.com/v2/';

export const SOLANA_RPC_URL =
  process.env.FRONTEND_SOLANA_API_TOKEN && process.env.FRONTEND_SOL_ENV
    ? RPC_API_BASE + process.env.FRONTEND_SOLANA_API_TOKEN
    : 'https://api.devnet.solana.com';

export const fetchSolanaCanisterAddress = async () => {
  try {
    return await solanaBackend.canister_solana_account();
  } catch (error) {
    console.error('Failed to fetch solana canister address:', error);
    throw error;
  }
};

export async function getRegisteredSolanaTokens(): Promise<
  Record<string, TokenInfo>
> {
  const raw: Array<[string, TokenInfo]> =
    await solanaBackend.get_registered_tokens();

  const out: Record<string, TokenInfo> = {};
  for (const [mint, info] of raw) {
    out[mint] = info;
  }
  return out;
}
