import React from 'react';
import { ethers } from 'ethers';

import { Order, OrderState, PaymentProvider } from '../../declarations/backend/backend.did';
import PayPalButton from '../PaypalButton';
import { UserTypes } from '../../model/types';
import {
    SepoliaTokens,
    BaseSepoliaTokens,
    PolygonZkEvmTokens,
    OptimismSepoliaTokens,
    NetworkIds
} from '../../tokens';

interface OrderProps {
    order: OrderState;
    userType: UserTypes;
    address: string;
    chainId: number;
    commitToOrder: (orderId: bigint, provider: PaymentProvider) => void;
    removeOrder: (order: Order) => void;
    handlePayPalSuccess: (transactionId: string, orderId: bigint) => void;
}

const OrderActions: React.FC<OrderProps> = ({ order, userType, commitToOrder, removeOrder, handlePayPalSuccess }) => {
    const getNetworkName = (chainId: number) => {
        switch (chainId) {
            case NetworkIds.SEPOLIA:
                return 'Sepolia';
            case NetworkIds.BASE_SEPOLIA:
                return 'Base Sepolia';
            case NetworkIds.OP_SEPOLIA:
                return 'Optimism Sepolia';
            case NetworkIds.POLYGON_ZKEVM_TESTNET:
                return 'Polygon zkEVM Testnet';
            default:
                return 'Unknown Network';
        }
    };

    const getTokenSymbol = (tokenType: string, chainId: number): string => {
        switch (chainId) {
            case NetworkIds.SEPOLIA:
                return SepoliaTokens[tokenType as keyof typeof SepoliaTokens] || tokenType;
            case NetworkIds.BASE_SEPOLIA:
                return BaseSepoliaTokens[tokenType as keyof typeof BaseSepoliaTokens] || tokenType;
            case NetworkIds.POLYGON_ZKEVM_TESTNET:
                return PolygonZkEvmTokens[tokenType as keyof typeof PolygonZkEvmTokens] || tokenType;
            case NetworkIds.OP_SEPOLIA:
                return OptimismSepoliaTokens[tokenType as keyof typeof OptimismSepoliaTokens] || tokenType;
            default:
                return tokenType;
        }
    };

    const truncate = (str: string, frontChars: number, backChars: number) => {
        if (str.length <= frontChars + backChars) {
            return str;
        }
        return str.slice(0, frontChars) + '...' + str.slice(-backChars);
    };

    const formatFiatAmount = (fiatAmount: bigint) => {
        return (Number(fiatAmount) / 100).toFixed(2);
    };

    return (
        <li className="p-4 border rounded shadow-md bg-white">
            {'Created' in order && (
                <div className="flex flex-col space-y-2">
                    <div><strong>Fiat Amount:</strong> {formatFiatAmount(order.Created.fiat_amount)}</div>
                    <div>
                        <strong>Crypto Amount:</strong> {ethers.formatEther(order.Created.crypto_amount.toString())} {getTokenSymbol(order.Created.token_address?.[0] ?? '', Number(order.Created.chain_id))}
                    </div>
                    <div><strong>PayPal ID:</strong> {truncate(order.Created.offramper_address, 6, 6)}</div>
                    <div><strong>Offramper Address:</strong> {truncate(order.Created.offramper_address, 6, 6)}</div>
                    <div><strong>Network:</strong> {getNetworkName(Number(order.Created.chain_id))}</div>
                    <div><strong>Token:</strong> {order.Created.token_address?.[0] ?? ''}</div>
                    {userType === 'Onramper' && (
                        <div>
                            <button
                                onClick={() => commitToOrder(order.Created.id, order.Created.offramper_providers.values().next().value)}
                                className="mt-2 px-4 py-2 bg-green-500 text-white rounded"
                            >
                                Commit
                            </button>
                        </div>
                    )}
                    {userType === 'Offramper' && (
                        <div>
                            <button
                                onClick={() => removeOrder(order.Created)}
                                className="mt-2 px-4 py-2 bg-red-500 text-white rounded"
                            >
                                Remove
                            </button>
                        </div>
                    )}
                </div>
            )}
            {'Locked' in order && (
                <div className="flex flex-col space-y-2">
                    <div><strong>Fiat Amount:</strong> {formatFiatAmount(order.Locked.base.fiat_amount)}</div>
                    <div>
                        <strong>Crypto Amount:</strong> {ethers.formatEther(order.Locked.base.crypto_amount.toString())} {getTokenSymbol(order.Locked.base.token_address?.[0] ?? '', Number(order.Locked.base.chain_id))}
                    </div>
                    <div><strong>PayPal ID:</strong> {truncate(order.Locked.base.offramper_address, 6, 6)}</div>
                    <div><strong>Offramper Address:</strong> {truncate(order.Locked.base.offramper_address, 6, 6)}</div>
                    <div><strong>Network:</strong> {getNetworkName(Number(order.Locked.base.chain_id))}</div>
                    <div><strong>Token:</strong> {order.Locked.base.token_address?.[0] ?? ''}</div>
                    {userType === 'Onramper' && (
                        <div>
                            <PayPalButton
                                amount={order.Locked.base.fiat_amount}
                                paypalId={order.Locked.base.offramper_address}
                                onSuccess={(transactionId) => handlePayPalSuccess(transactionId, order.Locked.base.id)}
                                currency="USD"
                            />
                        </div>
                    )}
                </div>
            )}
            {'Completed' in order && (
                <div className="flex flex-col space-y-2">
                    <div><strong>Fiat Amount:</strong> {formatFiatAmount(order.Completed.fiat_amount)}</div>
                    <div><strong>Onramper:</strong> {truncate(order.Completed.onramper, 6, 6)}</div>
                    <div><strong>Offramper:</strong> {truncate(order.Completed.offramper, 6, 6)}</div>
                    <div><strong>Network:</strong> {getNetworkName(Number(order.Completed.chain_id))}</div>
                </div>
            )}
            {'Cancelled' in order && (
                <div className="flex flex-col space-y-2">
                    <div><strong>Order ID:</strong> {order.Cancelled.toString()}</div>
                    <div><strong>Status:</strong> Cancelled</div>
                </div>
            )}
        </li>
    );
}

export default OrderActions;
