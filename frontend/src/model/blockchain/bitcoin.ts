import { bitcoin_backend } from '@/declarations/bitcoin_backend';
import { RuneMetadata } from '@/declarations/bitcoin_backend/bitcoin_backend.did';
import {
  fetchRuneUTXOBalance,
  fetchTransactionOutputs,
  waitForTransactionConfirmation,
} from './unisat';
import { RuneUTXOEntry } from '@/declarations/backend/backend.did';
import { TokenOption } from '@/model/types';
import { supportedRuneIds } from '@/constants/runes';
import bitcoinLogo from '@/assets/blockchains/bitcoin-logo.svg';

export const fetchBitcoinCanisterAddress = async (useTaproot: boolean) => {
  try {
    let response;

    if (useTaproot) {
      response = await bitcoin_backend.get_p2tr_raw_key_spend_address();
    } else {
      response = await bitcoin_backend.get_p2pkh_address();
    }

    if ('Ok' in response) {
      return response.Ok;
    }

    if ('Err' in response) {
      const error = response.Err;
      const [errorType, errorMessage] = Object.entries(error)[0];
      throw new Error(`${errorType}: ${JSON.stringify(errorMessage)}`);
    }

    throw new Error('Unexpected response format');
  } catch (error) {
    console.error('Failed to fetch bitcoin canister address:', error);
    throw error;
  }
};

export const transferBitcoinToCanister = async (
  amount: bigint,
  bitcoinBackendAddress: String,
) => {
  const txid = await (window as any).unisat.sendBitcoin(
    bitcoinBackendAddress,
    Number(amount),
    { feeRate: 55 },
  );

  console.log('Transaction ID:', txid);
  return txid;
};

export const transferRuneToCanister = async (
  amount: bigint,
  canisterAddress: string,
  runeId: string,
) => {
  const txid = await (window as any).unisat.sendRunes(
    canisterAddress,
    runeId,
    Number(amount),
    {
      feeRate: 55,
    },
  );

  console.log('Rune Transaction ID:', txid);
  return txid;
};

export const handleBitcoinTransaction = async (
  txid: string,
  isRune: boolean,
): Promise<Array<RuneUTXOEntry> | []> => {
  console.log(`Waiting for Bitcoin transaction ${txid} confirmation...`);
  const confirmed = await waitForTransactionConfirmation(txid);
  if (!confirmed) {
    console.error(`Transaction ${txid} not confirmed.`);
    return [];
  }

  if (!isRune) return [];

  console.log(`Fetching UTXOs for transaction ${txid}...`);
  const utxos = await fetchTransactionOutputs(txid);
  if (!Array.isArray(utxos) || utxos.length === 0) {
    console.error(`No UTXOs found for transaction ${txid}`);
    return [];
  }

  const runeUTXOs: RuneUTXOEntry[] = [];
  for (const utxo of utxos) {
    try {
      const runeData = await fetchRuneUTXOBalance(utxo.txid, utxo.vout);
      if (Array.isArray(runeData) && runeData.length > 0) {
        runeUTXOs.push({
          txid: utxo.txid,
          vout: utxo.vout,
          rune_amount: BigInt(runeData[0].amount),
          script_pubkey: utxo.scriptPk,
        });
      }
    } catch (error) {
      console.error(
        `Failed to fetch Rune balance for ${utxo.txid}:${utxo.vout}: ${error}`,
      );
    }
  }

  return runeUTXOs;
};

const fetchRuneMetadata = async (
  runeId: string,
): Promise<{ metadata: RuneMetadata; serialized: string }> => {
  try {
    const response = await bitcoin_backend.get_serialized_rune_metadata(runeId);
    if ('Ok' in response && response.Ok.length === 2) {
      return { metadata: response.Ok[0], serialized: response.Ok[1] };
    } else if ('Err' in response) {
      const error = response.Err;
      const [errorType, errorMessage] = Object.entries(error)[0];
      throw new Error(`${errorType}: ${JSON.stringify(errorMessage)}`);
    }
    throw new Error('Unexpected response format');
  } catch (error) {
    console.error('Failed to fetch rune metadata:', error);
    throw error;
  }
};

export const fetchBitcoinTokenOptions = async (): Promise<
  Array<TokenOption>
> => {
  const tokens: TokenOption[] = [
    {
      name: 'BTC',
      address: '',
      decimals: 8,
      isNative: true,
      rateSymbol: 'BTC',
      logo: bitcoinLogo,
    },
  ];

  for (const rune of supportedRuneIds) {
    try {
      const { metadata, serialized } = await fetchRuneMetadata(rune.runeId);
      tokens.push({
        name: rune.name,
        address: serialized,
        decimals: metadata.divisibility,
        isNative: false,
        rateSymbol: metadata.name,
        logo: rune.logo,
        runeMetadata: metadata,
      });
    } catch (error) {
      console.error(
        `Failed to fetch metadata for ${MediaMetadata.name}:`,
        error,
      );
    }
  }

  return tokens;
};
