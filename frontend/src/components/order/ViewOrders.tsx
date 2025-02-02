import { useState, useEffect } from 'react';
import { useSearchParams } from 'react-router-dom';

import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faArrowLeft, faArrowRight } from '@fortawesome/free-solid-svg-icons';

import { backend } from '../../model/backendProxy';
import { OrderFilter, OrderState } from '../../declarations/backend/backend.did';
import OrderFilters from './OrderFilters';
import Order from './Order';
import mockOrdersData from '../../assets/mocks/orders.json';
import { parseBigIntFields } from '../../model/mock';


function ViewOrders({ initialFilter }: { initialFilter: OrderFilter | null }) {
    const [loading, setLoading] = useState(false);
    const [orders, setOrders] = useState<OrderState[]>([]);
    const [filter, setFilter] = useState<OrderFilter | null>(initialFilter);

    const [searchParams, setSearchParams] = useSearchParams();

    const initialPage = Number(searchParams.get('page') || 1);
    const [page, setPage] = useState(initialPage);

    const pageSize = 5;

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

    const bottomPagination =
        <div className="flex justify-between items-center my-4">
            <button
                onClick={handlePreviousPage}
                disabled={page === 1}
                className={`px-4 py-2 rounded-lg text-white ${page === 1 ? 'bg-gray-400 cursor-not-allowed' : 'bg-blue-600 hover:bg-blue-700'}`}
            >
                <FontAwesomeIcon icon={faArrowLeft} />
            </button>
            <button
                onClick={handleNextPage}
                disabled={orders.length === 0}
                className={`px-4 py-2 rounded-lg text-white ${orders.length < pageSize ? 'bg-gray-400 cursor-not-allowed' : 'bg-blue-600 hover:bg-blue-700'}`}
            >
                <FontAwesomeIcon icon={faArrowRight} />
            </button>
        </div>

    return (
        <div className="container mx-auto max-w-7xl p-6">
            <div className="flex justify-between items-center mb-6 bg-gray-800 bg-opacity-70 shadow-xl p-4 rounded-lg">
                <button
                    onClick={handlePreviousPage}
                    disabled={page === 1}
                    className={`px-4 py-2 rounded-lg ${page === 1 ? 'bg-gray-500 cursor-not-allowed' : 'bg-blue-600 hover:bg-blue-700'}`}
                >
                    <FontAwesomeIcon icon={faArrowLeft} />
                </button>
                <div className="flex flex-grow mx-4">
                    <OrderFilters setFilter={setFilter} currentFilter={filter} />
                </div>
                <button
                    onClick={handleNextPage}
                    disabled={orders.length === 0}
                    className={`px-4 py-2 rounded-lg ${orders.length < pageSize ? 'bg-gray-500 cursor-not-allowed' : 'bg-blue-600 hover:bg-blue-700'}`}
                >
                    <FontAwesomeIcon icon={faArrowRight} />
                </button>
            </div>

            {loading ? (
                <div className="flex justify-center items-center h-32">
                    <div className="w-6 h-6 border-t-2 border-b-2 border-indigo-400 rounded-full animate-spin"></div>
                    {/* <div className="text-sm font-medium text-gray-300">Fetching orders...</div> */}
                </div>
            ) : (
                <ul className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
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
