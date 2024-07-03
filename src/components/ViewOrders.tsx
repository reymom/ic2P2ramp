import { useState, useEffect } from 'react';
import { backend } from '../declarations/backend';
import { Order, OrderFilter, OrderState, PaymentProvider } from '../declarations/backend/backend.did';
import OrderFilters from './order/OrderFilters';
import OrderActions from './order/Order';
import { useAccount } from 'wagmi';
import { ethers } from 'ethers';
import { icP2PrampABI } from '../constants/ic2P2ramp';
import addresses from '../constants/addresses';
import { UserTypes } from '../model/types';
import { filterStateToFilterStateType } from '../model/utils';

function ViewOrders({ userType, initialFilter }: { userType: UserTypes, initialFilter: OrderFilter | null }) {
    const [orders, setOrders] = useState<OrderState[]>([]);
    const [cachedAll, setCachedAll] = useState(false);
    const [loading, setLoading] = useState(false);
    const [isLoading, setIsLoading] = useState(false);
    const [message, setMessage] = useState('');
    const [filter, setFilter] = useState<OrderFilter | null>(initialFilter);

    const { address, chainId } = useAccount();

    useEffect(() => {
        fetchOrders();
    }, [filter]);

    const fetchOrders = async () => {
        try {
            setLoading(true);
            let orders: OrderState[] = [];
            if (!filter && !cachedAll) {
                orders = await backend.get_orders([]);
                setCachedAll(true);
            } else {
                orders = await backend.get_orders([filter!]);
            }
            setOrders(orders);
        } catch (err) {
            console.error(err);
        } finally {
            setLoading(false);
        }
    };

    const commitToOrder = async (orderId: bigint, provider: PaymentProvider) => {
        try {
            setIsLoading(true);
            setMessage(`Commiting to loan order ${orderId}...`);

            await backend.lock_order(orderId, provider, address as string, [100000]);
            await fetchOrders();
        } catch (err) {
            console.error(err);
            setMessage(`Error commiting to order ${orderId}.`);
        } finally {
            setIsLoading(false);
        }
    };

    const removeOrder = async (order: Order) => {
        console.log("order = ", order);
        try {
            setIsLoading(true);
            setMessage(`Removing order ${order.id}...`);

            if (!window.ethereum) {
                throw new Error('No crypto wallet found. Please install it.');
            }

            const provider = new ethers.BrowserProvider(window.ethereum);
            await provider.send('eth_requestAccounts', []);
            const signer = await provider.getSigner();

            const networkAddresses = addresses[chainId!];
            if (!networkAddresses) {
                throw new Error('Unsupported network');
            }

            const { native } = networkAddresses;
            const vaultContract = new ethers.Contract(native, icP2PrampABI, signer);

            const gasEstimate = await vaultContract.uncommitDeposit.estimateGas(order.offramper_address, ethers.ZeroAddress, order.crypto_amount);
            const transactionResponse = await vaultContract.uncommitDeposit(order.offramper_address, ethers.ZeroAddress, order.crypto_amount, {
                gasLimit: gasEstimate
            });

            setMessage('Transaction sent, waiting for confirmation...');
            const receipt = await transactionResponse.wait();
            console.log('Transaction receipt:', receipt);

            if (receipt.status === 1) {
                setMessage('Transaction successful!');
            } else {
                setMessage('ICP Transaction failed!');
                return;
            }

            await backend.cancel_order(order.id);
            await fetchOrders();
        } catch (err) {
            console.error(err);
            setMessage(`Error removing order ${order.id}.`);
        } finally {
            setIsLoading(false);
        }
    };

    const handlePayPalSuccess = async (transactionId: string, orderId: bigint) => {
        try {
            setMessage(`Payment successful for order ${orderId}, transaction ID: ${transactionId}`);
            // Send transaction ID to backend to verify payment
            const response = await backend.verify_transaction(
                orderId,
                transactionId,
                [100000]
            );
            console.log('Backend response:', response);

            fetchOrders();
        } catch (err) {
            console.error(err);
            setMessage(`Error verifying payment for order ${orderId}.`);
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
        }

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
            <OrderFilters setFilter={setFilter} userType={userType} />
            {loading ? (
                <div className="loader" />
            ) : (
                <ul className="space-y-4">
                    {filteredOrders.map((order, index) => (
                        <OrderActions
                            key={index}
                            order={order}
                            userType={userType}
                            address={address as string}
                            chainId={chainId as number}
                            commitToOrder={commitToOrder}
                            removeOrder={removeOrder}
                            handlePayPalSuccess={handlePayPalSuccess}
                        />
                    ))}
                </ul>
            )}
            {isLoading ? (
                <div className="mt-4 flex justify-center items-center space-x-2">
                    <div className="w-4 h-4 border-t-2 border-b-2 border-indigo-600 rounded-full animate-spin"></div>
                    <div className="text-sm font-medium text-gray-700">Processing transaction...</div>
                </div>
            ) : (
                message && <p className="mt-4 text-sm font-medium text-gray-700">{message}</p>
            )}
        </div >
    );
}

export default ViewOrders;
