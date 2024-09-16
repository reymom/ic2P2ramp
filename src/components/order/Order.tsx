import React, { useEffect, useState } from 'react';
import { ethers } from 'ethers';
import { useAccount } from 'wagmi';

import icpLogo from "../../assets/blockchains/icp-logo.svg";

import { backend } from '../../declarations/backend';
import { OrderState, PaymentProvider, PaymentProviderType } from '../../declarations/backend/backend.did';
import { NetworkIds, NetworkProps } from '../../constants/networks';
import { defaultReleaseEvmGas, getEvmTokenOptions, getIcpTokenOptions, defaultCommitEvmGas, TokenOption } from '../../constants/tokens';
import { tokenLogos } from '../../constants/addresses';
import { blockchainToBlockchainType, paymentProviderTypeToString, providerToProviderType } from '../../model/utils';
import { formatTimeLeft, truncate } from '../../model/helper';
import { rampErrorToString } from '../../model/error';
import { estimateGasAndGasPrice, withdrawFromVault } from '../../model/evm';
import { PaymentProviderTypes } from '../../model/types';
import PayPalButton from '../PaypalButton';
import { useUser } from '../user/UserContext';

interface OrderProps {
    order: OrderState;
    refetchOrders: () => void;
}

const defaultLoadingMessage = "Processing Transaction ...";
const lockTimeSeconds = 120;

