import React from "react";

import OrderCard from "./OrderCard";
import OrderRow from "./OrderRow";
import { OrderState } from "@/declarations/icramp_backend/icramp_backend.did";

interface OrderProps {
    order: OrderState;
    isListView: boolean;
    refetchOrders: () => void;
}

const Order: React.FC<OrderProps> = ({ order, isListView, refetchOrders }) => {
    return isListView ? (
        <OrderRow order={order} refetchOrders={refetchOrders} />
    ) : (
        <OrderCard order={order} refetchOrders={refetchOrders} />
    );
};

export default Order;
