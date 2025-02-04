import { useState, useEffect, useRef } from 'react';
import { useSearchParams } from 'react-router-dom';

import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faArrowLeft, faArrowRight, faTh, faList } from '@fortawesome/free-solid-svg-icons';

import { backend } from '@/model/backendProxy';
import { OrderFilter, OrderState } from '@/declarations/backend/backend.did';
import { parseBigIntFields } from '@/model/mock';
import OrderFilters from '@/components/order/OrderFilters';
import Order from '@/components/order/Order';
import mockOrdersData from '@/assets/mocks/orders.json';

function ViewOrders({ initialFilter }: { initialFilter: OrderFilter | null }) {
    const [loading, setLoading] = useState(false);
    const [orders, setOrders] = useState<OrderState[]>([]);
    const [isListView, setIsListView] = useState(false);
    const [filter, setFilter] = useState<OrderFilter | null>(initialFilter);
    const ordersRef = useRef<HTMLDivElement>(null);
    const [ordersHeight, setOrdersHeight] = useState<number | null>(null);

    const [searchParams, setSearchParams] = useSearchParams();
    const initialPage = Number(searchParams.get('page') || 1);
    const [page, setPage] = useState(initialPage);
    const pageSize = 5;

    useEffect(() => {
        if (ordersRef.current) {
            setOrdersHeight(ordersRef.current.clientHeight);
        }
    }, [orders]);

    useEffect(() => {
        const offramperId = searchParams.get('offramperId');
        if (offramperId) {
            setFilter({ ByOfframperId: BigInt(offramperId) });
            return;
        }

        const onramperId = searchParams.get('onramperId');
        if (onramperId) {
            setFilter({ ByOnramperId: BigInt(onramperId) });
            return;
        }

        const status = searchParams.get('status')
        if (status) {
            setFilter({ ByState: { [status]: null } } as OrderFilter);
            return;
        }
    }, [searchParams]);

    useEffect(() => {
        fetchOrders();
    }, [filter, page]);

    const fetchOrders = async () => {
        if (import.meta.env.VITE_USE_MOCKS) {
            console.warn("Using mock orders");
            const mockOrders: OrderState[] = mockOrdersData.map(parseBigIntFields);
            setOrders(mockOrders as OrderState[]);
            return;
        }

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
        const nextPage = page + 1
        setPage(nextPage);
        setSearchParams({ page: nextPage.toString() });
    };

    const handlePreviousPage = () => {
        if (page > 1) {
            const prevPage = page - 1;
            setPage(prevPage);
            setSearchParams({ page: prevPage.toString() });
        }
    };

    return (
        <div className="w-full px-6" ref={ordersRef}>
            <div className="flex justify-between items-center gap-4 mb-6">
                {/* Filters */}
                <div className="flex flex-grow justify-between items-center">
                    <OrderFilters setFilter={setFilter} currentFilter={filter} />
                    {/* List/Grid Toggle */}
                    <div className="flex gap-2">
                        <button
                            onClick={() => setIsListView(true)}
                            className={`w-12 h-10 flex items-center justify-center rounded-md
                            ${isListView ? "bg-blue-500 dark:bg-blue-600" : "bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600"}`}
                        >
                            <FontAwesomeIcon icon={faList} size="lg" />
                        </button>
                        <button
                            onClick={() => setIsListView(false)}
                            className={`w-12 h-10 flex items-center justify-center rounded-md
                            ${!isListView ? "bg-blue-500 dark:bg-blue-600" : "bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600"}`}
                        >
                            <FontAwesomeIcon icon={faTh} size="lg" />
                        </button>
                    </div>
                </div>
            </div>

            {/* Left Pagination Button */}
            {ordersHeight && (
                <button
                    onClick={handlePreviousPage}
                    disabled={page === 1}
                    style={{
                        height: `${ordersHeight}px`, // Match orders height
                        top: `calc(${ordersHeight / 2}px)`, // Center it dynamically
                    }}
                    className={`absolute left-0 w-[50px] flex items-center justify-center
                        bg-gray-700 bg-opacity-30 hover:bg-opacity-80 transition-all rounded-r-lg
                        ${page === 1 ? 'cursor-not-allowed opacity-40' : 'hover:bg-gray-600'}
                    `}
                >
                    <FontAwesomeIcon icon={faArrowLeft} size="lg" />
                </button>
            )}

            {/* Right Pagination Button */}
            {ordersHeight && (
                <button
                    onClick={handleNextPage}
                    disabled={orders.length === 0}
                    style={{
                        height: `${ordersHeight}px`, // Match orders height
                        top: `calc(${ordersHeight / 2}px)`, // Center it dynamically
                    }}
                    className={`absolute right-0 w-[50px] flex items-center justify-center
                        bg-gray-700 bg-opacity-30 hover:bg-opacity-80 transition-all rounded-l-lg
                        ${orders.length < pageSize ? 'cursor-not-allowed opacity-40' : 'hover:bg-gray-600'}
                    `}
                >
                    <FontAwesomeIcon icon={faArrowRight} size="lg" />
                </button>
            )}

            {/* Orders List/Grid */}
            {
                loading ? (
                    <div className="flex justify-center items-center h-32">
                        <div className="w-6 h-6 border-t-2 border-b-2 border-indigo-400 rounded-full animate-spin"></div>
                    </div>
                ) : (
                    isListView ? (
                        <ul className={`${isListView ? "space-y-4" : "grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-6"}`}>
                            <table className="w-full border-collapse">
                                <thead>
                                    <tr className="bg-gray-200 dark:bg-gray-800 text-gray-800 dark:text-gray-300 border-b border-gray-300 dark:border-gray-700">
                                        <th className="px-4 py-3 text-left">Status</th>
                                        <th className="px-4 py-3 text-left">Price</th>
                                        <th className="px-4 py-3 text-left">Amount</th>
                                        <th className="px-4 py-3 text-left">Address</th>
                                        <th className="px-4 py-3 text-left">Network</th>
                                        <th className="px-4 py-3 text-left">Time Left</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {orders.map((order, index) => (
                                        <Order key={index} order={order} isListView={isListView} refetchOrders={fetchOrders} />
                                    ))}
                                </tbody>
                            </table>
                        </ul>
                    ) : (
                        <ul className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
                            {orders.map((order, index) => (
                                <Order key={index} order={order} isListView={isListView} refetchOrders={fetchOrders} />
                            ))}
                        </ul>
                    )
                )
            }
        </div>
    );
}

export default ViewOrders;
