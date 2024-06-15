import { useState, useEffect } from 'react';
import { backend } from '../declarations/backend';
import { Order } from '../declarations/backend/backend.did';
import PayPalButton from './PaypalButton';
import { useAccount } from 'wagmi';
import { ethers } from 'ethers';
import { icP2PrampABI } from '../constants/ic2P2ramp';
import addresses from '../constants/addresses';
import {
    MantleSepoliaTokens,
    SepoliaTokens,
    PolygonZkEvmTokens,
    OptimismSepoliaTokens,
    NetworkIds
} from '../tokens';

function ViewOrders() {
    const [orders, setOrders] = useState<Order[]>([]);
    const [paypalId, setPaypalId] = useState('');
    const [loading, setLoading] = useState(false);
    const [isLoading, setIsLoading] = useState(false);
    const [message, setMessage] = useState('');
    const [filter, setFilter] = useState('all');

    const { address, chainId } = useAccount();

    useEffect(() => {
        fetchOrders();
    }, []);

    const fetchOrders = async () => {
        try {
            setLoading(true);
            const orders = await backend.get_orders();
            setOrders(orders);
        } catch (err) {
            console.error(err);
        } finally {
            setLoading(false);
        }
    };

    const getNetworkName = (chainId: number) => {
        switch (chainId) {
            case NetworkIds.MANTLE_SEPOLIA:
                return 'Mantle Sepolia';
            case NetworkIds.SEPOLIA:
                return 'Sepolia';
            case NetworkIds.OP_SEPOLIA:
                return 'Optimism Sepolia';
            case NetworkIds.POLYGON_ZKEVM_TESTNET:
                return 'Polygon zkEVM Testnet';
            default:
                return 'Unknown Network';
        }
    };

    const commitToOrder = async (order: Order) => {
        try {
            console.log("order = ", order)
            setIsLoading(true);
            setMessage(`Commiting to loan order ${order.id}...`);

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

            const gasEstimate = await vaultContract.commitDeposit.estimateGas(order.offramper_address, ethers.ZeroAddress, order.crypto_amount);
            const transactionResponse = await vaultContract.commitDeposit(order.offramper_address, ethers.ZeroAddress, order.crypto_amount, {
                gasLimit: gasEstimate
            });

            setMessage('Transaction sent, waiting for confirmation...');

            const receipt = await transactionResponse.wait();
            console.log('Transaction receipt:', receipt);

            if (receipt.status === 1) {
                setMessage('Transaction successful!');
                console.log("writing backend lock order")
            } else {
                setMessage('ICP Transaction failed!');
            }

            await backend.lock_order(order.id, paypalId, address as string);
            await fetchOrders();
        } catch (err) {
            console.error(err);
            setMessage(`Error commiting to order ${order.id}.`);
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

            console.log("hola?")
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
                // Remove the order in the backend
            } else {
                setMessage('ICP Transaction failed!');
            }

            await backend.remove_order(order.id);
            await fetchOrders();
        } catch (err) {
            console.error(err);
            setMessage(`Error removing order ${order.id}.`);
        } finally {
            setIsLoading(false);
        }
    };

    const handlePayPalSuccess = async (transactionId: string, orderId: string) => {
        try {
            setMessage(`Payment successful for order ${orderId}, transaction ID: ${transactionId}`);
            // Send transaction ID to backend to verify payment
            const response = await backend.verify_transaction(
                orderId,
                transactionId,
            );
            console.log('Backend response:', response);

            fetchOrders();
        } catch (err) {
            console.error(err);
            setMessage(`Error verifying payment for order ${orderId}.`);
        }
    };

    const truncate = (str: string, frontChars: number, backChars: number) => {
        if (str.length <= frontChars + backChars) {
            return str;
        }
        return str.slice(0, frontChars) + '...' + str.slice(-backChars);
    };

    const filteredOrders = orders.filter(order => {
        if (filter === 'all') return !order.removed;
        if (filter === 'locked') return order.locked && !order.removed;
        if (filter === 'unlocked') return !order.locked && !order.removed;
        return false;
    });

    const getTokenSymbol = (tokenType: string, chainId: number): string => {
        switch (chainId) {
            case NetworkIds.MANTLE_SEPOLIA:
                return MantleSepoliaTokens[tokenType as keyof typeof MantleSepoliaTokens] || tokenType;
            case NetworkIds.SEPOLIA:
                return SepoliaTokens[tokenType as keyof typeof SepoliaTokens] || tokenType;
            case NetworkIds.POLYGON_ZKEVM_TESTNET:
                return PolygonZkEvmTokens[tokenType as keyof typeof PolygonZkEvmTokens] || tokenType;
            case NetworkIds.OP_SEPOLIA:
                return OptimismSepoliaTokens[tokenType as keyof typeof OptimismSepoliaTokens] || tokenType;
            default:
                return tokenType;
        }
    };

    return (
        <div>
            <h2 className="text-lg font-bold mb-4">View Orders</h2>
            <div className="mb-4">
                <label className="block text-gray-700">Filter:</label>
                <select
                    value={filter}
                    onChange={(e) => setFilter(e.target.value)}
                    className="px-3 py-2 border rounded"
                >
                    <option value="all">All</option>
                    <option value="locked">Locked</option>
                    <option value="unlocked">Unlocked</option>
                </select>
            </div>
            {loading ? (
                <div className="loader" />
            ) : (
                <ul className="space-y-4">
                    {filteredOrders.map((order) => (
                        <li key={order.id} className="p-4 border rounded shadow-md bg-white">
                            <div className="flex flex-col space-y-2">
                                <div><strong>ID:</strong> {order.id}</div>
                                <div><strong>Fiat Amount:</strong> {order.fiat_amount.toString()}</div>
                                <div>
                                    <strong>Crypto Amount:</strong> {ethers.formatEther(order.crypto_amount.toString())} {getTokenSymbol(order.token_type, Number(order.chain_id))}
                                </div>
                                <div><strong>PayPal ID:</strong> {truncate(order.offramper_paypal_id, 6, 6)}</div>
                                <div><strong>Offramper Address:</strong> {truncate(order.offramper_address, 6, 6)}</div>
                                <div><strong>Network:</strong> {getNetworkName(Number(order.chain_id))}</div>
                                <div><strong>Token:</strong> {order.token_type}</div>
                                <div><strong>Locked:</strong> {order.locked ? 'Yes' : 'No'}</div>
                                <div><strong>Payment Done:</strong> {order.payment_done ? 'Yes' : 'No'}</div>
                            </div>
                            <div className="flex items-center mb-4 mt-4">
                                <label className="block text-gray-700 w-36">Your PayPal ID:</label>
                                <input
                                    type="text"
                                    value={paypalId}
                                    onChange={(e) => setPaypalId(e.target.value)}
                                    className="flex-grow px-3 py-2 border rounded"
                                    required
                                />
                            </div>
                            {order.locked && !order.payment_done ? (
                                <PayPalButton
                                    amount={order.fiat_amount}
                                    paypalId={paypalId}
                                    onSuccess={(transactionId) => handlePayPalSuccess(transactionId, order.id)}
                                    currency="USD"
                                />
                            ) : (
                                <>
                                    <button
                                        onClick={() => commitToOrder(order)}
                                        className="mt-2 px-4 py-2 bg-green-500 text-white rounded"
                                        disabled={order.locked}
                                    >
                                        Commit
                                    </button>
                                    <button
                                        onClick={() => removeOrder(order)}
                                        className="mt-2 px-4 py-2 bg-red-500 text-white rounded"
                                    >
                                        Remove
                                    </button>
                                </>
                            )
                            }
                        </li>
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
