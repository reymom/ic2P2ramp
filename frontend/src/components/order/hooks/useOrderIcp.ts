import { Principal } from '@dfinity/principal';
import type { HttpAgent } from '@dfinity/agent';
import type { TokenOption } from '@/model/types';
import {
  fetchIcpTransactionFee,
  transferICPTokensToCanister,
} from '@/model/blockchain/icp';
import { getBackendCanisterId } from '@/constants/canisters';

export const useOrderIcp = () => {
  const makeIcpDeposit = async (
    agent: HttpAgent,
    token: TokenOption,
    amount: bigint,
  ) => {
    const ledger = Principal.fromText(token.address);
    const fees = await fetchIcpTransactionFee(ledger);
    const result = await transferICPTokensToCanister(
      agent,
      ledger,
      amount,
      fees,
      getBackendCanisterId(),
    );
    return { result };
  };

  const makeIcpPayment = async (
    agent: HttpAgent,
    token: TokenOption,
    amount: bigint,
    destination: string,
  ) => {
    const ledger = Principal.fromText(token.address);
    const fees = await fetchIcpTransactionFee(ledger);

    const result = await transferICPTokensToCanister(
      agent,
      ledger,
      amount,
      fees,
      destination,
    );

    return { result };
  };

  return { makeIcpDeposit, makeIcpPayment };
};
