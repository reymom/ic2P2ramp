import {
  BlockchainAsset,
  FeeQuote,
} from '@/declarations/icramp_backend/icramp_backend.did';
import { backend } from '../backendProxy';

const safeStringify = (v: unknown) =>
  JSON.stringify(v, (_k, val) =>
    typeof val === 'bigint' ? val.toString() : val,
  );

export async function getFeeQuote(
  asset: BlockchainAsset,
  cryptoAmountUnits: bigint,
  opts?: { estimated_gas_withdraw?: bigint },
): Promise<FeeQuote> {
  const res = await backend.calculate_order_fees(
    asset,
    cryptoAmountUnits,
    opts?.estimated_gas_withdraw ? [BigInt(opts.estimated_gas_withdraw)] : [],
  );
  console.log(`[getFeeQuote] res=${safeStringify(res)}`);
  if ('Ok' in res) return res.Ok;
  throw new Error(`Fee quote failed: ${safeStringify(res.Err)}`);
}
