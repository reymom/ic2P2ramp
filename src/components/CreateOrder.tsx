import React, { useState, useEffect } from 'react';
import { backend } from '../declarations/backend';
import { PaymentProvider } from '../declarations/backend/backend.did';
import { useAccount } from 'wagmi';
import { ethers } from 'ethers';
import { icP2PrampABI } from '../constants/ic2P2ramp';
import addresses from '../constants/addresses';
import {
    SepoliaTokens,
    BaseSepoliaTokens,
    PolygonZkEvmTokens,
    OptimismSepoliaTokens,
    NetworkIds
} from '../tokens';
import { useUser } from '../UserContext';
import { useNavigate } from 'react-router-dom';

const CreateOrder: React.FC = () => {
    const [fiatAmount, setFiatAmount] = useState<number>();
    const [currency, setCurrency] = useState<string>("USD");
    const [cryptoAmount, setCryptoAmount] = useState(0);
    const [selectedToken, setSelectedToken] = useState<[] | [string]>([]);
    const [message, setMessage] = useState('');
    const [isLoading, setIsLoading] = useState(false);
    const [exchangeRate, setExchangeRate] = useState<number | null>(null);
    const [paymentProviders, setPaymentProviders] = useState<PaymentProvider[]>([]);
    const [selectedProviders, setSelectedProviders] = useState<PaymentProvider[]>([]);

    const { chain, chainId } = useAccount();
    const { user } = useUser();
    const navigate = useNavigate();

    useEffect(() => {
        const fetchPaymentProviders = async () => {
            if (user) {
                setPaymentProviders(user.payment_providers);
            } else {
                navigate("/view");
            }
        };
        fetchPaymentProviders();
    }, [user]);

    const getTokenOptions = () => {
        switch (chainId) {
            case NetworkIds.SEPOLIA:
                return Object.values(SepoliaTokens);
            case NetworkIds.BASE_SEPOLIA:
                return Object.values(BaseSepoliaTokens);
            case NetworkIds.POLYGON_ZKEVM_TESTNET:
                return Object.values(PolygonZkEvmTokens);
            case NetworkIds.OP_SEPOLIA:
                return Object.values(OptimismSepoliaTokens);
            default:
                return [];
        }
    };

    const getExchangeRateFromXRC = async (token: string) => {
        if (token === "") return;
        try {
            console.log("currency = ", currency);
            console.log("token = ", token);

            const result = await backend.get_usd_exchange_rate(currency, token);
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
        if (selectedToken.length > 0) {
            getExchangeRateFromXRC(selectedToken[0]!);
        }
    }, [selectedToken]);

    useEffect(() => {
        if (exchangeRate !== null) {
            setFiatAmount(cryptoAmount * exchangeRate);
        }
    }, [cryptoAmount, exchangeRate]);

    const handleProviderSelection = (provider: PaymentProvider) => {
        setSelectedProviders((prevSelected) => {
            if (prevSelected.includes(provider)) {
                return prevSelected.filter((p) => p !== provider);
            } else {
                return [...prevSelected, provider];
            }
        });
    };

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

            const { native } = networkAddresses;

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
                return;
            }

            const result = await backend.create_order(
                BigInt(Math.ceil(fiatAmount! * 100)),
                currency,
                cryptoAmountInWei,
                selectedProviders,
                user!.evm_address,
                BigInt(chainId!),
                selectedToken
            );
            console.log("result(backend.create_order) = ", result);

            setMessage("order created successfully!");
            navigate("/view");
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
                    <div className="flex-grow px-2 py-2 flex items-center">
                        <input
                            type="number"
                            value={fiatAmount?.toFixed(2)}
                            className="py-2 px-2 border rounded-l"
                            required
                            disabled
                        />
                        <select
                            value={currency}
                            onChange={(e) => setCurrency(e.target.value)}
                            className="py-2 border rounded-r"
                        >
                            <option value="USD">$</option>
                            <option value="EUR">€</option>
                            <option value="GBP">£</option>
                            <option value="CHF">₣</option>
                            <option value="CZK">Kč</option>
                            <option value="AUD">AU$</option>
                            <option value="JPY">¥</option>
                            <option value="SGD">S$</option>
                        </select>
                    </div>
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
                <div className="mb-4 items-center text-center">
                    <label className="block text-gray-700">Payment Providers:</label>
                    {paymentProviders.map((provider, index) => (
                        <div key={index} className="flex items-center mb-2">
                            <input
                                type="checkbox"
                                id={`provider-${index}`}
                                className="mr-2"
                                checked={selectedProviders.includes(provider)}
                                onChange={() => handleProviderSelection(provider)}
                            />
                            <label htmlFor={`provider-${index}`} className="text-gray-700">{Object.keys(provider)[0]}</label>
                        </div>
                    ))}
                </div>
                <div className="flex items-center mb-4">
                    <label className="block text-gray-700 w-24">Token:</label>
                    <select
                        value={selectedToken}
                        onChange={(e) => setSelectedToken([e.target.value])}
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