const Order: React.FC<OrderProps> = ({ order, refetchOrders }) => {
    const [committedProvider, setCommittedProvider] = useState<[PaymentProviderType, PaymentProvider]>();
    const [isLoading, setIsLoading] = useState(false);
    const [loadingMessage, setLoadingMessage] = useState(defaultLoadingMessage);
    const [message, setMessage] = useState('');
    const [txHash, setTxHash] = useState<string>();
    const [remainingTime, setRemainingTime] = useState<number | null>(null);

    const { chainId } = useAccount();
    const { user, userType, sessionToken, fetchIcpBalance, refetchUser } = useUser();

    const orderId = 'Created' in order ? order.Created.id
        : 'Locked' in order ? order.Locked.base.id
            : null;

    const baseOrder =
        'Created' in order ? order.Created
            : 'Locked' in order ? order.Locked.base : null

    const orderBlockchain = 'Created' in order ? order.Created.crypto.blockchain
        : 'Locked' in order ? order.Locked.base.crypto.blockchain
            : 'Completed' in order ? order.Completed.blockchain
                : null;

    const orderFiatAmount = 'Created' in order ? order.Created.fiat_amount + order.Created.offramper_fee
        : 'Locked' in order ? order.Locked.base.fiat_amount + order.Locked.base.offramper_fee
            : 'Completed' in order ? order.Completed.fiat_amount + order.Completed.offramper_fee
                : null;

    const getToken = (): TokenOption | undefined => {
        const crypto = 'Created' in order ? order.Created.crypto
            : 'Locked' in order ? order.Locked.base.crypto
                : null
        if (!crypto) return undefined;

        const tokens = 'EVM' in crypto.blockchain ? getEvmTokenOptions(Number(crypto.blockchain.EVM.chain_id))
            : 'ICP' in crypto.blockchain ? getIcpTokenOptions() : null;
        if (!tokens) return undefined;

        const tokenAddress = 'EVM' in crypto.blockchain ? crypto.token?.[0] ?? '' :
            'ICP' in crypto.blockchain ? crypto.blockchain.ICP.ledger_principal.toString() : '';

        return tokens.find(token => {
            return token.address === tokenAddress;
        })
    }

    const token = getToken();

    useEffect(() => {
        if ('Locked' in order) {
            const calculateRemainingTime = () => {
                const currentTime = Number(Date.now() * 1_000_000);
                const expiryTime = Number(order.Locked.locked_at) + lockTimeSeconds * 1_000_000_000;
                const timeLeftSeconds = (expiryTime - currentTime) / 1_000_000_000;

                if (!order.Locked.payment_done && timeLeftSeconds <= 0) {
                    setTimeout(() => {
                        refetchOrders();
                    }, 2500);
                };
                setRemainingTime(timeLeftSeconds > 0 ? timeLeftSeconds : null);
            };

            calculateRemainingTime();

            const timer = setInterval(calculateRemainingTime, 1000);
            return () => clearInterval(timer);
        }
    }, [order]);

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
        setLoadingMessage(`Commiting to order #${orderId} ...`);

        let gasEstimation: [] | [bigint] = [];
        if ('EVM' in orderBlockchain) {
            const gasForCommit = await estimateGasAndGasPrice(
                Number(orderBlockchain.EVM.chain_id),
                { Commit: null },
                defaultCommitEvmGas,
            );
            console.log("[commitToOrder] gasCommitEstimate = ", gasForCommit);
            gasEstimation = [gasForCommit[0]];
        }

        const onramperAddress = user.addresses.find(addr => Object.keys(orderBlockchain)[0] in addr.address_type);
        if (!onramperAddress) throw new Error("No address matches for user");
        console.log("onramperAddress = ", onramperAddress);

        try {
            const result = await backend.lock_order(orderId, sessionToken, user.id, provider, onramperAddress, gasEstimation);
            if ('Ok' in result) {
                if ('EVM' in orderBlockchain) {
                    setTxHash(result.Ok);

                    const provider = new ethers.BrowserProvider(window.ethereum);
                    provider.once(result.Ok, (transactionReceipt) => {
                        console.log("provider.once transactionReceipt = ", transactionReceipt)
                        if (transactionReceipt.status === 1) {
                            setMessage("Order Locked!");
                            setTxHash(undefined);
                            setTimeout(() => {
                                refetchOrders();
                                refetchUser();
                            }, 4500);
                        } else {
                            const errorMessage = "Transaction failed!";
                            setMessage(errorMessage);
                            setTxHash(undefined);
                        }
                        setIsLoading(false);
                    });
                } else {
                    setMessage("Order Locked!");
                    setLoadingMessage(defaultLoadingMessage);
                    setTimeout(() => {
                        refetchOrders();
                        refetchUser();
                    }, 2500);
                    setIsLoading(false);
                }
            } else {
                const errorMessage = rampErrorToString(result.Err);
                setMessage(errorMessage);
                setIsLoading(false);
            }
        } catch (err) {
            setMessage(`Error commiting to order ${orderId}: ${err}`);
            setIsLoading(false);
            console.error(err);
        } finally {
            console.log("finally");
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

        // estimate release gas
        let gasEstimation: [] | [bigint] = [];
        const hasToken = order.Locked.base.crypto.token.length > 0;
        if ('EVM' in orderBlockchain) {
            const gasForRelease = await estimateGasAndGasPrice(
                Number(orderBlockchain.EVM.chain_id),
                hasToken ? { ReleaseToken: null } : { ReleaseNative: null },
                defaultReleaseEvmGas
            );
            console.log("[handlePayPalSuccess] gasReleaseEstimate = ", gasForRelease);
            gasEstimation = [gasForRelease[0]];
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
                        console.log("[paypal] transactionReceipt = ", transactionReceipt)
                        if (transactionReceipt.status === 1) {
                            setMessage("Order Completed!");
                            setIsLoading(false);
                            setTxHash(undefined);
                            setTimeout(() => {
                                refetchOrders();
                                refetchUser();
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
                        refetchUser();
                    }, 2000);
                }
            } else {
                setIsLoading(false);
                const errorMessage = rampErrorToString(response.Err);
                setMessage(errorMessage);
            }
        } catch (err) {
            setIsLoading(false);
            setMessage(`Error verifying payment for order ${orderId.toString()}.`);
            console.error(err);
        } finally {
            console.log("finally")
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

    const getTokenName = (): string => {
        return token ? token.name : ""
    }

    const getTokenSymbol = (): string => {
        return token ? token.rateSymbol : "Unknown";
    };

    const getTokenDecimals = (): number => {
        if (!token) throw new Error("Token not found");
        return token.decimals
    }

    const tokenLogo = tokenLogos[getTokenName()] || null;

    const getNetwork = (): NetworkProps | undefined => {
        return (orderBlockchain && 'EVM' in orderBlockchain) ?
            Object.values(NetworkIds).find(network => network.id === Number(orderBlockchain.EVM.chain_id)) : undefined
    }

    const getNetworkExplorer = (): string | undefined => {
        return getNetwork()?.explorer
    }

    const getNetworkLogo = (): string | undefined => {
        if (!orderBlockchain) return "";
        if ('EVM' in orderBlockchain) {
            return getNetwork()!.logo;
        } else if ('ICP' in orderBlockchain) {
            return icpLogo
        }
    }

    const getNetworkName = (): string | undefined => {
        if (!orderBlockchain) return "";
        if ('EVM' in orderBlockchain) {
            return getNetwork()!.name;
        } else if ('ICP' in orderBlockchain) return 'ICP';
    }

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
                let fullAmountEVM: string;
                if (token?.isNative) {
                    fullAmountEVM = ethers.formatEther(crypto.amount - crypto.fee);
                } else {
                    fullAmountEVM = ethers.formatUnits(
                        (crypto.amount - crypto.fee).toString(),
                        getTokenDecimals()
                    );
                }
                const shortAmountEVM = parseFloat(fullAmountEVM).toPrecision(3);
                return { fullAmount: fullAmountEVM, shortAmount: shortAmountEVM };
            case 'ICP': backgroundColor
                const fullAmountICP = (Number(crypto.amount - crypto.fee) / 10 ** getTokenDecimals()).toString();
                const shortAmountICP = parseFloat(fullAmountICP).toPrecision(3);
                return { fullAmount: fullAmountICP, shortAmount: shortAmountICP };
            case 'Solana':
                return { fullAmount: "Solana not implemented", shortAmount: "Solana not implemented" };
        }
    }

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

    let textColor = 'Created' in order || 'Locked' in order ? "text-white" : "text-gray-200";

    const commonOrderDiv = crypto && token && (
        <div className="space-y-3">
            {/* Fiat and Crypto Amount */}
            <div className="text-lg flex justify-between">
                <span className="opacity-90">Price:</span>
                <span className="font-medium flex items-center space-x-2">
                    <span>{formatFiatAmount()}</span>
                    <span className="border border-white bg-amber-600 rounded-full h-5 w-5 flex items-center justify-center text-sm leading-none">
                        $
                    </span>
                </span>
            </div>
            <div className="text-lg flex justify-between">
                <span className="opacity-90">Amount:</span>
                <span className="font-medium flex items-center space-x-2" title={formatCryptoAmount()?.fullAmount}>
                    <span>{formatCryptoAmount()?.shortAmount}</span>
                    {tokenLogo && (
                        <img
                            src={tokenLogo}
                            alt={getTokenSymbol()}
                            title={getTokenSymbol()}
                            className="h-5 w-5 inline-block border border-white bg-gray-100 rounded-full"
                        />
                    )}
                </span>
            </div>

            {/* Offramper Address */}
            <div className="text-lg flex justify-between">
                <span className="opacity-90">Address:</span>
                <span className="font-medium">
                    {orderBlockchain && 'EVM' in orderBlockchain ? (
                        <a
                            href={`${getNetworkExplorer()}/address/${baseOrder!.offramper_address.address}`}
                            target="_blank"
                            rel="noopener noreferrer"
                            className="text-white hover:text-gray-400 transition-colors duration-200"
                            title="View on Block Explorer"
                        >
                            {truncate(baseOrder!.offramper_address.address, 8, 8)}
                        </a>
                    ) :
                        <span className="font-medium">{truncate(baseOrder!.offramper_address.address, 8, 8)}</span>
                    }
                </span>
            </div>

            {'EVM' in baseOrder!.crypto.blockchain && (
                <div className="text-lg flex justify-between">
                    <span className="opacity-80">Network:</span>
                    <img
                        src={getNetworkLogo()}
                        alt={getNetworkName()}
                        title={getNetworkName()}
                        className="h-5 w-5" />
                </div>
            )}
        </div>
    )

    return (
        <li className={`px-14 pt-10 pb-8 border rounded-xl shadow-md ${backgroundColor} ${borderColor} ${textColor} relative`}>
            {getNetworkLogo() && (
                <div className="absolute top-2.5 left-2.5 h-8 w-8">
                    <div className="relative inline-block">
                        <img
                            src={getNetworkLogo()}
                            alt="Blockchain Logo"
                            title={getNetworkName()}
                            className="h-8 w-8"
                        />
                        {tokenLogo && (
                            <img
                                src={tokenLogo}
                                alt={getTokenSymbol()}
                                title={getTokenSymbol()}
                                className="h-4 w-4 absolute -bottom-0.5 -right-0.5 border border-white rounded-full bg-gray-100 bg-opacity-100"
                            />
                        )}
                    </div>
                </div>
            )}
            {'Created' in order && (
                <div className="flex flex-col">
                    {commonOrderDiv}

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
                                className={`mt-3 px-4 py-2 rounded w-full font-medium ${disabled ? 'bg-gray-500 cursor-not-allowed' : 'bg-green-700 hover:bg-green-800'}`}
                                disabled={disabled}
                            >
                                Lock Order (1h)
                            </button>
                        );
                    })()}

                    {/* Remove Button for Offramper */}
                    {user && userType === 'Offramper' && order.Created.offramper_user_id === user.id && (
                        <button
                            onClick={removeOrder}
                            className="mt-3 px-4 py-2 bg-red-700 rounded w-full font-medium hover:bg-red-800"
                        >
                            Remove
                        </button>
                    )}
                </div>
            )}
            {'Locked' in order && (
                <div className="flex flex-col">
                    {commonOrderDiv}

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

                    {remainingTime !== null && (
                        <div className="text-sm text-gray-200 mt-2">
                            (Locked for {formatTimeLeft(remainingTime)})
                        </div>
                    )}

                </div>
            )}
            {'Completed' in order && (
                <div className="flex flex-col space-y-3">
                    <div className="text-lg flex justify-between">
                        <span className="opacity-90">Fiat Amount:</span>
                        <span className="font-medium flex items-center space-x-2">
                            <span>{formatFiatAmount()}</span>
                            <span className="border border-white bg-amber-600 rounded-full h-5 w-5 flex items-center justify-center text-sm leading-none">
                                $
                            </span>
                        </span>
                    </div>

                    <div className="text-lg flex justify-between">
                        <span className="opacity-90">Onramper:</span>
                        <span className="font-medium">
                            {orderBlockchain && 'EVM' in orderBlockchain ? (
                                <a
                                    href={`${getNetworkExplorer()}/address/${order.Completed.onramper.address}`}
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    className="text-white hover:text-gray-400 transition-colors duration-200"
                                    title="View on Block Explorer"
                                >
                                    {truncate(order.Completed.onramper.address, 8, 8)}
                                </a>
                            ) :
                                <span className="font-medium">{truncate(order.Completed.onramper.address, 8, 8)}</span>
                            }
                        </span>
                    </div>
                    <div className="text-lg flex justify-between">
                        <span className="opacity-90">Offramper:</span>
                        <span className="font-medium">
                            {orderBlockchain && 'EVM' in orderBlockchain ? (
                                <a
                                    href={`${getNetworkExplorer()}/address/${order.Completed.offramper.address}`}
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    className="text-white hover:text-gray-400 transition-colors duration-200"
                                    title="View on Block Explorer"
                                >
                                    {truncate(order.Completed.offramper.address, 8, 8)}
                                </a>
                            ) :
                                <span className="font-medium">{truncate(order.Completed.offramper.address, 8, 8)}</span>
                            }
                        </span>
                    </div>

                    {'EVM' in order.Completed.blockchain && (
                        <div className="text-lg flex justify-between">
                            <span className="opacity-80">Network:</span>
                            <img
                                src={getNetworkLogo()}
                                alt={getNetworkName()}
                                title={getNetworkName()}
                                className="h-5 w-5" />
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
                    <div className="text-sm font-medium text-gray-300">{loadingMessage}&nbsp;
                        {txHash && <a href={`${getNetworkExplorer()}/tx/${txHash}`} target="_blank">{truncate(txHash, 8, 8)}</a>}
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
