const UNISAT_API_TOKEN = process.env.UNISAT_API_TOKEN || "";
const FRONTEND_BTC_ENV = process.env.FRONTEND_BTC_ENV || "mainnet";

const UNISAT_API_BASE =
  FRONTEND_BTC_ENV === "test"
    ? "https://open-api-testnet.unisat.io"
    : "https://open-api.unisat.io";

const fetchFromUnisat = async (endpoint: string) => {
  try {
    const response = await fetch(`${UNISAT_API_BASE}${endpoint}`, {
      headers: {
        Authorization: `Bearer ${UNISAT_API_TOKEN}`,
        "Content-Type": "application/json",
      },
    });
    const data = await response.json();

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
  delay = 10000
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
  console.warn(`Transaction ${txid} not confirmed after ${maxRetries} retries.`);
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

export const fetchRuneBalance = async (txid: string, vout: number) => {
  const runeBalance = await fetchFromUnisat(`/v1/indexer/runes/utxo/${txid}/${vout}/balance`);
  if (!runeBalance) {
    console.error(`No rune balance found for UTXO ${txid}:${vout}`);
    return [];
  }
  return runeBalance;
};
