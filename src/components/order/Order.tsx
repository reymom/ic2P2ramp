import React, { useState } from 'react';
import { ethers } from 'ethers';
import { useAccount } from 'wagmi';

import { Order, OrderState, PaymentProvider, PaymentProviderType } from '../../declarations/backend/backend.did';
import PayPalButton from '../PaypalButton';
import { NetworkIds, getTokenMapping } from '../../constants/tokens';
import { useUser } from '../../UserContext';
import { paymentProviderTypeToString } from '../../model/utils';
import { truncate } from '../../model/helper';
import { addresses } from '../../constants/addresses';
import { icP2PrampABI } from '../../constants/ic2P2ramp';
import { backend } from '../../declarations/backend';
import { rampErrorToString } from '../../model/error';

interface OrderProps {
    order: OrderState;
    refetchOrders: () => void;
}

const OrderActions: React.FC<OrderProps> = ({ order, refetchOrders }) => {
    const [committedProvider, setCommittedProvider] = useState<[PaymentProviderType, String]>();
    const [isLoading, setIsLoading] = useState(false);
    const [message, setMessage] = useState('');

    const { address, chainId } = useAccount();
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

    const commitToOrder = async (orderId: bigint, provider: PaymentProvider) => {
        setIsLoading(true);
        setMessage(`Commiting to loan order ${orderId}...`);

        try {
            const result = await backend.lock_order(orderId, provider, address!.toString(), [100000]);

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
        console.log("order = ", order);
        try {
            setIsLoading(true);
            setMessage(`Removing order ${order.id}...`);

            if (!window.ethereum) {
                throw new Error('No crypto wallet found. Please install it.');
            }

            const provider = new ethers.BrowserProvider(window.ethereum);
            await provider.send('eth_requestAccounts', []);
            const signer = await provider.getSigner();

            const tokenAddress = order.token_address[0] ?? addresses[Number(order.chain_id)].native[1];
            const vaultContract = new ethers.Contract(tokenAddress, icP2PrampABI, signer);

            const gasEstimate = await vaultContract.uncommitDeposit.estimateGas(order.offramper_address, ethers.ZeroAddress, order.crypto_amount);
            const transactionResponse = await vaultContract.uncommitDeposit(order.offramper_address, ethers.ZeroAddress, order.crypto_amount, {
                gasLimit: gasEstimate
            });

            setMessage('Transaction sent, waiting for confirmation...');
            const receipt = await transactionResponse.wait();
            console.log('Transaction receipt:', receipt);

            if (receipt.status === 1) {
                setMessage('Transaction successful!');
            } else {
                setMessage('ICP Transaction failed!');
                return;
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
                            return (<div key={paymentProviderTypeToString(provider[0])}>{paymentProviderTypeToString(provider[0])}: {provider[1]}</div>);
                        })}
                    </div>
                    <div><strong>Onramper Provider:</strong> {paymentProviderTypeToString(order.Locked.onramper_provider.provider_type)}: {order.Locked.onramper_provider.id}</div>
                    <div><strong>Offramper Address:</strong> {truncate(order.Locked.base.offramper_address, 6, 6)}</div>
                    <div><strong>Network:</strong> {getNetworkName(Number(order.Locked.base.chain_id))}</div>
                    {userType === 'Onramper' && (
                        <div>
                            <PayPalButton
                                orderId={order.Locked.base.id.toString()}
                                amount={Number(order.Locked.base.fiat_amount) / 100.}
                                currency={order.Locked.base.currency_symbol}
                                paypalId={order.Locked.base.offramper_providers.find(
                                    provider => paymentProviderTypeToString(provider[0]) === paymentProviderTypeToString(order.Locked.onramper_provider.provider_type)
                                )?.[1]!}
                                onSuccess={(transactionId) => handlePayPalSuccess(transactionId, order.Locked.base.id)}
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
