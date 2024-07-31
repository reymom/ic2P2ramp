import React, { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAccount } from 'wagmi';
import { ethers } from 'ethers';

import { backend } from '../../declarations/backend';
import { PaymentProvider, PaymentProviderType, Blockchain } from '../../declarations/backend/backend.did';
import { icP2PrampABI } from '../../constants/ic2P2ramp';
import { TokenOption, addresses, getTokenOptions } from '../../constants/addresses';
import { useUser } from '../../UserContext';
import { rampErrorToString } from '../../model/error';
import { providerToProviderType } from '../../model/utils';

const CreateOrder: React.FC = () => {
    const [fiatAmount, setFiatAmount] = useState<number>();
    const [currency, setCurrency] = useState<string>("USD");
    const [cryptoAmount, setCryptoAmount] = useState(0);
    const [selectedToken, setSelectedToken] = useState<TokenOption | null>(null);
    const [selectedBlockchain, setSelectedBlockchain] = useState<Blockchain>({ EVM: { chain_id: BigInt(1) } });
    const [message, setMessage] = useState('');
    const [isLoading, setIsLoading] = useState(false);
    const [loadingRate, setLoadingRate] = useState(false);
    const [exchangeRate, setExchangeRate] = useState<number | null>(null);
    const [selectedProviders, setSelectedProviders] = useState<PaymentProvider[]>([]);

    const { chain, chainId } = useAccount();
    const { user } = useUser();
    const navigate = useNavigate();

    const handleTokenChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
        if (!chainId) {
            navigate("/");
            return;
        }

        const tokenOption = getTokenOptions(chainId).find(token => token.address === e.target.value);
        if (tokenOption) {
            setSelectedToken(tokenOption);
        }
    };

    const getExchangeRateFromXRC = async (token: string) => {
        if (token === "") return;
        setLoadingRate(true);
        try {
            const result = await backend.get_exchange_rate(currency, token);
            console.log("[exchangeRate] result = ", result);
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
        } finally {
            setLoadingRate(false);
        }
    };

    useEffect(() => {
        if (selectedToken) {
            getExchangeRateFromXRC(selectedToken.name);
        }
    }, [selectedToken]);

    useEffect(() => {
        if (exchangeRate !== null) {
            setFiatAmount(cryptoAmount * exchangeRate);
        }
    }, [cryptoAmount, exchangeRate]);

    const handleProviderSelection = (provider: PaymentProvider) => {
        if (selectedProviders.length === 0) {
            setSelectedProviders([provider]);
            return
        }
        setSelectedProviders((prevSelected) => {
            if (prevSelected.includes(provider)) {
                return prevSelected.filter((p) => p !== provider);
            } else {
                return [...prevSelected, provider];
            }
        });
    };

    const handleBlockchainChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
        const value = e.target.value;
        if (value === "EVM") {
            setSelectedBlockchain({ EVM: { chain_id: BigInt(chainId!) } });
        } else if (value === "ICP") {
            setSelectedBlockchain({ ICP: { subnet_id: "some-subnet-id" } });
        } else if (value === "Solana") {
            setSelectedBlockchain({ Solana: null });
        }
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

            const tokenAddress = selectedToken?.address;
            if (!tokenAddress) {
                throw new Error('No token selected');
            }

            const vaultContract = new ethers.Contract(tokenAddress, icP2PrampABI, signer);

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

            const providerTuples: [PaymentProviderType, PaymentProvider][] = selectedProviders.map((provider) => {
                const providerType: PaymentProviderType = providerToProviderType(provider);
                return [providerType, provider];
            });
            const result = await backend.create_order(
                BigInt(Math.ceil(fiatAmount! * 100)),
                currency,
                providerTuples,
                selectedBlockchain,
                selectedToken.isNative ? [] : [selectedToken.address],
                cryptoAmountInWei,
                user!.evm_address,
            );
            console.log("result(backend.create_order) = ", result);

            if ('Ok' in result) {
                setMessage(`Order with ID=${result.Ok} created!`);
                navigate("/view");
            } else {
                const errorMessage = rampErrorToString(result.Err);
                setMessage(errorMessage);
            }
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
                    <label className="block text-gray-700 w-24">Fiat:</label>
                    <div className="flex-grow px-2 py-2 flex items-center">
                        <input
                            type="number"
                            value={fiatAmount?.toFixed(2)}
                            className="py-2 w-36 px-2 border rounded-l"
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
                    <label className="block text-gray-700 w-24">Crypto:</label>
                    <input
                        type="number"
                        value={cryptoAmount}
                        onChange={(e) => setCryptoAmount(Number(e.target.value))}
                        className="flex-grow px-3 py-2 border rounded"
                        required
                    />
                </div>
                <div className="flex items-center mb-4">
                    <label className="block text-gray-700 w-24">Token:</label>
                    <select
                        value={selectedToken?.address || ""}
                        onChange={handleTokenChange}
                        className="flex-grow px-3 py-2 border rounded"
                        required
                    >
                        <option value="">Select a token</option>
                        {chainId && getTokenOptions(chainId).map((token) => (
                            <option key={token.address} value={token.address}>{token.name}</option>
                        ))}
                    </select>
                </div>
                <div className="flex items-center mb-4">
                    <label className="block text-gray-700 w-24">Blockchain:</label>
                    <select
                        value={Object.keys(selectedBlockchain)[0]}
                        onChange={handleBlockchainChange}
                        className="flex-grow px-3 py-2 border rounded"
                        required
                    >
                        <option value="EVM">EVM</option>
                        <option value="ICP">ICP</option>
                        <option value="Solana">Solana</option>
                    </select>
                </div>
                {loadingRate && (
                    <div className="my-2 flex justify-center items-center space-x-2">
                        <div className="w-4 h-4 border-t-2 border-b-2 border-indigo-600 rounded-full animate-spin"></div>
                        <div className="text-sm font-medium text-gray-700">Fetching Rates...</div>
                    </div>
                )}
                <div className="mb-4 items-center text-center">
                    <label className="block text-gray-700">Payment Providers:</label>
                    {user?.payment_providers.map((provider, index) => {
                        let label = '';
                        if ('PayPal' in provider) {
                            label = `PayPal: ${provider.PayPal.id}`;
                        } else if ('Revolut' in provider) {
                            label = `Revolut: ${provider.Revolut.id}, Scheme: ${provider.Revolut.scheme}`;
                        }

                        return (
                            <div key={index} className="flex items-center mb-2">
                                <input
                                    type="checkbox"
                                    id={`provider-${index}`}
                                    className="mr-2"
                                    checked={selectedProviders!.includes(provider)}
                                    onChange={() => handleProviderSelection(provider)}
                                />
                                <label htmlFor={`provider-${index}`} className="text-gray-700 text-center">{label}</label>
                            </div>
                        );
                    })}
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
                    disabled={!chainId || !fiatAmount}
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
