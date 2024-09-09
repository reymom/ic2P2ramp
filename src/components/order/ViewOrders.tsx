import { useState, useEffect } from 'react';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faArrowLeft, faArrowRight } from '@fortawesome/free-solid-svg-icons';


import { backend } from '../../declarations/backend';
import { OrderFilter, OrderState } from '../../declarations/backend/backend.did';
import OrderFilters from './OrderFilters';
import Order from './Order';

function ViewOrders({ initialFilter }: { initialFilter: OrderFilter | null }) {
    const [loading, setLoading] = useState(false);
    const [orders, setOrders] = useState<OrderState[]>([]);
    const [filter, setFilter] = useState<OrderFilter | null>(initialFilter);
    const [page, setPage] = useState(1);

    const pageSize = 5;

    useEffect(() => {
        fetchOrders();
    }, [filter, page]);

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

    const fetchOrders = async () => {
        try {
            setLoading(true);
            const orders = await backend.get_orders(filter ? [filter] : [], [page], [pageSize]);
            setOrders(orders);
        } catch (err) {
            console.error(err);
        } finally {
            setLoading(false);
        }
    };

    const handleNextPage = () => {
        setPage((prevPage) => prevPage + 1);
    };

    const handlePreviousPage = () => {
        setPage((prevPage) => (prevPage > 1 ? prevPage - 1 : 1));
    };

    const bottomPagination =
        <div className="flex justify-between items-center my-4">
            <button
                onClick={handlePreviousPage}
                disabled={page === 1}
                className={`px-4 py-2 rounded-lg text-white ${page === 1 ? 'bg-gray-400 cursor-not-allowed' : 'bg-blue-500 hover:bg-blue-600'}`}
            >
                <FontAwesomeIcon icon={faArrowLeft} />
            </button>
            <button
                onClick={handleNextPage}
                disabled={orders.length === 0}
                className={`px-4 py-2 rounded-lg text-white ${orders.length < pageSize ? 'bg-gray-400 cursor-not-allowed' : 'bg-blue-500 hover:bg-blue-600'}`}
            >
                <FontAwesomeIcon icon={faArrowRight} />
            </button>
        </div>

    return (
        <div className="container mx-auto p-4 bg-gray-700 border rounded-md text-white">
            {/* <h2 className="text-xl font-semibold mb-4">View Orders</h2> */}
            <div className="flex justify-between items-center mb-4">
                <button
                    onClick={handlePreviousPage}
                    disabled={page === 1}
                    className={`px-4 py-2 rounded-lg ${page === 1 ? 'bg-gray-400 cursor-not-allowed' : 'bg-blue-500 hover:bg-blue-600'}`}
                >
                    <FontAwesomeIcon icon={faArrowLeft} />
                </button>
                <div className="flex flex-grow mx-2">
                    <OrderFilters setFilter={setFilter} />
                </div>
                <button
                    onClick={handleNextPage}
                    disabled={orders.length === 0}
                    className={`px-4 py-2 rounded-lg ${orders.length < pageSize ? 'bg-gray-400 cursor-not-allowed' : 'bg-blue-500 hover:bg-blue-600'}`}
                >
                    <FontAwesomeIcon icon={faArrowRight} />
                </button>
            </div>

            {loading ? (
                <div className="mt-4 flex justify-center items-center space-x-2">
                    <div className="w-4 h-4 border-t-2 border-b-2 border-indigo-600 rounded-full animate-spin"></div>
                    <div className="text-sm font-medium text-gray-300">Fetching orders...</div>
                </div>
            ) : (
                <ul className="space-y-2">
                    {orders.map((order, index) => (
                        <Order key={index} order={order} refetchOrders={fetchOrders} />
                    ))}
                </ul>
            )}

            {orders.length > 3 ? bottomPagination : null}
        </div >
    );
}

export default ViewOrders;
