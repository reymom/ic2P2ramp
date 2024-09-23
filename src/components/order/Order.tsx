import React, { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { ethers } from 'ethers';

import icpLogo from "../../assets/blockchains/icp-logo.svg";

import { backend } from '../../declarations/backend';
import { OrderState, PaymentProvider, PaymentProviderType } from '../../declarations/backend/backend.did';
import { NetworkIds, NetworkProps } from '../../constants/networks';
import { defaultReleaseEvmGas, getEvmTokenOptions, getIcpTokenOptions, defaultCommitEvmGas, TokenOption } from '../../constants/tokens';
import { tokenLogos } from '../../constants/addresses';
import { blockchainToBlockchainType, paymentProviderTypeToString, providerToProviderType } from '../../model/utils';
import { formatCryptoUnits, formatPrice, formatTimeLeft, truncate } from '../../model/helper';
import { rampErrorToString } from '../../model/error';
import { estimateGasAndGasPrice } from '../../model/evm';
import { PaymentProviderTypes } from '../../model/types';
import PayPalButton from '../PaypalButton';
import { useUser } from '../user/UserContext';
import DynamicDots from '../ui/DynamicDots';
import { fetchOrderPrice } from '../../model/rate';
import { CURRENCY_ICON_MAP } from '../../constants/currencyIconsMap';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';

interface OrderProps {
    order: OrderState;
    refetchOrders: () => void;
}

const defaultLoadingMessage = "Processing Transaction";
const PRICE_DIFFERENCE_THRESHOLD = 0.025;
const CACHE_EXPIRY_MS = 1800000; // 30 min, but backend is caching it to 10 minuts
const LOCK_TIME_SECONDS = 1800;

const Order: React.FC<OrderProps> = ({ order, refetchOrders }) => {
    const [committedProvider, setCommittedProvider] = useState<[PaymentProviderType, PaymentProvider]>();
    const [isLoading, setIsLoading] = useState(false);
    const [loadingMessage, setLoadingMessage] = useState(defaultLoadingMessage);
    const [currentPrice, setCurrentPrice] = useState<bigint | null>(null);
    const [loadingPrice, setLoadingPrice] = useState<boolean>(false);
    const [txHash, setTxHash] = useState<string | null>(null);
    const [message, setMessage] = useState<string | null>(null);
    const [remainingTime, setRemainingTime] = useState<number | null>(null);
    const [isPayable, setIsPayable] = useState<boolean>(false);
    const [loadingPayable, setLoadingPayable] = useState<boolean>(true);

    const { user, userType, sessionToken, fetchBalances, refetchUser } = useUser();
    const navigate = useNavigate();

    const orderId = 'Created' in order ? order.Created.id
        : 'Locked' in order ? order.Locked.base.id
            : null;

    const committedMessage = `Locked Order #${orderId}, refetching data`;
    const releasedMessage = `Order Verified and Funds Released. Refetching data`;
    const cancelledMessage = `Cancelled Order #${orderId}, refetching data`;

    const baseOrder =
        'Created' in order ? order.Created
            : 'Locked' in order ? order.Locked.base : null

    const orderBlockchain = 'Created' in order ? order.Created.crypto.blockchain
        : 'Locked' in order ? order.Locked.base.crypto.blockchain
            : 'Completed' in order ? order.Completed.blockchain
                : null;

    useEffect(() => {
        const getCurrentPrice = async () => {
            if ('Locked' in order) {
                setCurrentPrice(order.Locked.price + order.Locked.offramper_fee);
                return;
            };
            if (!('Created' in order)) return;

            const cachedPriceData = localStorage.getItem(`order_${orderId}_price`);
            if (cachedPriceData) {
                const { price, timestamp } = JSON.parse(cachedPriceData);
                const now = Date.now();
                if (now - timestamp < CACHE_EXPIRY_MS) {
                    setCurrentPrice(BigInt(price));
                    return;
                }
            }

            if (!baseOrder || !token || !token.rateSymbol) return;
            setLoadingPrice(true);
            fetchOrderPrice(baseOrder.currency, baseOrder.crypto)
                .then(([price, offramperFee]) => {
                    const totalPrice = price + offramperFee;
                    localStorage.setItem(`order_${orderId}_price`, JSON.stringify({
                        price: Number(totalPrice),
                        timestamp: Date.now(),
                    }));
                    setCurrentPrice(totalPrice);
                    setLoadingPrice(false);
                })
                .catch((err) => {
                    console.error("Error fetching order price:", err);
                    setLoadingPrice(false);
                    setCurrentPrice(null);
                    setMessage(err);
                });
        }

        getCurrentPrice();
    }, []);

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
                const expiryTime = Number(order.Locked.locked_at) + LOCK_TIME_SECONDS * 1_000_000_000;
                const timeLeftSeconds = (expiryTime - currentTime) / 1_000_000_000;

                if (!order.Locked.payment_done && timeLeftSeconds <= 0) {
                    setTimeout(() => {
                        refetchOrders();
                        refetchUser();
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

    const pollTransactionLog = async (orderId: bigint, userId: bigint, maxAttempts = 25) => {
        if (!sessionToken) throw new Error("Please authenticate to get a token session");

        let attempts = 0;
        let pollingTimer: NodeJS.Timeout | null = null;

        const clearPolling = () => {
            if (pollingTimer) clearTimeout(pollingTimer);
        };

        const pollLog = async () => {
            console.log(`[pollTransactionLog] Polling attempt: ${attempts}, maxAttempts: ${maxAttempts}`);
            if (attempts >= maxAttempts) {
                clearPolling();
                setMessage("Transaction is still pending after multiple attempts. Please check manually.");
                setIsLoading(false);
                return;
            }

            attempts += 1;

            try {
                const logResult = await backend.get_transaction_log(orderId, userId, sessionToken);
                console.log("[pollTransactionLog] logResult = ", logResult);

                if ('Ok' in logResult && logResult.Ok.length > 0 && logResult.Ok[0]) {
                    const transactionLog = logResult.Ok[0];
                    console.log("[pollTransactionLog] Transaction Log:", transactionLog);

                    if ('Confirmed' in transactionLog.status) {
                        const receipt = transactionLog.status.Confirmed;
                        const successMessage = 'Commit' in transactionLog.action ? committedMessage
                            : 'Release' in transactionLog.action ? releasedMessage
                                : 'Cancel' in transactionLog.action ? cancelledMessage
                                    : "Transaction is successful!"

                        setLoadingMessage(successMessage);
                        setTxHash(receipt.transactionHash);
                        setTimeout(() => {
                            refetchOrders();
                            refetchUser();
                            fetchBalances();
                            setIsLoading(false);
                            navigate(
                                'Commit' in transactionLog.action ? `/view?onramperId=${user!.id}` :
                                    'Release' in transactionLog.action ? "/view?status=Completed" :
                                        'Cancel' in transactionLog.action ? "/view?status=Cancelled" : ""
                            );
                        }, 3500);
                        return;
                    } else if ('Failed' in transactionLog.status) {
                        console.log("[pollTransactionLog] Transaction Failed:", transactionLog.status.Failed);
                        setMessage("Transaction failed.");
                        setIsLoading(false);
                        clearPolling();
                        return;
                    }
                } else if ('Err' in logResult) {
                    setMessage(`Transaction failed: ${rampErrorToString(logResult.Err)}`)
                    setIsLoading(false);
                    setTxHash(null);
                    clearPolling();
                    return;
                }

                // If still pending, poll again after a short delay
                pollingTimer = setTimeout(pollLog, 4000);

            } catch (error) {
                console.error("Error polling transaction logs: ", error);
                setMessage("Failed to retrieve transaction status");
                setIsLoading(false);
                clearPolling();
            };
        };

        pollLog();
        return () => clearPolling();
    }

    const checkIfOrderIsPayable = async (orderId: bigint, tokenSession: string): Promise<boolean> => {
        if (user && order && 'Locked' in order) {
            try {
                const result = await backend.verify_order_is_payable(orderId, tokenSession);
                if ('Ok' in result) {
                    if (order.Locked.onramper.user_id === user.id) {
                        return true
                    }
                    return false;
                } else {
                    console.error('Order is not payable:', result.Err);
                    return false;
                }
            } catch (error) {
                console.error('Error checking order payable status:', error);
                return false;
            }
        } else {
            return false
        }
    };

    useEffect(() => {
        const validateOrderPayable = async () => {
            if (order && 'Locked' in order) {
                const payable = await checkIfOrderIsPayable(order.Locked.base.id, sessionToken!);
                setIsPayable(payable);
                setLoadingPayable(false);
            }
        };

        if (order && sessionToken) {
            validateOrderPayable();
        } else {
            setLoadingPayable(false);
        }
    }, [sessionToken]);

    const commitToOrder = async (provider: PaymentProvider) => {
        if (!sessionToken) throw new Error("Please authenticate to get a token session");
        if (!user || !('Onramper' in user.user_type) || !('Created' in order) || !(orderBlockchain) || !orderId) return;

        setIsLoading(true);
        setTxHash(null);
        setMessage(null);
        setLoadingMessage("Fetching order price");

        try {
            const [orderPrice, offramperFee] = await fetchOrderPrice(baseOrder!.currency, baseOrder!.crypto);
            const priceDifference = Math.abs(Number((orderPrice + offramperFee - currentPrice!) / currentPrice!));
            // if (priceDifference > PRICE_DIFFERENCE_THRESHOLD) {
            const confirm = window.confirm(
                `The real price differs significantly from the previously estimated price. 
                    Real price: $${formatPrice(Number(orderPrice + offramperFee))}, estimated price: $${formatPrice(Number(currentPrice))}. 
                    Do you want to proceed?`
            );
            if (!confirm) {
                setIsLoading(false);
                return;
            }
            // }
        } catch (e) {
            console.error("[commitToOrder] e: ", e);
            setMessage("Could not set order price.");
            return;
        }

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

        try {
            const result = await backend.lock_order(orderId, sessionToken, user.id, provider, onramperAddress, gasEstimation);
            if ('Ok' in result) {
                if ('EVM' in orderBlockchain) {
                    setTxHash(result.Ok);
                    console.log(`[commitToOrder] Transaction Hash: ${result.Ok}`);
                    pollTransactionLog(orderId, user.id);
                } else {
                    setLoadingMessage(committedMessage);
                    setTimeout(() => {
                        setIsLoading(false);
                        refetchOrders();
                        refetchUser();
                        fetchBalances();
                        navigate(`/view?onramperId=${user.id}`);
                    }, 2500);
                }
            } else {
                setMessage(rampErrorToString(result.Err));
                setIsLoading(false);
            }
        } catch (err) {
            setMessage(`Error while committing to order ${orderId}.`);
            setIsLoading(false);
            console.error(err);
        }
    };

    const removeOrder = async () => {
        if (!sessionToken) throw new Error("Please authenticate to get a token session")
        if (!user || !('Offramper' in user?.user_type) || !('Created' in order) || !orderBlockchain || !orderId) return;
        if (!baseOrder || user.id !== baseOrder.offramper_user_id) return;

        const scrollPosition = window.scrollY;

        setIsLoading(true);
        setTxHash(null);
        setMessage(null)
        setLoadingMessage(`Removing order ${orderId}`);

        try {
            const result = await backend.cancel_order(orderId, sessionToken);
            if ('Ok' in result) {
                if ('EVM' in orderBlockchain) {
                    setTxHash(result.Ok)
                    pollTransactionLog(orderId, user.id);
                } else {
                    setLoadingMessage(cancelledMessage);
                    setTimeout(() => {
                        refetchOrders();
                        refetchUser();
                        fetchBalances();
                        setIsLoading(false);

                        window.scrollTo(0, scrollPosition);
                    }, 2500);
                }
            } else {
                setMessage(rampErrorToString(result.Err));
                setIsLoading(false);
            }
        } catch (err) {
            setMessage(`Error while removing order ${orderId}.`);
            setIsLoading(false);
            console.error(err);
        }
    };

    const handlePayPalSuccess = async (transactionId: string) => {
        if (!sessionToken) throw new Error("Please authenticate to get a token session")
        if (!('Locked' in order) || !orderId || !orderBlockchain) return;
        if (!user || !('Onramper' in user.user_type)) return;

        console.log("[handlePayPalSuccess] transactionID = ", transactionId);

        setIsLoading(true);
        setTxHash(null);
        setMessage(null);
        setLoadingMessage(`Payment received. Verifying`);

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
                if ('EVM' in orderBlockchain!) {
                    setTxHash(response.Ok);
                    pollTransactionLog(orderId, user!.id);
                } else {
                    setLoadingMessage(releasedMessage);
                    setTimeout(() => {
                        refetchOrders();
                        refetchUser();
                        setIsLoading(false);
                        fetchBalances();
                        navigate("/view?completed");
                    }, 2500);
                }
            } else {
                setIsLoading(false);
                const errorMessage = rampErrorToString(response.Err);
                setMessage(errorMessage);
            }
        } catch (err) {
            setIsLoading(false);
            setMessage(`Error verifying payment for order ${orderId}.`);
            console.error(err);
        }
    };

    const handleRevolutRedirect = async () => {
        if (!sessionToken) throw new Error("Please authenticate to get a token session")
        if (!('Locked' in order) || !orderId) return;

        const consentUrl = order.Locked.revolut_consent[0]?.url;
        if (consentUrl) {
            console.log('Listening for Revolut transaction confirmation...');
            backend.execute_revolut_payment(orderId, sessionToken)
                .catch(err => console.error("Failed to execute revolut payment: ", err));
            window.location.href = consentUrl;
        } else {
            console.error('Consent URL is not available.');
        }
    };

    const getTokenName = (): string => {
        return token ? token.name : ""
    };

    const getTokenSymbol = (): string => {
        return token ? token.rateSymbol : "Unknown";
    };

    const getTokenDecimals = (): number => {
        if (!token) throw new Error("Token not found");
        return token.decimals
    };

    const tokenLogo = tokenLogos[getTokenName()] || null;

    const getNetwork = (): NetworkProps | undefined => {
        return (orderBlockchain && 'EVM' in orderBlockchain) ?
            Object.values(NetworkIds).find(network => network.id === Number(orderBlockchain.EVM.chain_id)) : undefined
    };

    const getNetworkExplorer = (): string | undefined => {
        return getNetwork()?.explorer
    };

    const getNetworkLogo = (): string | undefined => {
        if (!orderBlockchain) return "";
        if ('EVM' in orderBlockchain) {
            return getNetwork()!.logo;
        } else if ('ICP' in orderBlockchain) {
            return icpLogo
        }
    };

    const getNetworkName = (): string | undefined => {
        if (!orderBlockchain) return "";
        if ('EVM' in orderBlockchain) {
            return getNetwork()!.name;
        } else if ('ICP' in orderBlockchain) return 'ICP';
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
                const shortAmountEVM = formatCryptoUnits(parseFloat(fullAmountEVM));
                return { fullAmount: fullAmountEVM, shortAmount: shortAmountEVM };
            case 'ICP': backgroundColor
                const fullAmountICP = (Number(crypto.amount - crypto.fee) / 10 ** getTokenDecimals()).toString();
                const shortAmountICP = formatCryptoUnits(parseFloat(fullAmountICP));
                return { fullAmount: fullAmountICP, shortAmount: shortAmountICP };
            case 'Solana':
                return { fullAmount: "Solana not implemented", shortAmount: "Solana not implemented" };
        }
    };

    const cryptoAmount = formatCryptoAmount();

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
                    <span>
                        {loadingPrice ? (
                            <DynamicDots isLoading={loadingPrice} />
                        ) : currentPrice ? formatPrice(Number(currentPrice)) : ""}
                    </span>
                    <span className="border border-white bg-amber-600 rounded-full h-5 w-5 flex items-center justify-center text-sm leading-none">
                        <FontAwesomeIcon icon={CURRENCY_ICON_MAP[baseOrder!.currency]} className="text-gray-300" />
                    </span>
                </span>
            </div>
            <div className="text-lg flex justify-between">
                <span className="opacity-90">Amount:</span>
                <span className="font-medium flex items-center space-x-2" title={cryptoAmount?.fullAmount}>
                    <span>{cryptoAmount?.shortAmount}</span>
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
    );

    return (
        <li className={`px-14 pt-10 pb-8 border rounded-xl shadow-md ${backgroundColor} ${borderColor} ${textColor} relative`}>
            {isLoading && (
                <div className="absolute inset-0 rounded-xl bg-black bg-opacity-60 flex flex-col items-center justify-center z-40">
                    <div className="w-10 h-10 border-t-4 border-b-4 border-indigo-400 rounded-full animate-spin mb-4"></div>
                    {loadingMessage && (
                        <div className="text-white text-2xl font-bold mt-2">
                            {loadingMessage}<DynamicDots isLoading />
                        </div>
                    )}
                </div>
            )}

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
                                className={`mt-3 px-4 py-2 rounded-md w-full font-medium flex items-center justify-center ${disabled
                                    ? 'bg-gray-500 cursor-not-allowed' : 'bg-green-700 hover:bg-green-800'
                                    }`}
                                disabled={disabled || isLoading}
                            >
                                {isLoading ? (
                                    <>
                                        <div className="mr-2 w-4 h-4 border-t-2 border-b-2 border-white rounded-full animate-spin"></div>
                                        <span>Locking<DynamicDots isLoading /></span>
                                    </>
                                ) : (
                                    <span>Lock Order (1h)</span>
                                )}
                            </button>
                        );
                    })()}

                    {/* Remove Button for Offramper */}
                    {user && userType === 'Offramper' && order.Created.offramper_user_id === user.id && (
                        <button
                            onClick={removeOrder}
                            disabled={isLoading}
                            className="mt-3 px-4 py-2 bg-red-700 rounded-md w-full font-medium hover:bg-red-800 flex justify-center items-center"
                        >
                            {isLoading ? (
                                <>
                                    <div className="mr-2 w-4 h-4 border-t-2 border-b-2 border-white rounded-full animate-spin"></div>
                                    Removing<DynamicDots isLoading />
                                </>
                            ) : (
                                "Remove"
                            )}
                        </button>
                    )}
                </div>
            )}
            {'Locked' in order && (
                <div className="flex flex-col">
                    {commonOrderDiv}

                    {user && userType === 'Onramper' && order.Locked.onramper.user_id === user.id && !order.Locked.uncommited && (
                        <>
                            <div>
                                {order.Locked.onramper.provider.hasOwnProperty('PayPal') ? (
                                    <PayPalButton
                                        orderId={order.Locked.base.id.toString()}
                                        amount={Number(order.Locked.price + order.Locked.offramper_fee) / 100.}
                                        currency={order.Locked.base.currency}
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
                                        disabled={!isPayable || isLoading}
                                    />
                                ) : order.Locked.onramper.provider.hasOwnProperty('Revolut') ? (
                                    <div>
                                        <button
                                            className={`px-4 py-2 bg-blue-600 rounded-md hover:bg-blue-700 ${isPayable ? "cursor-not-allowed" : ""}`}
                                            onClick={handleRevolutRedirect}
                                            disabled={!isPayable || isLoading}
                                        >
                                            Confirm Revolut Consent
                                        </button>
                                    </div>
                                ) : null}
                            </div>
                            <div className="text-red-500 mt-2">
                                {!order.Locked.payment_done && !loadingPayable && !isPayable && (
                                    "This order cannot be paid at the moment. Please contact support or try again later."
                                )}
                                {order.Locked.payment_done && (
                                    "Payment is validated but couldn't release your funds. Please contact support to solve this issue."
                                )}
                            </div>
                        </>
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
                            <span>{formatPrice(Number(order.Completed.price))}</span>
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

            {!message && txHash && (
                <div className="relative mt-2 text-xs text-blue-400 z-50 flex items-center justify-center text-center">
                    <a href={`${getNetworkExplorer()}/tx/${txHash}`} target="_blank" className="hover:underline z-50">
                        View Transaction {truncate(txHash, 5, 5)}
                    </a>
                </div>
            )}

            {message && (
                <div className="relative mt-4 text-sm font-medium flex items-center justify-center text-center z-50">
                    <p className="text-red-600">{message}&nbsp;</p>
                    {txHash &&
                        <a href={`${getNetworkExplorer()}/tx/${txHash}`} target="_blank" className="text-red-500 hover:underline z-50">
                            View tx: {truncate(txHash, 6, 6)}
                        </a>
                    }
                </div>
            )}
        </li>
    );
}

export default Order;
