import { useState } from 'react';
import { useWallet, useConnection } from '@solana/wallet-adapter-react';
import { PublicKey, SystemProgram, Transaction } from '@solana/web3.js';
import {
  getAssociatedTokenAddress,
  createAssociatedTokenAccountInstruction,
  createTransferCheckedInstruction,
  TOKEN_PROGRAM_ID,
  TOKEN_2022_PROGRAM_ID,
} from '@solana/spl-token';

import { fetchSolanaCanisterAddress } from '@/model/blockchain/solana';
import type { DepositInput } from '@/declarations/icramp_backend/icramp_backend.did';
import type { TokenOption } from '@/model/types';
import { NETWORK } from '@/components/SolanaProvider';

export const useOrderSolana = () => {
  const { connection } = useConnection();
  const { publicKey, sendTransaction, wallet } = useWallet();
  const [solflareMismatch, setSolflareMismatch] = useState(false);

  const getMintProgramId = async (mintStr: string) => {
    const mint = new PublicKey(mintStr);
    const info = await connection.getAccountInfo(mint, {
      commitment: 'confirmed',
    });
    if (!info) throw new Error('Mint account not found');
    return info.owner.equals(TOKEN_2022_PROGRAM_ID)
      ? TOKEN_2022_PROGRAM_ID
      : TOKEN_PROGRAM_ID;
  };

  const makeSolanaDeposit = async (amountUnits: bigint, token: TokenOption) => {
    if (!publicKey) throw new Error('Please connect your Solana wallet');

    const canisterAddrStr = await fetchSolanaCanisterAddress();
    const canisterPk = new PublicKey(canisterAddrStr);

    const tx = new Transaction();
    let mintOpt: [] | [string] = [];

    if (token.isNative) {
      // SOL
      const lamports = Number(amountUnits);
      if (!Number.isSafeInteger(lamports)) throw new Error('Amount too large');
      tx.add(
        SystemProgram.transfer({
          fromPubkey: publicKey,
          toPubkey: canisterPk,
          lamports,
        }),
      );
    } else {
      // SPL
      const mintStr = token.address;
      mintOpt = [mintStr];
      const programId = await getMintProgramId(mintStr);
      const mint = new PublicKey(mintStr);
      const fromAta = await getAssociatedTokenAddress(
        mint,
        publicKey,
        false,
        programId,
      );
      const toAta = await getAssociatedTokenAddress(
        mint,
        canisterPk,
        true,
        programId,
      );

      const ataInfo = await connection.getAccountInfo(toAta, {
        commitment: 'confirmed',
      });
      if (!ataInfo) {
        tx.add(
          createAssociatedTokenAccountInstruction(
            publicKey,
            toAta,
            canisterPk,
            mint,
            programId,
          ),
        );
      }

      tx.add(
        createTransferCheckedInstruction(
          fromAta,
          mint,
          toAta,
          publicKey,
          amountUnits,
          token.decimals,
          [],
          programId,
        ),
      );
    }

    const { blockhash } = await connection.getLatestBlockhash('processed');
    tx.recentBlockhash = blockhash;
    tx.feePayer = publicKey;

    let sig: string;
    try {
      sig = await sendTransaction(tx, connection, {
        skipPreflight: false,
        maxRetries: 0,
      });
      setSolflareMismatch(false);
    } catch (e: any) {
      if (/network mismatch/i.test(String(e?.message ?? e)))
        setSolflareMismatch(true);
      throw e;
    }

    const depositInput: [DepositInput] = [
      {
        Solana: {
          signature: sig,
          mint: mintOpt,
        },
      },
    ];

    return { depositInput, txSig: sig };
  };

  const waitForSolanaConfirmation = async (
    sig: string,
    { timeoutMs = 60_000 },
  ) => {
    const start = Date.now();
    while (Date.now() - start < timeoutMs) {
      const st = await connection.getSignatureStatuses([sig], {
        searchTransactionHistory: true,
      });
      const v = st.value[0];

      if (v?.err) {
        throw new Error(
          `Solana tx failed: ${
            typeof v.err === 'string' ? v.err : JSON.stringify(v.err)
          }`,
        );
      }

      const okStatus = v?.confirmationStatus === 'finalized';
      if (okStatus) return;

      await new Promise((r) => setTimeout(r, 1200));
    }
    throw new Error('Timed out waiting for Solana confirmation');
  };

  const solanaNetworkLabel = NETWORK.toString();
  const isSolflare = (wallet?.adapter?.name ?? '')
    .toLowerCase()
    .includes('solflare');

  return {
    makeSolanaDeposit,
    waitForSolanaConfirmation,
    solanaNetworkLabel,
    isSolflare,
    solflareMismatch,
  };
};
