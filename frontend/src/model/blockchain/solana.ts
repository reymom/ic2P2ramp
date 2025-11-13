import { TokenInfo } from '@/declarations/solana_backend/solana_backend.did';
import { SPL_TOKEN_LOGOS } from '@/constants/solana_logos';
import { solanaBackend } from '../solanaBackendProxy';
import { TokenOption } from '../types';
import solanaLogo from '@/assets/blockchains/solana-logo.png';
import splGenericIcon from '@/assets/spl_tokens/spl-generic-icon.png';

const RPC_API_BASE =
  process.env.FRONTEND_SOL_ENV === 'devnet'
    ? 'https://solana-devnet.g.alchemy.com/v2/'
    : 'https://solana-mainnet.g.alchemy.com/v2/';

export const SOLANA_RPC_URL =
  process.env.FRONTEND_SOLANA_API_TOKEN && process.env.FRONTEND_SOL_ENV
    ? RPC_API_BASE + process.env.FRONTEND_SOLANA_API_TOKEN
    : 'https://api.devnet.solana.com';

export type Registry = Record<string, TokenInfo>;
const REGISTRY_CACHE_KEY = `sol-registry:${
  process.env.FRONTEND_SOL_ENV || 'devnet'
}`;
const REGISTRY_TTL_MS = 24 * 60 * 60 * 1000; // 24h
let __registryMem: { data: Registry | null; ts: number } = {
  data: null,
  ts: 0,
};
let __registryPending: Promise<Registry> | null = null;

function readLS(): Registry | null {
  if (typeof window === 'undefined') return null;
  const raw = window.localStorage.getItem(REGISTRY_CACHE_KEY);
  if (!raw) return null;
  try {
    const parsed = JSON.parse(raw) as {
      ts: number;
      data: Array<[string, TokenInfo]> | Registry;
    };
    if (Date.now() - parsed.ts > REGISTRY_TTL_MS) return null;
    const map = Array.isArray(parsed.data)
      ? Object.fromEntries(parsed.data as Array<[string, TokenInfo]>)
      : (parsed.data as Registry);
    return map;
  } catch {
    return null;
  }
}

function writeLS(kv: Array<[string, TokenInfo]>) {
  if (typeof window === 'undefined') return;
  window.localStorage.setItem(
    REGISTRY_CACHE_KEY,
    JSON.stringify({ ts: Date.now(), data: kv }),
  );
}

export function invalidateRegisteredSolanaTokensCache() {
  __registryMem = { data: null, ts: 0 };
  if (typeof window !== 'undefined')
    window.localStorage.removeItem(REGISTRY_CACHE_KEY);
}

export const fetchSolanaCanisterAddress = async () => {
  try {
    return await solanaBackend.canister_solana_account();
  } catch (error) {
    console.error('Failed to fetch solana canister address:', error);
    throw error;
  }
};

export async function getRegisteredSolanaTokens(
  forceRefresh = false,
): Promise<Registry> {
  const now = Date.now();

  // in-memory fresh
  if (
    !forceRefresh &&
    __registryMem.data &&
    now - __registryMem.ts < REGISTRY_TTL_MS
  ) {
    return __registryMem.data;
  }

  // localStorage
  if (!forceRefresh) {
    const ls = readLS();
    if (ls) {
      __registryMem = { data: ls, ts: now }; // refresh in-memory clock
      return ls;
    }
  }

  // coalesce concurrent calls
  if (__registryPending && !forceRefresh) return __registryPending;

  __registryPending = solanaBackend
    .get_registered_tokens()
    .then((kv) => {
      const map: Registry = Object.fromEntries(kv);
      __registryMem = { data: map, ts: now };
      writeLS(kv);
      __registryPending = null;
      return map;
    })
    .catch((e) => {
      __registryPending = null;
      if (__registryMem.data) return __registryMem.data;
      throw e;
    });

  return __registryPending;
}

export async function fetchSolanaTokenOptions(): Promise<TokenOption[]> {
  const registry = await getRegisteredSolanaTokens();

  const tokens: TokenOption[] = [
    {
      name: 'SOL',
      address: '',
      decimals: 9,
      isNative: true,
      rateSymbol: 'SOL',
      logo: solanaLogo,
    },
  ];

  for (const [mint, token] of Object.entries(registry)) {
    tokens.push({
      name: token.symbol,
      address: mint,
      decimals: token.decimals,
      isNative: false,
      rateSymbol: token.rate_symbol,
      logo: SPL_TOKEN_LOGOS[token.symbol] || splGenericIcon,
    });
  }

  return tokens;
}
