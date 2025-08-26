import React from 'react';

import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';

import { OrderState } from '@/declarations/icramp_backend/icramp_backend.did';
import { useUser } from '@/components/user/UserContext';
import { CURRENCY_ICON_MAP } from '@/constants/currencyIconsMap';
import { paymentProviderTypeToString } from '@/model/utils/utils';
import { formatPrice, formatTimeLeft, truncate } from '@/utils/helper';
import PayPalButton from '@/components/ui/PaypalButton';
import DynamicDots from '@/components/ui/DynamicDots';
import { useOrderLogic } from './useOrderLogic';

interface OrderProps {
    order: OrderState;
    refetchOrders: () => void;
}

const OrderCard: React.FC<OrderProps> = ({ order, refetchOrders }) => {
    const {
        orderState,
        baseOrder,
        orderBlockchainAsset,
        token,
        cryptoAmount,
        currentPrice,
        isLoading,
        loadingMessage,
        message,
        txHash,
        remainingTime,
        isPayable,
        loadingPayable,
        loadingPrice,
        committedProvider,
        getStatusColors,
        getNetworkLogo,
        getNetworkName,
        getExplorerLinks,
        handleProviderSelection,
        commitToOrder,
        removeOrder,
        handlePayPalSuccess,
        handleRevolutRedirect,
    } = useOrderLogic(order, refetchOrders);

    const { backgroundColor, borderColor, textColor } = getStatusColors();
    const { user, userType } = useUser();

    const commonOrderDiv = baseOrder && orderBlockchainAsset && (
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
                    {token && (
                        <img
                            src={token.logo}
                            alt={token.name}
                            title={token.name}
                            className="h-5 w-5 inline-block border border-white bg-gray-100 rounded-full"
                        />
                    )}
                </span>
            </div>

            {/* Offramper Address */}
            <div className="text-lg flex justify-between">
                <span className="opacity-90">Address:</span>
                <span className="font-medium">
                    <a
                        href={`${getExplorerLinks(baseOrder.offramper_address.address)?.address}`}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="text-blue-800 dark:text-white hover:text-blue-700 dark:hover:text-gray-400 transition-colors duration-200"
                        title="View on Explorer"
                    >
                        {truncate(baseOrder.offramper_address.address, 8, 8)}
                    </a>
                </span>
            </div>

            {'EVM' in baseOrder!.crypto.asset && (
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
                        <div className="text-black dark:text-white text-2xl font-bold mt-2">
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
                        {token && (
                            <img
                                src={token.logo}
                                alt={token.name}
                                title={token.name}
                                className="h-4 w-4 absolute -bottom-0.5 -right-0.5 border border-white rounded-full bg-gray-100 bg-opacity-100"
                            />
                        )}
                    </div>
                </div>
            )}
            {'Created' in orderState && (
                <div className="flex flex-col">
                    {commonOrderDiv}

                    <hr className="border-t border-gray-500 w-full my-3" />

                    {/* Providers */}
                    <div className="text-lg">
                        <span className="opacity-90">Payment Methods:</span>
                        <div className="font-medium">
                            {orderState.Created.offramper_providers.map((provider, index) => {
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
                                Object.keys(addr.address_type)[0] === Object.keys(orderState.Created.offramper_address.address_type)[0]
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
                    {user && userType === 'Offramper' && orderState.Created.offramper_user_id === user.id && (
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
            {'Locked' in orderState && (
                <div className="flex flex-col">
                    {commonOrderDiv}

                    {user && userType === 'Onramper' && orderState.Locked.onramper.user_id === user.id && !orderState.Locked.uncommited && (
                        <>
                            <div>
                                {orderState.Locked.onramper.provider.hasOwnProperty('PayPal') ? (
                                    <PayPalButton
                                        orderId={orderState.Locked.base.id.toString()}
                                        amount={Number(orderState.Locked.price + orderState.Locked.offramper_fee) / 100.}
                                        currency={orderState.Locked.base.currency}
                                        paypalId={(() => {
                                            const provider = orderState.Locked.base.offramper_providers.find(
                                                provider => 'PayPal' in provider[1]
                                            );
                                            if (provider && 'PayPal' in provider[1]) {
                                                return provider[1].PayPal.id;
                                            }
                                            return '';
                                        })()}
                                        onSuccess={(transactionId) => handlePayPalSuccess(transactionId)}
                                        disabled={!isPayable || isLoading || orderState.Locked.payment_done}
                                    />
                                ) : orderState.Locked.onramper.provider.hasOwnProperty('Revolut') ? (
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
                                {!orderState.Locked.payment_done && !loadingPayable && !isPayable && (
                                    "This order cannot be paid at the moment. Please contact support or try again later."
                                )}
                                {orderState.Locked.payment_done && (
                                    "Payment is validated but couldn't release your funds. Please contact support to solve this issue."
                                )}
                            </div>
                        </>
                    )}

                    {remainingTime !== null && (
                        <div className="text-sm text-gray-600 dark:text-gray-200 mt-2">
                            (Locked for {formatTimeLeft(remainingTime)})
                        </div>
                    )}
                </div>
            )}
            {'Completed' in orderState && (
                <div className="flex flex-col space-y-3">
                    <div className="text-lg flex justify-between">
                        <span className="opacity-90">Fiat Amount:</span>
                        <span className="font-medium flex items-center space-x-2">
                            <span>{formatPrice(Number(orderState.Completed.price))}</span>
                            <span className="border border-white bg-amber-600 rounded-full h-5 w-5 flex items-center justify-center text-sm leading-none">
                                $
                            </span>
                        </span>
                    </div>

                    <div className="text-lg flex justify-between">
                        <span className="opacity-90">Onramper:</span>
                        <span className="font-medium">
                            <a
                                href={`${getExplorerLinks(orderState.Completed.onramper.address)?.address}`}
                                target="_blank"
                                rel="noopener noreferrer"
                                className="text-black dark:text-white hover:text-gray-400 transition-colors duration-200"
                                title="View on Block Explorer"
                            >
                                {truncate(orderState.Completed.onramper.address, 8, 8)}
                            </a>

                        </span>
                    </div>
                    <div className="text-lg flex justify-between">
                        <span className="opacity-90">Offramper:</span>
                        <span className="font-medium">
                            <a
                                href={`${getExplorerLinks(orderState.Completed.offramper.address)?.address}`}
                                target="_blank"
                                rel="noopener noreferrer"
                                className="text-black dark:text-white hover:text-gray-400 transition-colors duration-200"
                                title="View on Block Explorer"
                            >
                                {truncate(orderState.Completed.offramper.address, 8, 8)}
                            </a>
                        </span>
                    </div>

                    {'EVM' in orderState.Completed.asset && (
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
            {'Cancelled' in orderState && (
                <div className="flex flex-col space-y-3">
                    <div><strong>Order ID:</strong> {orderState.Cancelled.toString()}</div>
                    <div><strong>Status:</strong> Cancelled</div>
                </div>
            )}

            {!message && txHash && (
                <div className="relative mt-2 text-xs text-blue-400 z-50 flex items-center justify-center text-center">
                    <a href={`${getExplorerLinks("", txHash)?.transaction}`} target="_blank" className="hover:underline z-50">
                        View Transaction {truncate(txHash, 5, 5)}
                    </a>
                </div>
            )}

            {isLoading && ('Bitcoin' in orderBlockchainAsset!) && txHash && (
                <div className="relative mt-4 text-sm font-medium flex items-center justify-center text-center flex-col z-50">
                    <p className="mb-2">
                        Bitcoin transactions typically take 15-60+ minutes to confirm.
                    </p>
                    <p className="text-amber-500 dark:text-amber-400">
                        Your funds will be released automatically once confirmed.
                    </p>
                </div>
            )}

            {message && (
                <div className="relative mt-4 text-sm font-medium flex items-center justify-center text-center z-50">
                    <p className="text-red-600">{message}&nbsp;</p>
                    {txHash &&
                        <a href={`${getExplorerLinks("", txHash)?.transaction}`} target="_blank" className="text-red-500 hover:underline z-50">
                            View tx: {truncate(txHash, 6, 6)}
                        </a>
                    }
                </div>
            )}
        </li>
    );
}

export default OrderCard;
