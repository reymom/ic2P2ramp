import { useState, useEffect, useRef, useCallback, useMemo } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import clsx from 'clsx';

import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faArrowLeft, faArrowRight, faTh, faList } from '@fortawesome/free-solid-svg-icons';

import { backend } from '@/model/backendProxy';
import { OrderFilter, OrderState } from '@/declarations/icramp_backend/icramp_backend.did';
import { parseBigIntFields } from '@/model/helpers/mock';
import OrderFilters from '@/components/order/OrderFilters';
import Order from '@/components/order/Order';
import { useUser } from '@/components/user/UserContext';
import mockOrdersData from '@/assets/mocks/orders.json';

const stableKey = (v: OrderFilter | null) =>
    v == null ? 'null' : JSON.stringify(v, (_k, val) => (typeof val === 'bigint' ? `__bigint__${val}` : val));

function ViewOrders({ initialFilter }: { initialFilter: OrderFilter | null }) {
    const [loading, setLoading] = useState(false);
    const [orders, setOrders] = useState<OrderState[]>([]);
    const [isListView, setIsListView] = useState(false);
    const [filter, _setFilter] = useState<OrderFilter | null>(initialFilter);

    const setFilterSafe = useCallback((next: OrderFilter | null) => {
        _setFilter(prev => (stableKey(prev) === stableKey(next) ? prev : next));
    }, []);
    const filterKey = useMemo(() => stableKey(filter), [filter]);

    const ordersRef = useRef<HTMLDivElement>(null);
    const [ordersHeight, setOrdersHeight] = useState<number | null>(null);

    const [searchParams, setSearchParams] = useSearchParams();
    const initialPage = Number(searchParams.get('page') || 1);
    const [page, setPage] = useState(initialPage);
    const pageSize = 5;

    const { userType } = useUser();
    const navigate = useNavigate();

    useEffect(() => {
        const updateHeight = () => {
            if (ordersRef.current) setOrdersHeight(ordersRef.current.clientHeight);
        };
        updateHeight();
        window.addEventListener("resize", updateHeight);
        return () => {
            window.removeEventListener("resize", updateHeight);
        };
    }, [orders]);

    useEffect(() => {
        const offramperId = searchParams.get('offramperId');
        if (offramperId) { setFilterSafe({ ByOfframperId: BigInt(offramperId) }); return; }
        const onramperId = searchParams.get('onramperId');
        if (onramperId) { setFilterSafe({ ByOnramperId: BigInt(onramperId) }); return; }
        const s = searchParams.get('status') as 'Created' | 'Locked' | 'Completed' | 'Cancelled' | null;
        if (s) { setFilterSafe({ ByState: { [s]: null } } as OrderFilter); return; }
        setFilterSafe(null);
    }, [searchParams, setFilterSafe]);

    const normalizeFilter = (f: OrderFilter | null): OrderFilter | null => {
        if (!f) return f;
        const v = f as any;
        // If someone accidentally sent a bare BlockchainType ({Bitcoin:null}), wrap it.
        if (v && (v.EVM || v.ICP || v.Bitcoin || v.Solana)) {
            return { ByBlockchain: v } as OrderFilter;
        }
        return f;
    };
    const reqRef = useRef(0);
    useEffect(() => {
        const run = async () => {
            const myReq = ++reqRef.current;
            try {
                setLoading(true);
                console.log("loading with filter = ", filter);
                const f = normalizeFilter(filter);
                const res = await backend.get_orders(f ? [f] : [], [page], [pageSize]);
                if (reqRef.current !== myReq) return;
                setOrders(res);
            } catch (err) {
                if (reqRef.current !== myReq) return;
                console.error(err);
            } finally {
                if (reqRef.current === myReq) setLoading(false);
            }
        };
        run();
    }, [filterKey, page]);

    const fetchOrders = async () => {
        if (process.env.FRONTEND_USE_MOCKS === "true") {
            console.warn("Using mock orders");
            const mockOrders: OrderState[] = mockOrdersData.map(parseBigIntFields);
            setOrders(mockOrders as OrderState[]);
            return;
        }

        try {
            setLoading(true);
            console.log("loading with filter = ", filter);
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
        <div className="relative w-full" >
            <div className="flex justify-between items-center gap-4 mb-6">
                {/* Filters */}
                <div className="flex flex-grow justify-between items-center">
                    <OrderFilters setFilter={setFilterSafe} currentFilter={filter} />
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
            {!!ordersHeight && orders.length > 0 && (
                <button
                    onClick={handlePreviousPage}
                    disabled={page === 1}
                    className={clsx(
                        "absolute w-[40px] flex items-center justify-center",
                        "bg-gray-200 dark:bg-gray-800 transition-all rounded-l-lg",
                        page === 1 ? 'cursor-not-allowed' : 'hover:bg-gray-300 dark:hover:bg-gray-700'
                    )}
                    style={{
                        height: `${ordersHeight}px`,
                        marginLeft: "-50px",
                    }}
                >
                    <FontAwesomeIcon icon={faArrowLeft} size="lg" />
                </button>
            )}

            {/* Right Pagination Button */}
            {ordersHeight && orders.length > 0 && (
                <button
                    onClick={handleNextPage}
                    disabled={orders.length === 0}
                    className={clsx(
                        "absolute right-0 w-[40px] flex items-center justify-center",
                        "bg-gray-200 dark:bg-gray-800 transition-all rounded-r-lg",
                        orders.length < pageSize ? 'cursor-not-allowed' : 'hover:bg-gray-300 dark:hover:bg-gray-700'
                    )}
                    style={{
                        height: `${ordersHeight}px`,
                        marginRight: "-50px",
                    }}
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
                ) : orders.length === 0 ? (

                    <div className="flex flex-col items-center justify-center h-64 text-center">
                        <div className="text-gray-400 dark:text-gray-500 mb-3">
                            <svg className="w-16 h-16 mx-auto" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
                            </svg>
                        </div>
                        <h3 className="text-lg font-medium text-gray-600 dark:text-gray-400 mb-1">
                            No orders found
                        </h3>
                        <p className="text-sm text-gray-500 dark:text-gray-500">
                            There are no orders matching your current filters.
                        </p>
                        {userType === "Offramper" && (
                            <button
                                onClick={() => navigate('/create')}
                                className="mt-4 px-4 py-2 bg-blue-500 text-white rounded-md hover:bg-blue-600 dark:bg-blue-700 dark:hover:bg-blue-800 transition-colors"
                            >
                                Create Order
                            </button>
                        )}
                    </div>

                ) : isListView ? (
                    <div className="overflow-x-auto">
                        <table className="w-full border-collapse">
                            <thead>
                                <tr className="bg-gray-200 dark:bg-gray-800 text-gray-800 dark:text-gray-300 border-b border-gray-300 dark:border-gray-700">
                                    <th className="px-4 py-3 text-left">Status</th>
                                    <th className="px-4 py-3 text-left">Price</th>
                                    <th className="px-4 py-3 text-left">Amount</th>
                                    <th className="px-4 py-3 text-left">Address</th>
                                    <th className="px-4 py-3 text-left">Network</th>
                                    <th className="px-4 py-3 text-left">Time Left</th>
                                    <th className="px-4 py-3 text-left">Actions</th>
                                </tr>
                            </thead>
                            <tbody>
                                {orders.map((order, index) => (
                                    <Order key={index} order={order} isListView={isListView} refetchOrders={fetchOrders} />
                                ))}
                            </tbody>
                        </table>
                    </div>
                ) : (
                    <div ref={ordersRef}>
                        <ul className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
                            {orders.map((order, index) => (
                                <Order key={index} order={order} isListView={isListView} refetchOrders={fetchOrders} />
                            ))}
                        </ul>
                    </div>
                )
            }
        </div>
    );
}

export default ViewOrders;
