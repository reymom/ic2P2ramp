import type { DepositInput, BitcoinOrderInput } from '@/declarations/icramp_backend/icramp_backend.did';
import type { TokenOption } from '@/model/types';
import {
    fetchBitcoinCanisterAddress,
    transferBitcoinToCanister,
    transferRuneToCanister
} from '@/model/blockchain/bitcoin';

export const useOrderBitcoin = () => {
    const makeBitcoinDeposit = async (amount: bigint, token: TokenOption) => {
        if (!(window as any).unisat) throw new Error('Unisat wallet not detected. Please install it.');

        const canisterAddr = await fetchBitcoinCanisterAddress(true);
        if (!canisterAddr) throw new Error('Failed to retrieve Bitcoin canister address.');

        let txid: string;
        if (token.runeMetadata) {
            txid = await transferRuneToCanister(amount, canisterAddr, token.runeMetadata.id);
        } else {
            txid = await transferBitcoinToCanister(amount, canisterAddr);
        }

        const depositInput: [DepositInput] = [{
            Bitcoin: {
                tx_id: txid,
                canister_address: canisterAddr,
            } as BitcoinOrderInput
        }];

        return { depositInput, txid };
    };

    return { makeBitcoinDeposit };
};
