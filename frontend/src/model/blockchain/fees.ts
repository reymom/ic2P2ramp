import {
  BlockchainAsset,
  FeeQuote,
} from '@/declarations/icramp_backend/icramp_backend.did';
import { backend } from '../backendProxy';

export async function getFeeQuote(
  asset: BlockchainAsset,
  cryptoAmountUnits: bigint,
  opts?: { estimated_gas_lock?: bigint; estimated_gas_withdraw?: bigint },
): Promise<FeeQuote> {
  const res = await backend.calculate_order_fees(
    asset,
    cryptoAmountUnits,
    opts?.estimated_gas_lock ? [BigInt(opts.estimated_gas_lock)] : [],
    opts?.estimated_gas_withdraw ? [BigInt(opts.estimated_gas_withdraw)] : [],
  );
  if ('Ok' in res) {
    const q = res.Ok as any;
    return {
      blockchain_fee: BigInt(q.blockchain_fee),
      admin_fee: BigInt(q.admin_fee),
      total_fee: BigInt(q.total_fee),
    };
  }
  throw new Error(`Fee quote failed: ${JSON.stringify(res.Err)}`);
}
