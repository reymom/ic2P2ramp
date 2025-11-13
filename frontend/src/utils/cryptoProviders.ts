import { Principal } from '@dfinity/principal';

import { fetchSolanaTokenOptions } from '@/model/blockchain/solana';
import { getEvmTokens } from '@/constants/evm_tokens';
import { ICP_TOKENS } from '@/constants/icp_tokens';
import { NetworkIds } from '@/constants/networks';
import {
  BlockchainAsset,
  PaymentProvider,
  TransactionAddress,
} from '@/declarations/icramp_backend/icramp_backend.did';

export const EVM_CHAIN_IDS = Object.values(NetworkIds).map((n) => n.id);

export const hasEvm = (addrs: TransactionAddress[]) =>
  addrs.some((a) => 'EVM' in a.address_type);
export const hasSol = (addrs: TransactionAddress[]) =>
  addrs.some((a) => 'Solana' in a.address_type);
export const hasIcp = (addrs: TransactionAddress[]) =>
  addrs.some((a) => 'ICP' in a.address_type);

export const deepEqual = (a: any, b: any): boolean => {
  if (a === b) return true;
  if (typeof a === 'bigint' && typeof b === 'bigint') return a === b;
  if (
    typeof a !== 'object' ||
    typeof b !== 'object' ||
    a === null ||
    b === null
  )
    return false;
  if (Array.isArray(a) && Array.isArray(b)) {
    if (a.length !== b.length) return false;
    for (let i = 0; i < a.length; i++) {
      if (!deepEqual(a[i], b[i])) return false;
    }
    return true;
  }
  const keysA = Object.keys(a);
  const keysB = Object.keys(b);
  if (keysA.length !== keysB.length) return false;
  for (const key of keysA) {
    if (!keysB.includes(key) || !deepEqual(a[key], b[key])) return false;
  }
  return true;
};

export const getRecvAddr = (
  addrs: TransactionAddress[],
  kind: 'EVM' | 'Solana' | 'ICP',
): TransactionAddress | null => {
  if (kind === 'EVM') {
    return addrs.find((a) => 'EVM' in a.address_type) ?? null;
  }
  if (kind === 'Solana') {
    return addrs.find((a) => 'Solana' in a.address_type) ?? null;
  }
  return addrs.find((a) => 'ICP' in a.address_type) ?? null;
};

export const buildEvmProviders = async (
  recv: TransactionAddress,
  symbol: 'USDC' | 'EURC',
): Promise<PaymentProvider[]> => {
  const out: PaymentProvider[] = [];
  for (const cid of EVM_CHAIN_IDS) {
    try {
      const tokens = getEvmTokens(cid);
      const t = tokens.find(
        (tt) => tt.name.toUpperCase() === symbol && !tt.isNative,
      );
      if (!t) continue;
      out.push({
        Crypto: {
          asset: { EVM: { chain_id: BigInt(cid), token_address: [t.address] } },
          address: recv,
        },
      });
    } catch {
      // unknown chain mapping → skip
    }
  }
  return out;
};

export const buildSolProvider = async (
  recv: TransactionAddress,
  symbol: 'USDC' | 'EURC',
): Promise<PaymentProvider | null> => {
  const registry = await fetchSolanaTokenOptions();
  const t = registry.find((tt) => tt.name.toUpperCase() === symbol);
  if (!t) return null;
  return {
    Crypto: {
      asset: { Solana: { spl_token: [t.address] } },
      address: recv,
    },
  };
};

export const buildIcpProvider = async (
  recv: TransactionAddress,
): Promise<PaymentProvider | null> => {
  const ckUsd = ICP_TOKENS.find((t) => t.name.toUpperCase() === 'CKUSDT');
  if (!ckUsd) return null;
  return {
    Crypto: {
      asset: { ICP: { ledger_principal: Principal.from(ckUsd.address) } },
      address: recv,
    },
  };
};

/* ---------- group EVM for UI ---------- */
export type EvmGroup = {
  symbol: 'USDC' | 'EURC';
  address: TransactionAddress;
  chains: number[];
  providers: PaymentProvider[];
  logo: string;
};

export const groupEvmProviders = (all: PaymentProvider[]): EvmGroup[] => {
  const map = new Map<string, EvmGroup>();

  for (const p of all) {
    if (!('Crypto' in p) || !('EVM' in p.Crypto.asset)) continue;
    const cid = Number(p.Crypto.asset.EVM.chain_id);
    const token = p.Crypto.asset.EVM.token_address?.[0];
    if (!token) continue;

    let symbol: 'USDC' | 'EURC' | undefined;
    let logo: string = '';
    try {
      const t = getEvmTokens(cid).find(
        (tt) => tt.address.toLowerCase() === token.toLowerCase(),
      );
      if (!t) continue;
      const n = t.name.toUpperCase();
      if (n === 'USDC' || n === 'EURC') symbol = n as 'USDC' | 'EURC';
      logo = t.logo;
    } catch {}
    if (!symbol) continue;

    const key = `${symbol}|${JSON.stringify(p.Crypto.address)}`;
    const existing = map.get(key) ?? {
      symbol,
      address: p.Crypto.address,
      chains: [],
      providers: [],
      logo: logo,
    };
    if (!existing.chains.includes(cid)) existing.chains.push(cid);
    existing.providers.push(p);
    map.set(key, existing);
  }

  return Array.from(map.values()).map((g) => ({
    ...g,
    chains: g.chains.sort(
      (a, b) => EVM_CHAIN_IDS.indexOf(a) - EVM_CHAIN_IDS.indexOf(b),
    ),
  }));
};

export const sameCryptoAsset = (
  a: BlockchainAsset,
  b: BlockchainAsset,
): boolean => {
  if (!a || !b) return false;

  if ('EVM' in a && 'EVM' in b) {
    if (a.EVM.chain_id !== b.EVM.chain_id) return false;
    const addrA = a.EVM.token_address?.[0] ?? '';
    const addrB = b.EVM.token_address?.[0] ?? '';
    return addrA.toLowerCase() === addrB.toLowerCase();
  }

  if ('Solana' in a && 'Solana' in b) {
    const mintA = a.Solana.spl_token?.[0] ?? '';
    const mintB = b.Solana.spl_token?.[0] ?? '';
    return mintA === mintB;
  }

  if ('ICP' in a && 'ICP' in b) {
    try {
      const pA = a.ICP.ledger_principal;
      const pB = b.ICP.ledger_principal;
      const tA = typeof pA.toText === 'function' ? pA.toText() : String(pA);
      const tB = typeof pB.toText === 'function' ? pB.toText() : String(pB);
      return tA === tB;
    } catch {
      return false;
    }
  }

  if ('Bitcoin' in a && 'Bitcoin' in b) {
    const rA = a.Bitcoin.rune_id?.[0] ?? '';
    const rB = b.Bitcoin.rune_id?.[0] ?? '';
    return rA === rB;
  }

  return false;
};
