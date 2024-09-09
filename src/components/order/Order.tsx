import React, { useState } from 'react';
import { ethers } from 'ethers';
import { useAccount } from 'wagmi';

import icpLogo from "../../assets/icp-logo.svg";
import ethereumLogo from "../../assets/ethereum-logo.png";

import { backend } from '../../declarations/backend';
import { OrderState, PaymentProvider, PaymentProviderType } from '../../declarations/backend/backend.did';
import { NetworkIds } from '../../constants/networks';
import { commitEvmGas, getEvmTokenOptions, getIcpTokenOptions, releaseEvmGas, TokenOption } from '../../constants/tokens';
import { blockchainToBlockchainType, paymentProviderTypeToString, providerToProviderType } from '../../model/utils';
import { truncate } from '../../model/helper';
import { rampErrorToString } from '../../model/error';
import { estimateOrderLockGas, estimateOrderReleaseGas, withdrawFromVault } from '../../model/evm';
import { PaymentProviderTypes } from '../../model/types';
import PayPalButton from '../PaypalButton';
import { useUser } from '../user/UserContext';

interface OrderProps {
    order: OrderState;
    refetchOrders: () => void;
}

const Order: React.FC<OrderProps> = ({ order, refetchOrders }) => {
    const [committedProvider, setCommittedProvider] = useState<[PaymentProviderType, PaymentProvider]>();
    const [isLoading, setIsLoading] = useState(false);
    const [message, setMessage] = useState('');
    const [txHash, setTxHash] = useState<string>();

    const { chainId } = useAccount();
    const { user, userType, sessionToken, fetchIcpBalance } = useUser();

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

    const explorerUrl = Object.values(NetworkIds).find(network => network.id === chainId)?.explorer;

    const handleProviderSelection = (selectedProviderType: PaymentProviderTypes) => {
        if (!user) return;

        const onramperProvider = user.payment_providers.find(userProvider => {
            return paymentProviderTypeToString(providerToProviderType(userProvider)) === selectedProviderType;
        });
        if (!onramperProvider) return;

        if (committedProvider && paymentProviderTypeToString(committedProvider[0]) === selectedProviderType) {
            setCommittedProvider(undefined);
        } else {
            const provider: [PaymentProviderType, PaymentProvider] = [providerToProviderType(onramperProvider), onramperProvider];
            setCommittedProvider(provider);
        }
    };

    const commitToOrder = async (provider: PaymentProvider) => {
        if (!sessionToken) throw new Error("Please authenticate to get a token session")

        if (!user || !('Onramper' in user.user_type) || !('Created' in order) || !(orderBlockchain) || !orderId) return;

        setIsLoading(true);
        setTxHash(undefined);
        setMessage(`Commiting to loan order ${orderId}...`);

        let gasEstimation: [] | [number] = [];
        const hasToken = order.Created.crypto.token.length > 0;
        const tokenOption: TokenOption = {
            name: "",
            address: hasToken ? order.Created.crypto.token[0]! : "",
            isNative: hasToken ? true : false,
            rateSymbol: "",
        }
        try {
            const orderAddress = user.addresses.find(address => {
                if ('EVM' in orderBlockchain && 'EVM' in address.address_type) {
                    // gasEstimation = [Number(estimateOrderLockGas(
                    //     Number(orderBlockchain.EVM.chain_id),
                    //     tokenOption,
                    //     order.Created.crypto.amount - order.Created.crypto.fee
                    // ))]
                    gasEstimation = [commitEvmGas];
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

            const result = await backend.lock_order(orderId, sessionToken, user.id, provider, orderAddress, gasEstimation);

            if ('Ok' in result) {
                if ('EVM' in orderBlockchain) {
                    setTxHash(result.Ok);
                    const provider = new ethers.BrowserProvider(window.ethereum);
                    provider.once(result.Ok, (transactionReceipt) => {
                        if (transactionReceipt.status === 1) {
                            setMessage("Order Locked!");
                            setTxHash(undefined);
                            setIsLoading(false);
                            setTimeout(() => {
                                refetchOrders();
                            }, 4500);
                        } else {
                            setMessage("Transaction failed!");
                            setTxHash(undefined);
                        }
                    });
                } else {
                    setMessage("Order Locked!");
                    setIsLoading(false);
                    setTimeout(() => {
                        refetchOrders();
                    }, 2500);
                }
            } else {
                const errorMessage = rampErrorToString(result.Err);
                setMessage(errorMessage);
                setIsLoading(false);
            }
        } catch (err) {
            setIsLoading(false);
            console.error(err);
            setMessage(`Error commiting to order ${orderId}.`);
        }
    };

    const removeOrder = async () => {
        if (!sessionToken) throw new Error("Please authenticate to get a token session")
        if (!('Created' in order) || !orderBlockchain || !orderId) return;

        try {
            setIsLoading(true);
            setTxHash(undefined);
            setMessage(`Removing order ${orderId}...`);

            switch (blockchainToBlockchainType(orderBlockchain)) {
                case 'EVM':
                    const orderChainId = 'EVM' in order.Created.crypto.blockchain ? order.Created.crypto.blockchain.EVM.chain_id : undefined;
                    if (!chainId || chainId !== Number(orderChainId)) throw new Error('Connect to same network than the order crypto');
                    try {
                        const receipt = await withdrawFromVault(chainId, order.Created);
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

            const result = await backend.cancel_order(orderId, sessionToken);
            if ('Ok' in result) {
                setMessage("Order Cancelled");
                refetchOrders();
                fetchIcpBalance();
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
        if (!sessionToken) throw new Error("Please authenticate to get a token session")
        if (!('Locked' in order) || !orderId || !orderBlockchain) return;

        console.log("[handlePayPalSuccess] transactionID = ", transactionId);

        setIsLoading(true);
        setTxHash(undefined);
        setMessage(`Payment successful for order ${orderId}, transaction ID: ${transactionId}. Verifying...`);

        // estimate withdraw gas
        let gasEstimation: [] | [number] = [];
        const hasToken = order.Locked.base.crypto.token.length > 0;
        const tokenOption: TokenOption = {
            name: "",
            address: hasToken ? order.Locked.base.crypto.token[0]! : "",
            isNative: hasToken ? true : false,
            rateSymbol: "",
        }
        if ('EVM' in orderBlockchain) {
            // gasEstimation = [Number(estimateOrderReleaseGas(
            //     Number(orderBlockchain.EVM.chain_id),
            //     tokenOption,
            //     order.Locked.base.crypto.amount - order.Locked.base.crypto.fee
            // ))]
            gasEstimation = [releaseEvmGas];
        }

        try {
            // Send transaction ID to backend to verify payment
            const response = await backend.verify_transaction(orderId, sessionToken, transactionId, gasEstimation);
            if ('Ok' in response) {
                setMessage(`Order Verified and Funds Transferred successfully!`);
                if ('EVM' in orderBlockchain!) {
                    setTxHash(response.Ok);

                    const provider = new ethers.BrowserProvider(window.ethereum);
                    provider.once(response.Ok, (transactionReceipt) => {
                        if (transactionReceipt.status === 1) {
                            setMessage("Order Completed!");
                            setIsLoading(false);
                            setTxHash(undefined);
                            setTimeout(() => {
                                refetchOrders();
                            }, 4500);
                        } else {
                            setMessage("Transaction failed!");
                            setTxHash(undefined);
                        }
                    });
                } else {
                    setMessage("Order Locked!");
                    setIsLoading(false);
                    setTimeout(() => {
                        refetchOrders();
                        fetchIcpBalance();
                    }, 2000);
                }
            } else {
                setIsLoading(false);
                const errorMessage = rampErrorToString(response.Err);
                setMessage(errorMessage);
            }
        } catch (err) {
            setIsLoading(false);
            console.error(err);
            setMessage(`Error verifying payment for order ${orderId.toString()}.`);
        }
    };

    const handleRevolutRedirect = async () => {
        if (!sessionToken) throw new Error("Please authenticate to get a token session")
        if (!('Locked' in order) || !orderId) return;

        const consentUrl = order.Locked.consent_url?.[0];
        if (consentUrl) {
            console.log('Listening for Revolut transaction confirmation...');
            backend.execute_revolut_payment(orderId, sessionToken)
                .catch(err => console.error("Failed to execute revolut payment: ", err));
            window.location.href = consentUrl;
        } else {
            console.error('Consent URL is not available.');
        }
    }

    const getNetworkName = (): string => {
        if (!orderBlockchain) return "";
        const chainId = 'EVM' in orderBlockchain ? orderBlockchain.EVM.chain_id : null;
        return Object.values(NetworkIds).find(network => network.id === Number(chainId))?.name!;
    }

    const getTokenSymbol = (): string => {
        const crypto = 'Created' in order ? order.Created.crypto
            : 'Locked' in order ? order.Locked.base.crypto
                : null
        if (!crypto) return "";

        const tokens = 'EVM' in crypto.blockchain ? getEvmTokenOptions(Number(crypto.blockchain.EVM.chain_id))
            : 'ICP' in crypto.blockchain ? getIcpTokenOptions() : null;
        if (!tokens) return "";

        const tokenAddress = 'EVM' in crypto.blockchain ? crypto.token?.[0] ?? '' :
            'ICP' in crypto.blockchain ? crypto.blockchain.ICP.ledger_principal.toString() : '';

        const token = tokens.find(token => {
            return token.address === tokenAddress;
        });
        return token ? token.rateSymbol : "Unknown";
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
                const fullAmountEVM = ethers.formatEther(crypto.amount - crypto.fee);
                const shortAmountEVM = parseFloat(fullAmountEVM).toPrecision(3);
                return { fullAmount: fullAmountEVM, shortAmount: shortAmountEVM };
            case 'ICP':
                const fullAmountICP = (Number(crypto.amount - crypto.fee) / 100_000_000).toString();
                const shortAmountICP = parseFloat(fullAmountICP).toPrecision(3);
                return { fullAmount: fullAmountICP, shortAmount: shortAmountICP };
            case 'Solana':
                return { fullAmount: "Solana not implemented", shortAmount: "Solana not implemented" };
        }
    }

    let blockchainLogo = !orderBlockchain ? null :
        'EVM' in orderBlockchain ? ethereumLogo : 'ICP' in orderBlockchain ? icpLogo : null;
    let backgroundColor =
        'Created' in order ? "bg-blue-900 bg-opacity-30"
            : 'Locked' in order ? "bg-yellow-800 bg-opacity-30"
                : 'Completed' in order ? "bg-green-800 bg-opacity-30"
                    : 'Cancelled' in order ? "bg-red-800 bg-opacity-30"
                        : "bg-gray-800 bg-opacity-20";

    let borderColor =
        'Created' in order ? "border-blue-600"
            : 'Locked' in order ? "border-yellow-600"
                : 'Completed' in order ? "border-green-600"
                    : 'Cancelled' in order ? "border-red-600"
                        : "border-gray-600";

    let textColor = 'Created' in order || 'Locked' in order ? "text-white" : "text-gray-300";

    return (
        <li className={`px-12 py-8 border rounded-xl shadow-md ${backgroundColor} ${borderColor} ${textColor} relative`}>
            {blockchainLogo && (
                <img
                    src={blockchainLogo}
                    alt="Blockchain Logo"
                    className="absolute top-2 left-2 h-6 w-6 opacity-90"
                />
            )}
            {'Created' in order && (
                <div className="flex flex-col">
                    <div className="space-y-3">
                        {/* Fiat and Crypto Amount */}
                        <div className="text-lg flex justify-between">
                            <span className="opacity-90">Price:</span>
                            <span className="font-medium">{formatFiatAmount()} $</span>
                        </div>
                        <div className="text-lg flex justify-between">
                            <span className="opacity-90">Amount:</span>
                            <span className="font-medium" title={formatCryptoAmount()?.fullAmount}>
                                {formatCryptoAmount()?.shortAmount} {getTokenSymbol()}
                            </span>
                        </div>

                        {/* Offramper Address */}
                        <div className="text-lg flex justify-between">
                            <span className="opacity-90">Address:</span>
                            <span className="font-medium">{truncate(order.Created.offramper_address.address, 6, 6)}</span>
                        </div>

                        {'EVM' in order.Created.crypto.blockchain && (
                            <div className="text-lg flex justify-between">
                                <span className="opacity-80">Network:</span>
                                <span className="font-medium">{getNetworkName()}</span>
                            </div>
                        )}
                    </div>

                    <hr className="border-t border-gray-500 w-full my-3" />

                    {/* Providers */}
                    <div className="text-lg">
                        <span className="opacity-90">Payment Methods:</span>
                        <div className="font-medium">
                            {order.Created.offramper_providers.map((provider, index) => {
                                let providerType = paymentProviderTypeToString(provider[0]);

                                if (userType === 'Onramper') {
                                    return (
                                        <div key={index} className="my-2">
                                            <input
                                                type="checkbox"
                                                id={`provider-${index}`}
                                                onChange={() => handleProviderSelection(providerType)}
                                                checked={committedProvider && paymentProviderTypeToString(committedProvider[0]) === providerType}
                                                className="form-checkbox h-5 w-5 text-center"
                                            />
                                            <label htmlFor={`provider-${index}`} className="ml-3 text-lg">{providerType}</label>
                                        </div>
                                    );
                                } else {
                                    return (
                                        <div key={index} className="text-lg my-2">{providerType}</div>
                                    );
                                }
                            })}
                        </div>
                    </div>

                    <hr className="border-t border-gray-500 w-full my-3" />

                    {/* Commit Button for Onramper */}
                    {user && userType === 'Onramper' && (() => {
                        const disabled = !committedProvider ||
                            !user.addresses.some(addr =>
                                Object.keys(addr.address_type)[0] === Object.keys(order.Created.offramper_address.address_type)[0]
                            );
                        return (
                            <button
                                onClick={() => commitToOrder(committedProvider![1])}
                                className={`mt-3 px-4 py-2 rounded w-full font-medium ${disabled ? 'bg-gray-500 cursor-not-allowed' : 'bg-green-600 hover:bg-green-700'}`}
                                disabled={disabled}
                            >
                                Lock Order (1h to pay)
                            </button>
                        );
                    })()}

                    {/* Remove Button for Offramper */}
                    {user && userType === 'Offramper' && order.Created.offramper_user_id === user.id && (
                        <button
                            onClick={removeOrder}
                            className="mt-3 px-4 py-2 bg-red-600 rounded w-full font-medium hover:bg-red-700"
                        >
                            Remove
                        </button>
                    )}
                </div>
            )}
            {'Locked' in order && (
                <div className="flex flex-col">
                    <div className="space-y-3">
                        {/* Fiat and Crypto Amount */}
                        <div className="text-lg flex justify-between">
                            <span className="opacity-90">Price:</span>
                            <span className="font-medium">{formatFiatAmount()} $</span>
                        </div>
                        <div className="text-lg flex justify-between">
                            <span className="opacity-90">Amount:</span>
                            <span className="font-medium" title={formatCryptoAmount()?.fullAmount}>
                                {formatCryptoAmount()?.shortAmount} {getTokenSymbol()}
                            </span>
                        </div>

                        {/* Offramper Address */}
                        <div className="text-lg flex justify-between">
                            <span className="opacity-90">Address:</span>
                            <span className="font-medium">{truncate(order.Locked.base.offramper_address.address, 6, 6)}</span>
                        </div>

                        {'EVM' in order.Locked.base.crypto.blockchain && (
                            <div className="text-lg flex justify-between">
                                <span className="opacity-80">Network:</span>
                                <span className="font-medium">{getNetworkName()}</span>
                            </div>
                        )}
                    </div>

                    {user && userType === 'Onramper' && order.Locked.onramper_user_id === user.id && (
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
                                        className="px-4 py-2 bg-blue-600 rounded-md hover:bg-blue-700"
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
                <div className="flex flex-col space-y-3">
                    <div className="text-lg flex justify-between">
                        <span className="opacity-90">Fiat Amount:</span>
                        <span className="font-medium">{formatFiatAmount()} $</span>
                    </div>
                    <div className="text-lg flex justify-between">
                        <span className="opacity-90">Onramper:</span>
                        <span className="font-medium">{truncate(order.Completed.onramper.address, 6, 6)}</span>
                    </div>
                    <div>
                        <span className="opacity-90">Offramper:</span>
                        <span className="font-medium">{truncate(order.Completed.offramper.address, 6, 6)}</span>
                    </div>
                    {'EVM' in order.Completed.blockchain && (
                        <div className="text-lg flex justify-between">
                            <span className="opacity-80">Network:</span>
                            <span className="font-medium">{getNetworkName()}</span>
                        </div>
                    )}
                </div>
            )}
            {'Cancelled' in order && (
                <div className="flex flex-col space-y-3">
                    <div><strong>Order ID:</strong> {order.Cancelled.toString()}</div>
                    <div><strong>Status:</strong> Cancelled</div>
                </div>
            )}
            {isLoading ? (
                <div className="mt-4 flex justify-center items-center space-x-2">
                    <div className="w-4 h-4 border-t-2 border-b-2 border-indigo-600 rounded-full animate-spin"></div>
                    <div className="text-sm font-medium text-gray-300">Processing transaction...
                        {txHash && <a href={`${explorerUrl}${txHash}`} target="_blank">{truncate(txHash, 8, 8)}</a>}
                    </div>
                </div>
            ) : (
                message && (
                    <div className="mt-4 text-sm font-medium">
                        <p className="text-red-600 break-all">{message}</p>
                    </div>
                )
            )}
        </li>
    );
}

export default Order;
