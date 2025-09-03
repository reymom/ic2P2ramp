import { Principal } from '@dfinity/principal';
import type { HttpAgent } from '@dfinity/agent';
import type { TokenOption } from '@/model/types';
import { fetchIcpTransactionFee, transferICPTokensToCanister } from '@/model/blockchain/icp';

export const useOrderIcp = () => {
    const makeIcpDeposit = async (agent: HttpAgent, token: TokenOption, amount: bigint) => {
        const ledger = Principal.fromText(token.address);
        const fees = await fetchIcpTransactionFee(ledger);
        const result = await transferICPTokensToCanister(agent, ledger, amount, fees);
        return { result };
    };
    return { makeIcpDeposit };
};
