import { HttpAgent } from '@dfinity/agent';
import { IcrcLedgerCanister } from '@dfinity/ledger-icrc';
import { Principal } from '@dfinity/principal';

import { backend } from './backendProxy';
import { rampErrorToString } from './error';

console.log('FRONTEND_ICP_ENV = ', process.env.FRONTEND_ICP_ENV);

export const icpHost =
  process.env.FRONTEND_ICP_ENV === 'test'
    ? 'http://127.0.0.1:8080'
    : 'https://ic0.app';

export const iiUrl =
  process.env.FRONTEND_ICP_ENV === 'production'
    ? `https://identity.ic0.app`
    : `http://${process.env.CANISTER_ID_INTERNET_IDENTITY}.localhost:8080`;

export const fetchIcpTransactionFee = async (ledgerPrincipal: Principal) => {
  try {
    const feeResult = await backend.get_icp_token_info(ledgerPrincipal);
    if ('Ok' in feeResult) {
      return BigInt(feeResult.Ok.fee);
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
  if (!process.env.CANISTER_ID_BACKEND) {
    throw new Error('Backend Canister ID not defined in env variables');
  }

  const ledger = IcrcLedgerCanister.create({ agent, canisterId });
  try {
    const result = await ledger.transfer({
      to: {
        owner: Principal.fromText(process.env.CANISTER_ID_BACKEND),
        subaccount: [],
      },
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
