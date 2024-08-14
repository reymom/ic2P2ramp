import { HttpAgent } from '@dfinity/agent';
import { LedgerCanister, AccountIdentifier } from '@dfinity/ledger-icp';
import { Principal } from '@dfinity/principal';
import { backend } from '../declarations/backend';
import { rampErrorToString } from './error';

export const fetchIcpTransactionFee = async (ledgerPrincipal: Principal) => {
  try {
    const feeResult = await backend.get_icp_transaction_fee(ledgerPrincipal);
    if ('Ok' in feeResult) {
      return BigInt(feeResult.Ok);
    } else {
      throw new Error(rampErrorToString(feeResult.Err));
    }
  } catch (error) {
    console.error('Failed to fetch ICP transaction fee:', error);
    throw error;
  }
};

export const transferICPTokensToCanister = async (
  agent: HttpAgent,
  canisterId: Principal,
  amount: bigint,
  fee: bigint,
) => {
  const ledger = LedgerCanister.create({ agent, canisterId });

  if (!process.env.CANISTER_ID_BACKEND) {
    throw new Error('Backend Canister ID not defined in env variables');
  }
  const toAccountIdentifier = AccountIdentifier.fromPrincipal({
    principal: Principal.fromText(process.env.CANISTER_ID_BACKEND),
  });

  try {
    const result = await ledger.transfer({
      to: toAccountIdentifier,
      amount: amount + fee,
      fee,
    });

    console.log('Transfer successful, block index:', result);
    return result;
  } catch (error) {
    console.error('Transfer failed:', error);
    throw error;
  }
};
