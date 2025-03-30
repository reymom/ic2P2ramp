import { bitcoin_backend } from '@/declarations/bitcoin_backend';
import { RuneMetadata } from '@/declarations/bitcoin_backend/bitcoin_backend.did';
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
  );

  console.log('Transaction ID:', txid);
  return txid;
};

export const transferRuneToCanister = async (
  amount: bigint,
  canisterAddress: string,
  runeId: string,
) => {
  console.log('[transferRuneToCanister] amount = ', amount);
  console.log('[transferRuneToCanister] runeId = ', runeId);
  const txid = await (window as any).unisat.sendRunes(
    canisterAddress,
    runeId,
    amount.toString(),
  );

  console.log('Rune Transaction ID:', txid);
  return txid;
};

const fetchRuneMetadata = async (
  runeId: string,
): Promise<{ metadata: RuneMetadata; serialized: string }> => {
  try {
    const response = await bitcoin_backend.get_serialized_rune_metadata(runeId);
    console.log('[fetchRuneMetadata] response = ', response);
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
      console.log('[fetchBitcoinTokenOptions] metadata = ', metadata);
      console.log('[fetchBitcoinTokenOptions] serialized = ', serialized);
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
      console.error(`Failed to fetch metadata for ${rune.name}:`, error);
    }
  }

  return tokens;
};
