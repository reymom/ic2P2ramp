import { useEffect } from 'react';
import { ethers } from 'ethers';
import type { TokenOption } from '@/model/types';
import type { BlockchainAsset } from '@/declarations/icramp_backend/icramp_backend.did';

export const useParsedAmount = (
  cryptoAmount: number,
  token: TokenOption | null,
  asset: BlockchainAsset | undefined,
  setUnits: (v: bigint | null) => void,
) => {
  useEffect(() => {
    if (!token || !asset || cryptoAmount <= 0) return;
    try {
      const s = cryptoAmount.toFixed(token.decimals);
      setUnits(ethers.parseUnits(s, token.decimals));
    } catch {
      setUnits(null);
    }
  }, [cryptoAmount, token, asset, setUnits]);
};

export const useAutoClearMessage = (
  message: string | null,
  clear: () => void,
  ms = 20000,
) => {
  useEffect(() => {
    if (!message) return;
    const t = setTimeout(clear, ms);
    return () => clearTimeout(t);
  }, [message, ms, clear]);
};
