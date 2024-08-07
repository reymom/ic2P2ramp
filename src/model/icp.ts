import { HttpAgent } from '@dfinity/agent';
import { LedgerCanister, AccountIdentifier } from '@dfinity/ledger-icp';
import { Principal } from '@dfinity/principal';

export const transferICPTokensToCanister = async (
  agent: HttpAgent,
  canisterId: string,
  amount: bigint,
  fee: bigint,
) => {
  const ledgerCanister = Principal.fromText(canisterId);
  const ledger = LedgerCanister.create({ agent, canisterId: ledgerCanister });

  if (!process.env.CANISTER_ID_BACKEND) {
    throw new Error('Backend Canister ID not defined in env variables');
  }
  const toAccountIdentifier = AccountIdentifier.fromPrincipal({
    principal: Principal.fromText(process.env.CANISTER_ID_BACKEND),
  });

  try {
    const result = await ledger.transfer({
      to: toAccountIdentifier,
      amount,
      fee,
    });

    console.log('Transfer successful, block index:', result);
    return result;
  } catch (error) {
    console.error('Transfer failed:', error);
    throw error;
  }
};
