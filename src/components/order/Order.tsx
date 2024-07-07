import React, { useState } from 'react';
import { ethers } from 'ethers';

import { Order, OrderState, PaymentProvider, PaymentProviderType } from '../../declarations/backend/backend.did';
import PayPalButton from '../PaypalButton';
import { NetworkIds, getTokenMapping } from '../../constants/addresses';
import { useUser } from '../../UserContext';
import { paymentProviderTypeToString } from '../../model/utils';
import { truncate } from '../../model/helper';

interface OrderProps {
    order: OrderState;
    commitToOrder: (orderId: bigint, provider: PaymentProvider) => void;
    removeOrder: (order: Order) => void;
    handlePayPalSuccess: (transactionId: string, orderId: bigint) => void;
}

const OrderActions: React.FC<OrderProps> = ({ order, commitToOrder, removeOrder, handlePayPalSuccess }) => {
    const [committedProvider, setCommittedProvider] = useState<[PaymentProviderType, String]>();
    const { user, userType } = useUser();

    const handleProviderSelection = (providerType: PaymentProviderType) => {
        if (!user) return;

        const onramperProvider = user.payment_providers.find(provider => {
            return paymentProviderTypeToString(provider.provider_type) === paymentProviderTypeToString(providerType);
        });
        console.log("onramperProvider = ", onramperProvider);
        if (!onramperProvider) return;

        const provider: [PaymentProviderType, String] = [providerType, onramperProvider.id];
        setCommittedProvider(provider);
    };

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

    const getTokenSymbol = (tokenAddress: string, chainId: number): string => {
        const tokenMapping = getTokenMapping(chainId);
        return tokenMapping[tokenAddress] || tokenAddress;
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
                    <div><strong>Providers:</strong>
                        {order.Created.offramper_providers.map(provider => (
                            <div key={paymentProviderTypeToString(provider[0])}>
                                <input
                                    type="checkbox"
                                    id={paymentProviderTypeToString(provider[0])}
                                    name={paymentProviderTypeToString(provider[0])}
                                    onChange={() => handleProviderSelection(provider[0])}
                                    checked={committedProvider && paymentProviderTypeToString(committedProvider[0]) === paymentProviderTypeToString(provider[0])}
                                />
                                <label htmlFor={paymentProviderTypeToString(provider[0])}>{paymentProviderTypeToString(provider[0])}</label>
                            </div>
                        ))}
                    </div>
                    <div><strong>Offramper Address:</strong> {truncate(order.Created.offramper_address, 6, 6)}</div>
                    <div><strong>Network:</strong> {getNetworkName(Number(order.Created.chain_id))}</div>
                    {userType === 'Onramper' && (
                        <div>
                            <button
                                onClick={() => commitToOrder(order.Created.id, {
                                    provider_type: committedProvider![0], id: committedProvider![1]
                                } as PaymentProvider)}
                                className="mt-2 px-4 py-2 bg-green-500 text-white rounded"
                                disabled={!committedProvider}
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
                    <div><strong>Offramper Provider:</strong>
                        {order.Locked.base.offramper_providers.map(provider => {
                            console.log("provider ", provider);
                            return (<div key={paymentProviderTypeToString(provider[0])}>{paymentProviderTypeToString(provider[0])}: {provider[1]}</div>);
                        })}
                    </div>
                    <div><strong>Onramper Provider:</strong> {paymentProviderTypeToString(order.Locked.onramper_provider.provider_type)}: {order.Locked.onramper_provider.id}</div>
                    <div><strong>Offramper Address:</strong> {truncate(order.Locked.base.offramper_address, 6, 6)}</div>
                    <div><strong>Network:</strong> {getNetworkName(Number(order.Locked.base.chain_id))}</div>
                    {userType === 'Onramper' && (
                        <div>
                            <PayPalButton
                                amount={order.Locked.base.fiat_amount / BigInt(100)}
                                clientId="Ab_E80t7BM4rNxj7trOAlRz_UmpEqPHANABmFUzD-7Zj-iiUI9nhkRilop_2lWKoWTE_bfEFiXV33mHb"
                                paypalId={order.Locked.base.offramper_providers.find(
                                    provider => paymentProviderTypeToString(provider[0]) === paymentProviderTypeToString(order.Locked.onramper_provider.provider_type)
                                )?.[1]!}
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
