import { useState, useEffect } from 'react';
import { ethers } from 'ethers';
import { useAccount } from 'wagmi';

import { backend } from '../declarations/backend';
import { Order, OrderFilter, OrderState, PaymentProvider } from '../declarations/backend/backend.did';
import OrderFilters from './order/OrderFilters';
import OrderActions from './order/Order';
import { addresses } from '../constants/addresses';
import { icP2PrampABI } from '../constants/ic2P2ramp';
import { filterStateToFilterStateType } from '../model/utils';

function ViewOrders({ initialFilter }: { initialFilter: OrderFilter | null }) {
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
        setIsLoading(true);
        setMessage(`Commiting to loan order ${orderId}...`);

        try {
            const result = await backend.lock_order(orderId, provider, address!.toString(), [100000]);

            if ('Ok' in result) {
                setMessage(`Order Locked! tx = ${result.Ok}`);
            } else {
                let errorMessage = 'An unknown error occurred';

                for (const [key, value] of Object.entries(result.Err)) {
                    if (value !== null) {
                        errorMessage = `Error: ${key} - ${JSON.stringify(value)}`;
                        break;
                    } else {
                        errorMessage = `Error: ${key}`;
                        break;
                    }
                }

                setMessage(errorMessage);
            }
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

            const tokenAddress = order.token_address[0] ?? addresses[Number(order.chain_id)].native[1];
            const vaultContract = new ethers.Contract(tokenAddress, icP2PrampABI, signer);

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

            const result = await backend.cancel_order(order.id);
            if ('Ok' in result) {
                setMessage("Order Cancelled");
                await fetchOrders();
            } else {
                let errorMessage = 'An unknown error occurred';

                for (const [key, value] of Object.entries(result.Err)) {
                    if (value !== null) {
                        errorMessage = `Error: ${key} - ${JSON.stringify(value)}`;
                        break;
                    } else {
                        errorMessage = `Error: ${key}`;
                        break;
                    }
                }

                setMessage(errorMessage);
            }
        } catch (err) {
            console.error(err);
            setMessage(`Error removing order ${order.id}.`);
        } finally {
            setIsLoading(false);
        }
    };

    const handlePayPalSuccess = async (transactionId: string, orderId: bigint) => {
        setIsLoading(true);
        setMessage(`Payment successful for order ${orderId}, transaction ID: ${transactionId}. Verifying...`);
        try {
            // Send transaction ID to backend to verify payment
            const response = await backend.verify_transaction(
                orderId,
                transactionId,
                [100000]
            );
            console.log('Backend response:', response);

            if ('Ok' in response) {
                setMessage(`Order Verified and Funds Transferred successfully!`);
            } else {
                let errorMessage = 'An unknown error occurred';

                for (const [key, value] of Object.entries(response.Err)) {
                    if (value !== null) {
                        errorMessage = `Error: ${key} - ${JSON.stringify(value)}`;
                        break;
                    } else {
                        errorMessage = `Error: ${key}`;
                        break;
                    }
                }

                setMessage(errorMessage);
            }
        } catch (err) {
            console.error(err);
            setMessage(`Error verifying payment for order ${orderId}.`);
        } finally {
            setIsLoading(false);
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
            <OrderFilters setFilter={setFilter} />
            {loading ? (
                <div className="loader" />
            ) : (
                <ul className="space-y-4">
                    {filteredOrders.map((order, index) => (
                        <OrderActions
                            key={index}
                            order={order}
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
