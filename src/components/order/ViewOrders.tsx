import { useState, useEffect } from 'react';

import { backend } from '../../declarations/backend';
import { OrderFilter, OrderState } from '../../declarations/backend/backend.did';
import OrderFilters from './OrderFilters';
import OrderActions from './Order';
import { filterStateToFilterStateType } from '../../model/utils';

function ViewOrders({ initialFilter }: { initialFilter: OrderFilter | null }) {
    const [orders, setOrders] = useState<OrderState[]>([]);
    const [cachedAll, setCachedAll] = useState(false);
    const [loading, setLoading] = useState(false);
    const [filter, setFilter] = useState<OrderFilter | null>(initialFilter);

    useEffect(() => {
        fetchOrders(false);
    }, [filter]);

    useEffect(() => {
        const clientId = process.env.FRONTEND_PAYPAL_CLIENT_ID;
        console.log("clientId = ", clientId);
        if (!clientId) return;

        // Load PayPal SDK script only once
        const scriptId = 'paypal-sdk';
        if (!document.getElementById(scriptId)) {
            const script = document.createElement('script');
            script.id = scriptId;
            script.src = `https://www.paypal.com/sdk/js?client-id=${clientId}&currency=USD`;
            script.async = true;
            script.onload = () => {
                console.log('PayPal SDK script loaded');
            };
            script.onerror = () => {
                console.error('Failed to load PayPal SDK script');
            };
            document.head.appendChild(script);
        }
    }, []);

    const fetchOrders = async (refetch: Boolean) => {
        try {
            setLoading(true);
            let orders: OrderState[] = [];
            if ((!filter && !cachedAll) || refetch) {
                orders = await backend.get_orders([]);
                setCachedAll(true);
            } else if (filter) {
                orders = await backend.get_orders([filter!]);
            }
            setOrders(orders);
        } catch (err) {
            console.error(err);
        } finally {
            setLoading(false);
        }
    };

    const filteredOrders = orders.filter(order => {
        if (!filter) return true;

        if ("ByState" in filter) {
            return (filterStateToFilterStateType(filter.ByState)) in order;
        };

        if ("ByOfframperAddress" in filter) {
            return ("Created" in order && order.Created.offramper_address == filter.ByOfframperAddress) ||
                ("Locked" in order && order.Locked.base.offramper_address == filter.ByOfframperAddress)
        };

        if ("LockedByOnramper" in filter) {
            return "Locked" in order && order.Locked.onramper_address === filter.LockedByOnramper
        };

        if ("ByChainId" in filter) {
            if ("Created" in order) return order.Created.chain_id === filter.ByChainId;
            if ("Locked" in order) return order.Locked.base.chain_id === filter.ByChainId;
            if ("Completed" in order) return order.Completed.chain_id === filter.ByChainId;
        }

        return false;
    });

    return (
        <div>
            <h2 className="text-lg font-bold mb-4">View Orders</h2>
            <OrderFilters setFilter={setFilter} />
            {loading ? (
                <div className="loader" />
            ) : (
                <ul className="space-y-4">
                    {filteredOrders.map((order, index) => (
                        <OrderActions key={index} order={order} refetchOrders={() => fetchOrders(true)} />
                    ))}
                </ul>
            )}
        </div >
    );
}

export default ViewOrders;
