import { OrderState } from '@/declarations/backend/backend.did';

// Function to convert string numbers in JSON to bigint where needed
export const parseBigIntFields = (order: any): OrderState => {
    if (order.Created) {
        return {
            Created: {
                ...order.Created,
                id: BigInt(order.Created.id),
                created_at: BigInt(order.Created.created_at),
                offramper_user_id: BigInt(order.Created.offramper_user_id),
                crypto: {
                    ...order.Created.crypto,
                    fee: BigInt(order.Created.crypto.fee),
                    amount: BigInt(order.Created.crypto.amount),
                    blockchain: {
                        EVM: { chain_id: BigInt(order.Created.crypto.blockchain.EVM.chain_id) }
                    }
                }
            }
        };
    }
    if (order.Locked) {
        return {
            Locked: {
                ...order.Locked,
                locked_at: BigInt(order.Locked.locked_at),
                offramper_fee: BigInt(order.Locked.offramper_fee),
                price: BigInt(order.Locked.price),
                base: {
                    ...order.Locked.base,
                    id: BigInt(order.Locked.base.id),
                    created_at: BigInt(order.Locked.base.created_at),
                    offramper_user_id: BigInt(order.Locked.base.offramper_user_id),
                    crypto: {
                        ...order.Locked.base.crypto,
                        fee: BigInt(order.Locked.base.crypto.fee),
                        amount: BigInt(order.Locked.base.crypto.amount),
                        blockchain: {
                            EVM: { chain_id: BigInt(order.Locked.base.crypto.blockchain.EVM.chain_id) }
                        }
                    }
                },
                onramper: {
                    ...order.Locked.onramper,
                    user_id: BigInt(order.Locked.onramper.user_id)
                }
            }
        };
    }
    return order as OrderState;
};
