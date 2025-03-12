import { formatCryptoUnits } from '@/utils/helper';

const UNISAT_API_TOKEN = process.env.FRONTEND_UNISAT_API_TOKEN || '';
const FRONTEND_BTC_ENV = process.env.FRONTEND_BTC_ENV || 'mainnet';

const UNISAT_API_BASE =
  FRONTEND_BTC_ENV === 'test'
    ? 'https://open-api-testnet4.unisat.io'
    : 'https://open-api.unisat.io';

const fetchFromUnisat = async (
  endpoint: string,
  method: string = 'GET',
  body?: object,
) => {
  try {
    const response = await fetch(`${UNISAT_API_BASE}${endpoint}`, {
      method,
      headers: {
        Authorization: `Bearer ${UNISAT_API_TOKEN}`,
        'Content-Type': 'application/json',
      },
      body: body ? JSON.stringify(body) : undefined,
    });
    console.log('response', response);
    const data = await response.json();
    console.log('data', data);

    if (data.code !== 0) {
      throw new Error(`Unisat API error: ${data.msg}`);
    }

    return data.data;
  } catch (error) {
    console.error(`Error fetching from Unisat (${endpoint}):`, error);
    return null;
  }
};

/**
 * Listen for a transaction until it has been confirmed.
 */
export const waitForTransactionConfirmation = async (
  txid: string,
  maxRetries = 20,
  delay = 10000,
): Promise<boolean> => {
  for (let i = 0; i < maxRetries; i++) {
    const txStatus = await fetchFromUnisat(`/v1/indexer/tx/${txid}`);
    if (txStatus && txStatus.confirmations > 0) {
      console.log(`Transaction ${txid} confirmed.`);
      return true;
    }
    console.log(`Waiting for transaction ${txid} confirmation...`);
    await new Promise((res) => setTimeout(res, delay));
  }
  console.warn(
    `Transaction ${txid} not confirmed after ${maxRetries} retries.`,
  );
  return false;
};

/**
 * Fetch the UTXOs from a confirmed transaction.
 */
export const fetchTransactionOutputs = async (txid: string) => {
  const utxos = await fetchFromUnisat(`/v1/indexer/tx/${txid}/outs`);
  if (!utxos) {
    console.error(`No UTXOs found for transaction ${txid}`);
    return [];
  }
  return utxos;
};

export const fetchRuneUTXOBalance = async (txid: string, vout: number) => {
  const runeBalance = await fetchFromUnisat(
    `/v1/indexer/runes/utxo/${txid}/${vout}/balance`,
  );
  if (!runeBalance) {
    console.error(`No rune balance found for UTXO ${txid}:${vout}`);
    return [];
  }
  return runeBalance;
};

export const fetchRuneBalances = async (
  bitcoinAddress: string,
  runes: { runeId: string; symbol: string; logo: string; name: string }[],
) => {
  const runeBalances: Record<string, any> = {};

  await Promise.all(
    runes.map(async (rune) => {
      try {
        const runeBalanceData = await fetchFromUnisat(
          `/v1/indexer/address/${bitcoinAddress}/runes/${rune.runeId}/balance`,
        );

        console.log('runeBalanceData', runeBalanceData);
        if (runeBalanceData) {
          runeBalances[rune.runeId] = {
            raw: BigInt(runeBalanceData.amount),
            formatted: formatCryptoUnits(
              runeBalanceData.amount / 10 ** runeBalanceData.divisibility,
            ),
            symbol: rune.symbol,
            logo: rune.logo,
            name: rune.name,
          };
        }
      } catch (err) {
        console.error(`Failed to fetch balance for Rune ${rune.runeId}`, err);
      }
    }),
  );

  return runeBalances;
};

export const fetchRunePrice = async (tick: string) => {
  const endpoint = `/v3/market/runes/auction/runes_types_specified`;
  try {
    const data = await fetchFromUnisat(endpoint, 'POST', {
      tick,
      timeType: 'day1',
    });
    return data?.curPrice ?? null;
  } catch (error) {
    console.error(`Error fetching Rune price for ${tick}:`, error);
    return null;
  }
};

interface unisatChains {
  name?: string;
  enum: string;
  unit?: string;
  network?: string;
}

/**
 * Check if the Unisat wallet is on the correct chain based on environment
 * @returns {Promise<boolean>} True if on correct network, false otherwise
 */
export const isCorrectUnisatChain = async (): Promise<boolean> => {
  if (!(window as any).unisat) return false;

  try {
    console.log('[isCorrectUnisatChain]');
    const currentChain: unisatChains = await (window as any).unisat.getChain();
    console.log('currentChain = ', currentChain);
    const requiredChain =
      process.env.FRONTEND_BTC_ENV === 'test'
        ? 'BITCOIN_TESTNET4'
        : 'BITCOIN_MAINNET';

    return currentChain.enum === requiredChain;
  } catch (error) {
    console.error('Error checking Unisat chain:', error);
    return false;
  }
};

/**
 * Switch Unisat wallet to the correct chain based on environment
 * @returns {Promise<boolean>} True if switched successfully, false otherwise
 */
export const switchUnisatChain = async (): Promise<boolean> => {
  if (!(window as any).unisat) return false;

  try {
    const requiredChain =
      process.env.FRONTEND_BTC_ENV === 'test'
        ? 'BITCOIN_TESTNET4'
        : 'BITCOIN_MAINNET';
    await (window as any).unisat.switchChain(requiredChain);
    return true;
  } catch (error) {
    console.error('Error switching Unisat chain:', error);
    return false;
  }
};
