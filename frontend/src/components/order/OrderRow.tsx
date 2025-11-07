import React, { useState } from "react";

import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faEye } from '@fortawesome/free-solid-svg-icons';
import { OrderState } from "@/declarations/icramp_backend/icramp_backend.did";
import { formatPrice, truncate, formatTimeLeft } from "@/utils/formatters";
import { CURRENCY_ICON_MAP } from '@/constants/currencyIconsMap';
import { useOrderLogic } from "./hooks/useOrderLogic";
import OrderDetailModal from './OrderDetailModal';

interface OrderRowProps {
    order: OrderState;
    refetchOrders: () => Promise<void>;
}

const OrderRow: React.FC<OrderRowProps> = ({ order, refetchOrders }) => {
    const [showDetailModal, setShowDetailModal] = useState(false);

    const {
        orderState,
        baseOrder,
        token,
        cryptoAmount,
        currentPrice,
        isLoading,
        loadingPrice,
        remainingTime,
        getStatusColors,
        getNetworkLogo,
        getNetworkName,
        getExplorerLinks,
    } = useOrderLogic(order, refetchOrders);

    const { textColor } = getStatusColors();

    let state = '';
    if ('Created' in orderState) state = 'Created';
    else if ('Locked' in orderState) state = 'Locked';
    else if ('Completed' in orderState) state = 'Completed';
    else if ('Cancelled' in orderState) state = 'Cancelled';

    return (
        <>
            <tr className={`border-b border-gray-300 dark:border-gray-700 ${textColor} ${isLoading ? 'opacity-70' : ''} bg-white dark:bg-gray-900 hover:bg-gray-100 dark:hover:bg-gray-800 transition relative`}>
                {/* Loading overlay */}
                {isLoading && (
                    <td colSpan={7} className="absolute inset-0 bg-black bg-opacity-30 flex items-center justify-center z-10">
                        <div className="w-5 h-5 border-t-2 border-b-2 border-indigo-400 rounded-full animate-spin"></div>
                    </td>
                )}

                <td className="px-4 py-3">{state}</td>

                <td className="px-4 py-3">
                    {loadingPrice ? (
                        <span className="flex items-center">
                            <div className="w-4 h-4 border-t-2 border-b-2 border-indigo-400 rounded-full animate-spin"></div>
                        </span>
                    ) : (
                        <span className="flex items-center">
                            {currentPrice ? formatPrice(Number(currentPrice)) : "-"}
                            {baseOrder && (
                                <span className="border border-white bg-amber-600 rounded-full h-4 w-4 flex items-center justify-center text-xs leading-none ml-2">
                                    <FontAwesomeIcon icon={CURRENCY_ICON_MAP[baseOrder.currency]} className="text-gray-300" />
                                </span>
                            )}
                        </span>
                    )}
                </td>

                <td className="px-4 py-3">
                    <span className="flex items-center" title={cryptoAmount?.fullAmount}>
                        {cryptoAmount?.shortAmount || "-"}
                        {token && (
                            <img
                                src={token.logo}
                                alt={token.name}
                                title={token.name}
                                className="h-4 w-4 inline-block border border-white bg-gray-100 rounded-full ml-2"
                            />
                        )}
                    </span>
                </td>

                <td className="px-4 py-3">
                    {baseOrder ? (
                        <a
                            href={`${getExplorerLinks(baseOrder.offramper_address.address)?.address}`}
                            target="_blank"
                            rel="noopener noreferrer"
                            className="text-blue-800 dark:text-white hover:text-blue-700 dark:hover:text-gray-400 transition-colors duration-200"
                            title="View on Block Explorer"
                        >
                            {truncate(baseOrder.offramper_address.address, 6, 6)}
                        </a>
                    ) : "-"}
                </td>

                <td className="px-4 py-3">
                    <div className="flex items-center">
                        {getNetworkLogo() && (
                            <div className="flex items-center">
                                <img src={getNetworkLogo()} alt={getNetworkName()} title={getNetworkName()} className="h-5 w-5" />
                                {token && (
                                    <img
                                        src={token.logo}
                                        alt={token.name}
                                        title={token.name}
                                        className="h-3 w-3 -ml-1 mt-3 border border-white rounded-full bg-gray-100"
                                    />
                                )}
                            </div>
                        )}
                    </div>
                </td>

                <td className="px-4 py-3">
                    {remainingTime !== null && (
                        <span className="text-sm opacity-75">
                            {formatTimeLeft(remainingTime)}
                        </span>
                    )}
                </td>

                <td className="px-4 py-3">
                    <button
                        onClick={() => setShowDetailModal(true)}
                        className="px-3 py-1 bg-blue-600 hover:bg-blue-700 text-white rounded-md text-sm flex items-center"
                    >
                        <FontAwesomeIcon icon={faEye} className="mr-1" />
                        Details
                    </button>
                </td>
            </tr>

            {showDetailModal && (
                <OrderDetailModal
                    order={order}
                    refetchOrders={refetchOrders}
                    onClose={() => setShowDetailModal(false)}
                />
            )}
        </>
    );
};

export default OrderRow;