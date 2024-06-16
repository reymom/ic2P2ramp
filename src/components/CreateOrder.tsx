import React, { useState, useEffect } from 'react';
import { backend } from '../declarations/backend';
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

interface CreateOrderProps {
    selectedCurrency: string;
}

const CreateOrder: React.FC<CreateOrderProps> = ({ selectedCurrency }) => {
    const [fiatAmount, setFiatAmount] = useState<number>();
    const [cryptoAmount, setCryptoAmount] = useState(0);
    const [paypalId, setPaypalId] = useState('');
    const [selectedToken, setSelectedToken] = useState('');
    const [message, setMessage] = useState('');
    const [isLoading, setIsLoading] = useState(false);
    const [exchangeRate, setExchangeRate] = useState<number | null>(null);

    const { address, chain, chainId } = useAccount();

    const getTokenOptions = () => {
        switch (chainId) {
            case NetworkIds.MANTLE_SEPOLIA:
                return Object.values(MantleSepoliaTokens);
            case NetworkIds.SEPOLIA:
                return Object.values(SepoliaTokens);
            case NetworkIds.POLYGON_ZKEVM_TESTNET:
                return Object.values(PolygonZkEvmTokens);
            case NetworkIds.OP_SEPOLIA:
                return Object.values(OptimismSepoliaTokens);
            default:
                return [];
        }
    };

    const getExchangeRateFromXRC = async (token: string) => {
        try {
            const result = await backend.get_usd_exchange_rate(selectedCurrency, token);
            console.log("result = ", result);
            if ('Ok' in result) {
                const rate = parseFloat(result.Ok);
                setExchangeRate(rate);
            } else {
                console.error(result.Err);
                setExchangeRate(null);
            }
        } catch (error) {
            console.error(error);
            setExchangeRate(null);
        }
    };

    useEffect(() => {
        if (selectedToken) {
            getExchangeRateFromXRC(selectedToken);
        }
    }, [selectedToken]);

    useEffect(() => {
        if (exchangeRate !== null) {
            setFiatAmount(cryptoAmount * exchangeRate);
        }
    }, [cryptoAmount, exchangeRate]);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        try {
            setIsLoading(true);
            setMessage('Creating offramping order...');

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

            const { native, usdt } = networkAddresses;

            const vaultContract = new ethers.Contract(native, icP2PrampABI, signer);

            const cryptoAmountInWei = ethers.parseEther(cryptoAmount.toString());
            const gasEstimate = await vaultContract.depositBaseCurrency.estimateGas(
                { value: cryptoAmountInWei }
            );

            const transactionResponse = await vaultContract.depositBaseCurrency({
                value: cryptoAmountInWei,
                gasLimit: gasEstimate
            });

            setMessage('Transaction sent, waiting for confirmation...');

            const receipt = await transactionResponse.wait();
            console.log('Transaction receipt:', receipt);

            if (receipt.status === 1) {
                setMessage('Transaction successful!');
            } else {
                setMessage('Transaction failed!');
            }

            const result = await backend.create_order(
                BigInt(Math.ceil(fiatAmount! * 100)),
                cryptoAmountInWei,
                paypalId,
                address as string,
                BigInt(chainId!),
                selectedToken
            );
            console.log("result(backend.create_order) = ", result);

            setMessage("order created successfully!");
        } catch (error) {
            setMessage(`Error creating offramp order, error = ${error}`);
        } finally {
            setIsLoading(false);
        }
    };

    return (
        <div>
            <h2 className="text-lg font-bold mb-4">Create Offramping Order</h2>
            <form onSubmit={handleSubmit}>
                <div className="flex items-center mb-4">
                    <label className="block text-gray-700 w-24">Fiat Amount:</label>
                    <input
                        type="number"
                        value={fiatAmount?.toFixed(2)}
                        className="flex-grow px-3 py-2 border rounded"
                        required
                        disabled
                    />
                </div>
                <div className="flex items-center mb-4">
                    <label className="block text-gray-700 w-24">Crypto Amount:</label>
                    <input
                        type="number"
                        value={cryptoAmount}
                        onChange={(e) => setCryptoAmount(Number(e.target.value))}
                        className="flex-grow px-3 py-2 border rounded"
                        required
                    />
                </div>
                <div className="flex items-center mb-4">
                    <label className="block text-gray-700 w-24">PayPal ID:</label>
                    <input
                        type="text"
                        value={paypalId}
                        onChange={(e) => setPaypalId(e.target.value)}
                        className="flex-grow px-3 py-2 border rounded"
                        required
                    />
                </div>
                <div className="flex items-center mb-4">
                    <label className="block text-gray-700 w-24">Token:</label>
                    <select
                        value={selectedToken}
                        onChange={(e) => setSelectedToken(e.target.value)}
                        className="flex-grow px-3 py-2 border rounded"
                        required
                    >
                        <option value="">Select a token</option>
                        {getTokenOptions().map((token) => (
                            <option key={token} value={token}>{token}</option>
                        ))}
                    </select>
                </div>
                <div className="mb-4">
                    {chainId ? (
                        <div className="text-green-500">On chain: {chain?.name}</div>
                    ) : (
                        <div className="text-red-500">Please connect to a network</div>
                    )}
                </div>
                <button
                    type="submit"
                    className={`px-4 py-2 rounded ${chainId ? 'bg-blue-500 text-white' : 'bg-gray-500 text-white cursor-not-allowed'}`}
                    disabled={!chainId}
                >
                    Create Order
                </button>
            </form>
            {isLoading ? (
                <div className="mt-4 flex justify-center items-center space-x-2">
                    <div className="w-4 h-4 border-t-2 border-b-2 border-indigo-600 rounded-full animate-spin"></div>
                    <div className="text-sm font-medium text-gray-700">Processing transaction...</div>
                </div>
            ) : (
                message && <p className="mt-4 text-sm font-medium text-gray-700">{message}</p>
            )}
        </div>
    );
}

export default CreateOrder;
