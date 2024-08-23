import { useState, useEffect } from 'react';

import { backend } from '../../declarations/backend';
import { OrderFilter, OrderState } from '../../declarations/backend/backend.did';
import OrderFilters from './OrderFilters';
import Order from './Order';
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
        if (!refetch && cachedAll) return;
        try {
            setLoading(true);
            let orders: OrderState[] = [];
            if (!filter || !cachedAll) {
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
        if (filter === null) return true;

        if ("ByState" in filter) {
            return (filterStateToFilterStateType(filter.ByState)) in order;
        };

        if ("ByOfframperAddress" in filter) {
            const offramperAddress = "Created" in order ? order.Created.offramper_address :
                "Locked" in order ? order.Locked.base.offramper_address : null;

            return offramperAddress &&
                (offramperAddress.address == filter.ByOfframperAddress.address)
        };

        if ("LockedByOnramper" in filter) {
            return "Locked" in order &&
                order.Locked.onramper_address.address === filter.LockedByOnramper.address
        };

        if ("ByBlockchain" in filter) {
            const orderBlockchain = "Created" in order ? order.Created.crypto.blockchain :
                "Locked" in order ? order.Locked.base.crypto.blockchain :
                    "Completed" in order ? order.Completed.blockchain : null;

            return orderBlockchain && (
                ('EVM' in orderBlockchain && 'EVM' in filter.ByBlockchain &&
                    orderBlockchain.EVM.chain_id === filter.ByBlockchain.EVM.chain_id
                ) || ('ICP' in orderBlockchain && 'ICP' in filter.ByBlockchain &&
                    orderBlockchain.ICP.ledger_principal.toString() === filter.ByBlockchain.ICP.ledger_principal.toString()
                )
            );
        }

        if ("ByOfframperId" in filter) {
            return ("Created" in order && order.Created.offramper_user_id == filter.ByOfframperId) ||
                ("Locked" in order && order.Locked.base.offramper_user_id == filter.ByOfframperId)
        }

        if ("ByOnramperId" in filter) {
            return "Locked" in order && order.Locked.onramper_user_id == filter.ByOnramperId
        }

        return false;
    });

    return (
        <div className="container mx-auto p-4">
            <h2 className="text-xl font-semibold mb-4">View Orders</h2>
            <OrderFilters setFilter={setFilter} />
            {loading ? (
                <div className="loader" />
            ) : (
                <ul className="space-y-4">
                    {filteredOrders.map((order, index) => (
                        <Order key={index} order={order} refetchOrders={() => fetchOrders(true)} />
                    ))}
                </ul>
            )}
        </div >
    );
}

export default ViewOrders;
