import { bitcoin_backend } from '@/declarations/bitcoin_backend';
import { RuneMetadata } from '@/declarations/bitcoin_backend/bitcoin_backend.did';

const isTaprootAddress = (address: string): boolean => {
  return address.startsWith('bc1p');
};

export const fetchBitcoinCanisterAddress = async (
  useTaproot: boolean,
  isScriptSpend: boolean = false,
) => {
  try {
    let response;

    if (useTaproot) {
      response = isScriptSpend
        ? await bitcoin_backend.get_p2tr_script_spend_address()
        : await bitcoin_backend.get_p2tr_raw_key_spend_address();
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
  const satoshiAmount = Number(amount);
  const txid = await (window as any).unisat.sendBitcoin(
    bitcoinBackendAddress,
    satoshiAmount,
    { feeRate: 55 },
  );

  console.log('Transaction ID:', txid);
  return txid;
};

export const transferRuneToCanister = async (
  amount: bigint,
  canisterAddress: string,
  runeData: string,
) => {
  const satoshiAmount = Number(amount);
  const txid = await (window as any).unisat.sendBitcoin(
    canisterAddress,
    satoshiAmount,
    {
      data: runeData, // Attach the rune metadata as additional data
      feeRate: 55,
    },
  );

  console.log('Rune Transaction ID:', txid);
  return txid;
};

export const fetchRuneMetadata = async (
  symbol: string,
): Promise<{ metadata: RuneMetadata; serialized: string }> => {
  try {
    const response = await bitcoin_backend.get_serialized_rune_metadata(symbol);
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
