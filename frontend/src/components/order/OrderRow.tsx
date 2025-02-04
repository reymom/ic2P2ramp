import React from "react";

import { OrderState } from "@/declarations/backend/backend.did";
import { formatPrice, truncate, formatTimeLeft } from "@/utils/helper";
import { getNetworkLogo, getNetworkName } from "@/utils/network";

interface OrderRowProps {
    order: OrderState;
    refetchOrders: () => void;
}

const OrderRow: React.FC<OrderRowProps> = ({ order, refetchOrders }) => {
    let price, amount, address, networkLogo, networkName, state, timeLeft;
    let orderBlockchain = "Created" in order ? order.Created.crypto.blockchain
        : "Locked" in order ? order.Locked.base.crypto.blockchain
            : "Completed" in order ? order.Completed.blockchain
                : null;


    if ("Created" in order) {
        state = "Created";
        price = formatPrice(Number(order.Created.currency));
        amount = order.Created.crypto.amount.toString();
        address = truncate(order.Created.offramper_address.address, 8, 8);
    } else if ("Locked" in order) {
        state = "Locked";
        price = formatPrice(Number(order.Locked.price + order.Locked.offramper_fee));
        amount = order.Locked.base.crypto.amount.toString();
        address = truncate(order.Locked.base.offramper_address.address, 8, 8);
        timeLeft = formatTimeLeft(Number(order.Locked.locked_at));
    } else if ("Completed" in order) {
        state = "Completed";
        price = formatPrice(Number(order.Completed.price));
        address = truncate(order.Completed.offramper.address, 8, 8);
    } else if ("Cancelled" in order) {
        state = "Cancelled";
        price = "-";
        amount = "-";
        address = "-";
    }

    networkLogo = getNetworkLogo(orderBlockchain);
    networkName = getNetworkName(orderBlockchain);

    return (
        <tr className="border-b border-gray-300 dark:border-gray-700 text-gray-900 dark:text-gray-300 bg-white dark:bg-gray-900 hover:bg-gray-100 dark:hover:bg-gray-800 transition">
            <td className="px-4 py-3">{state}</td>
            <td className="px-4 py-3">{price}</td>
            <td className="px-4 py-3">{amount}</td>
            <td className="px-4 py-3">{address}</td>
            <td className="px-4 py-3 flex items-center">
                {networkLogo && <img src={networkLogo} alt={networkName} className="h-5 w-5" />}
            </td>
            {timeLeft && <td className="px-4 py-3">{timeLeft}</td>}
        </tr>
    );
};

export default OrderRow;
