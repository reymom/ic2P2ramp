import React, { useState } from 'react';
import { ethers } from 'ethers';

import { backend } from '../../declarations/backend';
import { OrderState, PaymentProvider, PaymentProviderType } from '../../declarations/backend/backend.did';
import { NetworkIds, getTokenMapping } from '../../constants/tokens';
import { blockchainToBlockchainType, paymentProviderTypeToString, providerToProviderType } from '../../model/utils';
import { truncate } from '../../model/helper';
import { rampErrorToString } from '../../model/error';
import { withdrawFromVault } from '../../model/evm';
import { PaymentProviderTypes } from '../../model/types';
import PayPalButton from '../PaypalButton';
import { useUser } from '../../UserContext';

interface OrderProps {
    order: OrderState;
    refetchOrders: () => void;
}

const OrderActions: React.FC<OrderProps> = ({ order, refetchOrders }) => {
    const [committedProvider, setCommittedProvider] = useState<[PaymentProviderType, PaymentProvider]>();
    const [isLoading, setIsLoading] = useState(false);
    const [message, setMessage] = useState('');

    const { user, userType } = useUser();

    const orderId = 'Created' in order ? order.Created.id
        : 'Locked' in order ? order.Locked.base.id
            : null;

    const orderBlockchain = 'Created' in order ? order.Created.crypto.blockchain
        : 'Locked' in order ? order.Locked.base.crypto.blockchain
            : 'Completed' in order ? order.Completed.blockchain
                : null;


    const orderFiatAmount = 'Created' in order ? order.Created.fiat_amount + order.Created.offramper_fee
        : 'Locked' in order ? order.Locked.base.fiat_amount + order.Locked.base.offramper_fee
            : 'Completed' in order ? order.Completed.fiat_amount + order.Completed.offramper_fee
                : null;

    const handleProviderSelection = (selectedProviderType: PaymentProviderTypes) => {
        if (!user) return;

        const onramperProvider = user.payment_providers.find(userProvider => {
            return paymentProviderTypeToString(providerToProviderType(userProvider)) === selectedProviderType;
        });
        if (!onramperProvider) return;

        const provider: [PaymentProviderType, PaymentProvider] = [providerToProviderType(onramperProvider), onramperProvider];
        setCommittedProvider(provider);
    };

    const commitToOrder = async (provider: PaymentProvider) => {
        if (!user || !('Onramper' in user.user_type) || !('Created' in order) || !(orderBlockchain) || !orderId) return;

        setIsLoading(true);
        setMessage(`Commiting to loan order ${orderId}...`);

        try {
            const orderAddress = user.addresses.find(address => {
                if ('EVM' in orderBlockchain && 'EVM' in address.address_type) {
                    return true;
                }
                if ('ICP' in orderBlockchain && 'ICP' in address.address_type) {
                    return true;
                }
                if ('Solana' in orderBlockchain && 'Solana' in address.address_type) {
                    return true;
                }
                return false;
            }) || null;

            if (!orderAddress) throw new Error("No address matches for user");

            const result = await backend.lock_order(orderId, user.id, provider, orderAddress, [100000]);

            if ('Ok' in result) {
                if ('EVM' in orderBlockchain) {
                    const explorerUrl = Object.values(NetworkIds).find(network => network.id === orderBlockchain.EVM.chain_id)?.explorer;
                    const txLink = `${explorerUrl}${result.Ok}`;
                    setMessage(`Order Locked! tx = <a href="${txLink}" target="_blank">${truncate(result.Ok, 8, 8)}</a>`);
                } else {
                    setMessage(`Order Locked!`)
                }
                setTimeout(() => {
                    refetchOrders();
                }, 1000);
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

    const removeOrder = async () => {
        if (!('Created' in order) || !orderBlockchain || !orderId) return;

        try {
            setIsLoading(true);
            setMessage(`Removing order ${orderId}...`);

            switch (blockchainToBlockchainType(orderBlockchain)) {
                case 'EVM':
                    try {
                        const receipt = await withdrawFromVault(order.Created);
                        console.log('Transaction receipt: ', receipt);
                        setMessage('Transaction successful!');
                    } catch (e: any) {
                        setMessage(`Could not delete order: ${e.message || e}`);
                        return;
                    }
                    break;
                case 'ICP':
                    // funds are transfered to the offramper from the backend
                    break;
                case 'Solana':
                    throw new Error("Solana orders are not implemented yet")
                default:
                    throw new Error('Blockchain not defined')
            }

            const result = await backend.cancel_order(orderId);
            if ('Ok' in result) {
                setMessage("Order Cancelled");
                refetchOrders();
            } else {
                const errorMessage = rampErrorToString(result.Err);
                setMessage(errorMessage);
            }
        } catch (err) {
            console.error(err);
            setMessage(`Error removing order ${orderId}.`);
        } finally {
            setIsLoading(false);
        }
    };

    const handlePayPalSuccess = async (transactionId: string) => {
        if (!('Locked' in order) || !orderId) return;

        console.log("[handlePayPalSuccess] transactionID = ", transactionId);

        setIsLoading(true);
        setMessage(`Payment successful for order ${orderId}, transaction ID: ${transactionId}. Verifying...`);
        try {
            // Send transaction ID to backend to verify payment
            const response = await backend.verify_transaction(orderId, transactionId, [100000]);
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
            setTimeout(() => {
                refetchOrders();
            }, 1000);
        }
    };

    const handleRevolutRedirect = async () => {
        if (!('Locked' in order) || !orderId) return;

        const consentUrl = order.Locked.consent_url?.[0];
        if (consentUrl) {
            console.log('Listening for Revolut transaction confirmation...');
            backend.execute_revolut_payment(orderId)
                .catch(err => console.error("Failed to execute revolut payment: ", err));
            window.location.href = consentUrl;
        } else {
            console.error('Consent URL is not available.');
        }
    }

    const getNetworkName = (): string => {
        if (!orderBlockchain) return "";
        const chainId = 'EVM' in orderBlockchain ? orderBlockchain.EVM.chain_id : null;
        return Object.values(NetworkIds).find(network => network.id === chainId)?.name!;
    }

    const getTokenSymbol = (): string => {
        const crypto = 'Created' in order && 'EVM' in order.Created.crypto ? order.Created.crypto
            : 'Locked' in order && 'EVM' in order.Locked.base.crypto ? order.Locked.base.crypto
                : null
        if (!crypto) return "";

        const chain = 'EVM' in crypto.blockchain ? Number(crypto.blockchain.EVM.chain_id) : undefined;
        const tokenAddress = 'EVM' in crypto.blockchain ? crypto.token?.[0] ?? ''
            : 'ICP' in crypto.blockchain ? crypto.blockchain.ICP.ledger_principal.toString() : '';

        const tokenMapping = getTokenMapping(blockchainToBlockchainType(crypto.blockchain), Number(chain));
        return tokenMapping[tokenAddress] || tokenAddress;
    };

    const formatFiatAmount = () => {
        return (Number(orderFiatAmount) / 100).toFixed(2);
    };

    const formatCryptoAmount = () => {
        const crypto = 'Created' in order ? order.Created.crypto
            : 'Locked' in order ? order.Locked.base.crypto
                : null
        if (!crypto) return;

        switch (blockchainToBlockchainType(crypto.blockchain)) {
            case 'EVM':
                return ethers.formatEther(crypto.amount);
            case 'ICP':
                return (Number(crypto.amount) / 100_000_000).toFixed(2);
            case 'Solana':
                return "Solana not implemented"
        }
    }

    return (
        <li className="p-4 border rounded shadow-md bg-white">
            {'Created' in order && (
                <div className="flex flex-col space-y-2">
                    <div><strong>Fiat Amount:</strong> {formatFiatAmount()} $</div>
                    <div>
                        <strong>Crypto Amount:</strong> {formatCryptoAmount()} {getTokenSymbol()}
                    </div>
                    <div><strong>Providers:</strong>
                        {order.Created.offramper_providers.map((provider, index) => {
                            let providerType = paymentProviderTypeToString(provider[0])
                            return (
                                <div key={index}>
                                    <input
                                        type="checkbox"
                                        id={`provider-${index}`}
                                        onChange={() => handleProviderSelection(providerType)}
                                        checked={committedProvider && paymentProviderTypeToString(committedProvider[0]) === providerType}
                                    />
                                    <label htmlFor={providerType}>{providerType}</label>
                                </div>
                            )
                        })}
                    </div>
                    <div><strong>Offramper Address:</strong> {truncate(order.Created.offramper_address.address, 6, 6)} ({Object.keys(order.Created.offramper_address.address_type)[0]})</div>
                    {'EVM' in order.Created.crypto.blockchain && (
                        <div><strong>Network:</strong> {getNetworkName()}</div>
                    )}
                    {userType === 'Onramper' && (
                        <div>
                            <button
                                onClick={() => commitToOrder(committedProvider![1])}
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
                                onClick={removeOrder}
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
                    <div><strong>Fiat Amount:</strong> {formatFiatAmount()} $</div>
                    <div>
                        <strong>Crypto Amount:</strong> {formatCryptoAmount()} {getTokenSymbol()}
                    </div>
                    <div><strong>Provider:</strong> {paymentProviderTypeToString(providerToProviderType(order.Locked.onramper_provider))}</div>
                    <div><strong>Offramper Address:</strong> {truncate(order.Locked.base.offramper_address.address, 6, 6)} ({Object.keys(order.Locked.base.offramper_address.address_type)[0]})</div>
                    {'EVM' in order.Locked.base.crypto.blockchain && (
                        <div><strong>Network:</strong> {getNetworkName()}</div>
                    )}
                    {userType === 'Onramper' && (
                        <div>
                            {order.Locked.onramper_provider.hasOwnProperty('PayPal') ? (
                                <PayPalButton
                                    orderId={order.Locked.base.id.toString()}
                                    amount={Number(order.Locked.base.fiat_amount + order.Locked.base.offramper_fee) / 100.}
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
                                    onSuccess={(transactionId) => handlePayPalSuccess(transactionId)}
                                />
                            ) : order.Locked.onramper_provider.hasOwnProperty('Revolut') ? (
                                <div>
                                    <button
                                        className="px-4 py-2 bg-blue-500 text-white rounded"
                                        onClick={handleRevolutRedirect}
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
                    <div><strong>Fiat Amount:</strong> {formatFiatAmount()} $</div>
                    <div><strong>Onramper:</strong> {truncate(order.Completed.onramper.address, 6, 6)}</div>
                    <div><strong>Offramper:</strong> {truncate(order.Completed.offramper.address, 6, 6)}</div>
                    {'EVM' in order.Completed.blockchain && (
                        <div><strong>Network:</strong> {getNetworkName()}</div>
                    )}
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
