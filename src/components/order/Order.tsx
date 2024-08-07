import React, { useState } from 'react';
import { ethers } from 'ethers';
import { useAccount } from 'wagmi';

import { Blockchain, Order, OrderState, PaymentProvider, PaymentProviderType } from '../../declarations/backend/backend.did';
import PayPalButton from '../PaypalButton';
import { NetworkIds, getTokenMapping } from '../../constants/tokens';
import { useUser } from '../../UserContext';
import { blockchainToBlockchainType, paymentProviderTypeToString, providerToProviderType } from '../../model/utils';
import { truncate } from '../../model/helper';
import { backend } from '../../declarations/backend';
import { rampErrorToString } from '../../model/error';
import { withdrawFromVault } from '../../model/evm';

interface OrderProps {
    order: OrderState;
    refetchOrders: () => void;
}

const OrderActions: React.FC<OrderProps> = ({ order, refetchOrders }) => {
    const [committedProvider, setCommittedProvider] = useState<[PaymentProviderType, PaymentProvider]>();
    const [isLoading, setIsLoading] = useState(false);
    const [message, setMessage] = useState('');

    const { address, chainId } = useAccount();
    const { user, userType } = useUser();

    const handleProviderSelection = (selectedProviderType: PaymentProviderType) => {
        if (!user) return;

        const onramperProvider = user.payment_providers.find(userProvider => {
            return providerToProviderType(userProvider) === selectedProviderType;
        });
        console.log("onramperProvider = ", onramperProvider);
        if (!onramperProvider) return;

        const provider: [PaymentProviderType, PaymentProvider] = [providerToProviderType(onramperProvider), onramperProvider];
        setCommittedProvider(provider);
    };

    const commitToOrder = async (orderId: bigint, provider: PaymentProvider) => {
        setIsLoading(true);
        setMessage(`Commiting to loan order ${orderId}...`);

        try {
            const orderAddress = {
                address_type: { EVM: null },
                address: address as string,
            };
            const result = await backend.lock_order(orderId, provider, orderAddress, [100000]);

            if ('Ok' in result) {
                const explorerUrl = Object.values(NetworkIds).find(network => network.id === chainId)?.explorer;
                const txLink = `${explorerUrl}${result.Ok}`;
                setMessage(`Order Locked! tx = <a href="${txLink}" target="_blank">${truncate(result.Ok, 8, 8)}</a>`);
                setTimeout(() => {
                    refetchOrders();
                }, 3500);
            } else {
                const errorMessage = rampErrorToString(result.Err);
                setMessage(errorMessage);
            }
        } catch (err) {
            console.error(err);
            setMessage(`Error commiting to order ${orderId}.`);
        } finally {
            setIsLoading(false);
        }
    };

    const removeOrder = async (order: Order) => {
        try {
            setIsLoading(true);
            setMessage(`Removing order ${order.id}...`);

            switch (blockchainToBlockchainType(order.crypto.blockchain)) {
                case 'EVM':
                    try {
                        const receipt = await withdrawFromVault(order);
                        console.log('Transaction receipt: ', receipt);
                        setMessage('Transaction successful!');
                    } catch (e: any) {
                        setMessage(`Could not delete order: ${e.message || e}`);
                        return;
                    }
                    break;
                case 'ICP':
                    try {
                        // todo: implement in backend canister
                        setMessage('Transaction successful!');
                    } catch (e: any) {
                        setMessage(`Could not delete order: ${e.message || e}`);
                        return;
                    }
                    break;
                case 'Solana':
                    throw new Error("Solana orders are not implemented yet")
                default:
                    throw new Error('Blockchain not defined')
            }

            const result = await backend.cancel_order(order.id);
            if ('Ok' in result) {
                setMessage("Order Cancelled");
                refetchOrders();
            } else {
                const errorMessage = rampErrorToString(result.Err);
                setMessage(errorMessage);
            }
        } catch (err) {
            console.error(err);
            setMessage(`Error removing order ${order.id}.`);
        } finally {
            setIsLoading(false);
        }
    };

    const handlePayPalSuccess = async (transactionId: string, orderId: bigint) => {
        console.log("[handlePayPalSuccess] transactionID = ", transactionId);

        setIsLoading(true);
        setMessage(`Payment successful for order ${orderId}, transaction ID: ${transactionId}. Verifying...`);
        try {
            // Send transaction ID to backend to verify payment
            const response = await backend.verify_transaction(
                orderId,
                transactionId,
                [100000]
            );

            if ('Ok' in response) {
                setMessage(`Order Verified and Funds Transferred successfully!`);
            } else {
                const errorMessage = rampErrorToString(response.Err);
                setMessage(errorMessage);
            }
        } catch (err) {
            console.error(err);
            setMessage(`Error verifying payment for order ${orderId.toString()}.`);
        } finally {
            setIsLoading(false);
        }
    };

    const getNetworkName = (chainId: number): string => {
        return Object.values(NetworkIds).find(network => network.id === chainId)?.name!;
    }

    const getTokenSymbol = (blockchain: Blockchain, tokenAddress: string, chainId: number): string => {
        const tokenMapping = getTokenMapping(blockchainToBlockchainType(blockchain), chainId);
        return tokenMapping[tokenAddress] || tokenAddress;
    };

    const formatFiatAmount = (fiatAmount: bigint) => {
        return (Number(fiatAmount) / 100).toFixed(2);
    };

    const formatCryptoAmount = (amount: bigint, blockchain: Blockchain) => {
        switch (blockchainToBlockchainType(blockchain)) {
            case 'EVM':
                return ethers.formatEther(amount);
            case 'ICP':
                return (Number(amount) / 1_000_000).toFixed(2);
            case 'Solana':
                return "Solana not implemented"
        }
    }

    return (
        <li className="p-4 border rounded shadow-md bg-white">
            {'Created' in order && (
                <div className="flex flex-col space-y-2">
                    <div><strong>Fiat Amount:</strong> {formatFiatAmount(order.Created.fiat_amount)}</div>
                    <div>
                        <strong>Crypto Amount:</strong> {formatCryptoAmount(order.Created.crypto.amount, order.Created.crypto.blockchain)} {getTokenSymbol(order.Created.crypto.blockchain, order.Created.crypto.token?.[0] ?? '', Number(order.Created.crypto.blockchain))}
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
                    <div><strong>Offramper Address:</strong> {truncate(order.Created.offramper_address.address, 6, 6)} ({Object.keys(order.Created.offramper_address.address_type)[0]})</div>
                    <div><strong>Network:</strong> {getNetworkName(Number(order.Created.crypto.blockchain))}</div>
                    {userType === 'Onramper' && (
                        <div>
                            <button
                                onClick={() => commitToOrder(order.Created.id, committedProvider![1])}
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
                        <strong>Crypto Amount:</strong> {ethers.formatEther(order.Locked.base.crypto.amount.toString())} {getTokenSymbol(order.Locked.base.crypto.blockchain, order.Locked.base.crypto.token?.[0] ?? '', Number(order.Locked.base.crypto.blockchain))}
                    </div>
                    <div><strong>Offramper Provider:</strong>
                        {order.Locked.base.offramper_providers.map(provider => {
                            return (<div key={paymentProviderTypeToString(provider[0])}>{paymentProviderTypeToString(provider[0])}</div>);
                        })}
                    </div>
                    <div><strong>Onramper Provider:</strong> {paymentProviderTypeToString(providerToProviderType(order.Locked.onramper_provider))}</div>
                    <div><strong>Offramper Address:</strong> {truncate(order.Locked.base.offramper_address.address, 6, 6)} ({Object.keys(order.Locked.base.offramper_address.address_type)[0]})</div>
                    <div><strong>Network:</strong> {getNetworkName(Number(order.Locked.base.crypto.blockchain))}</div>
                    {userType === 'Onramper' && (
                        <div>
                            {order.Locked.onramper_provider.hasOwnProperty('PayPal') ? (
                                <PayPalButton
                                    orderId={order.Locked.base.id.toString()}
                                    amount={Number(order.Locked.base.fiat_amount) / 100.}
                                    currency={order.Locked.base.currency_symbol}
                                    paypalId={(() => {
                                        const provider = order.Locked.base.offramper_providers.find(
                                            provider => 'PayPal' in provider[1]
                                        );
                                        if (provider && 'PayPal' in provider[1]) {
                                            return provider[1].PayPal.id;
                                        }
                                        return '';
                                    })()}
                                    onSuccess={(transactionId) => handlePayPalSuccess(transactionId, order.Locked.base.id)}
                                />
                            ) : order.Locked.onramper_provider.hasOwnProperty('Revolut') ? (
                                <div>
                                    <button
                                        className="px-4 py-2 bg-blue-500 text-white rounded"
                                        onClick={() => {
                                            // Placeholder for redirecting to Revolut consent URL
                                            const consentUrl = order.Locked.consent_url?.[0];
                                            if (consentUrl) {
                                                window.location.href = consentUrl;
                                            }
                                            // Placeholder for backend calls to listen for the transaction
                                            console.log('Listening for Revolut transaction confirmation...');
                                        }}
                                    >
                                        Confirm Revolut Consent
                                    </button>
                                </div>
                            ) : null}

                        </div>
                    )}
                </div>
            )}
            {'Completed' in order && (
                <div className="flex flex-col space-y-2">
                    <div><strong>Fiat Amount:</strong> {formatFiatAmount(order.Completed.fiat_amount)}</div>
                    <div><strong>Onramper:</strong> {truncate(order.Completed.onramper.address, 6, 6)}</div>
                    <div><strong>Offramper:</strong> {truncate(order.Completed.offramper.address, 6, 6)}</div>
                    {/* <div><strong>Network:</strong> {getNetworkName(Number(order.Completed.chain_id))}</div> */}
                </div>
            )}
            {'Cancelled' in order && (
                <div className="flex flex-col space-y-2">
                    <div><strong>Order ID:</strong> {order.Cancelled.toString()}</div>
                    <div><strong>Status:</strong> Cancelled</div>
                </div>
            )}
            {isLoading ? (
                <div className="mt-4 flex justify-center items-center space-x-2">
                    <div className="w-4 h-4 border-t-2 border-b-2 border-indigo-600 rounded-full animate-spin"></div>
                    <div className="text-sm font-medium text-gray-700">Processing transaction...</div>
                </div>
            ) : (
                message && <div className="message-container" dangerouslySetInnerHTML={{ __html: message }}></div>
            )}
        </li>
    );
}

export default OrderActions;
