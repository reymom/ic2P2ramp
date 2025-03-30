import React, { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAccount } from 'wagmi';
import { ethers } from 'ethers';
import { Principal } from '@dfinity/principal';

import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faInfoCircle } from '@fortawesome/free-solid-svg-icons';

import { backend } from '@/model/backendProxy';
import { PaymentProvider, PaymentProviderType, BlockchainAsset, EvmOrderInput, BitcoinOrderInput } from '@/declarations/backend/backend.did';
import { defaultReleaseEvmGas, getEvmTokens, defaultCommitEvmGas } from '@/constants/evm_tokens';
import { CURRENCY_ICON_MAP } from '@/constants/currencyIconsMap';
import { ICP_TOKENS } from '@/constants/icp_tokens';
import { NetworkIds, NetworkProps } from '@/constants/networks';
import { rampErrorToString } from '@/model/utils/error';
import { BlockchainTypes, TokenOption } from '@/model/types';
import { blockchainAssetToBlockchainType, providerToProviderType } from '@/model/utils/utils';
import { fetchIcpTransactionFee, transferICPTokensToCanister } from '@/model/blockchain/icp';
import { depositInVault, estimateGasAndGasPrice, estimateOrderFees } from '@/model/blockchain/evm';
import { useUser } from '@/components/user/UserContext';
import { isSessionExpired } from '@/model/session';
import { getExchangeRate } from '@/model/utils/rate';
import { formatPrice, truncate } from '@/utils/helper';
import DynamicDots from '@/components/ui/DynamicDots';
import CurrencySelect from '@/components/ui/CurrencySelect';
import TokenSelect from '@/components/ui/TokenSelect';
import BlockchainSelect from '@/components/ui/BlockchainSelect';
import {
    fetchBitcoinCanisterAddress,
    fetchBitcoinTokenOptions,
    transferBitcoinToCanister,
    transferRuneToCanister
} from '@/model/blockchain/bitcoin';
import { getExplorerUrls } from '@/model/utils/blockchain';

const CreateOrder: React.FC = () => {
    const [cryptoAmount, setCryptoAmount] = useState(0);
    const [cryptoAmountUnits, setCryptoAmountUnits] = useState<bigint | null>(null);
    const [tokenOptions, setTokenOptions] = useState<TokenOption[]>([]);
    const [selectedToken, setSelectedToken] = useState<TokenOption | null>(null);
    const [selectedBlockchainAsset, setSelectedBlockchainAsset] = useState<BlockchainAsset>();
    const [blockchainType, setBlockchainType] = useState<BlockchainTypes>();
    const [selectedProviders, setSelectedProviders] = useState<PaymentProvider[]>([]);

    const [message, setMessage] = useState<string | null>(null);
    const [isLoading, setIsLoading] = useState(false);
    const [loadingMessage, setLoadingMessage] = useState<string | null>(null);
    const [txHash, setTxHash] = useState<string | null>(null);
    const [loadingRate, setLoadingRate] = useState(false);
    const [exchangeRate, setExchangeRate] = useState<number | null>(null);
    const [estimatedPrice, setEstimatedPrice] = useState<string | null>(null);
    const [offramperFeeCents, setOfframperFeeCents] = useState<number | null>(null);

    const { chain, chainId, address } = useAccount();
    const {
        user,
        currency: initialCurrency,
        sessionToken,
        icpAgent,
        principal,
        icpBalances,
        evmBalances,
        bitcoinBalance,
        bitcoinAddress,
        fetchBalances,
        refetchUser,
        logout
    } = useUser();
    const [currency, setCurrency] = useState<string>(initialCurrency ?? 'USD');
    const navigate = useNavigate();

    useEffect(() => {
        if (!user) navigate('/');
    }, [user, navigate]);

    if (!user) {
        navigate('/');
        return;
    }

    if (isSessionExpired(user)) {
        logout();
        navigate("/");
        return;
    }

    useEffect(() => {
        if (blockchainType && blockchainType === 'EVM') {
            setSelectedToken(null);
            let tokens: TokenOption[] = [];
            if (!chainId || !isValidChainId(chainId)) {
                setSelectedBlockchainAsset(undefined);
                return
            };
            setSelectedBlockchainAsset({ EVM: { chain_id: BigInt(chainId), token_address: [] } });
            tokens = getEvmTokens(chainId);
            setTokenOptions(tokens);
        }
    }, [chainId]);

    useEffect(() => {
        setSelectedToken(null);
        if (blockchainType === 'Bitcoin') {
            fetchBitcoinTokenOptions().then((bitcoinTokens) => {
                setTokenOptions(bitcoinTokens);
            }).catch((error) => {
                console.error("Error fetching bitcoin tokens:", error);
            });
        } else if (blockchainType === 'ICP') {
            setTokenOptions(ICP_TOKENS);
        } else if (blockchainType === 'EVM') {
            if (!chainId || !isValidChainId(chainId)) {
                setSelectedBlockchainAsset(undefined);
            } else {
                setTokenOptions(getEvmTokens(chainId));
            };
        }
    }, [blockchainType]);

    const handleBlockchainChange = (blockchainName: string) => {
        if (loadingRate) return;

        setTokenOptions([]);
        setSelectedToken(null);
        setBlockchainType(blockchainName as BlockchainTypes);
        if (blockchainName === "EVM") {
            if (!chainId) return;
            setSelectedBlockchainAsset({ EVM: { chain_id: BigInt(chainId), token_address: [] } });
        } else if (blockchainName === "ICP") {
            setSelectedBlockchainAsset({ ICP: { ledger_principal: Principal.fromText(ICP_TOKENS[0].address) } });
        } else if (blockchainName === "Solana") {
            setSelectedBlockchainAsset({ Solana: null });
        } else if (blockchainName === 'Bitcoin') {
            setSelectedBlockchainAsset({ Bitcoin: { rune_id: [] } });
        }
    };

    const handleTokenChange = (tokenAddress: string) => {
        if (loadingRate) return;

        const token = tokenOptions.find(token => token.address === tokenAddress);
        if (token && selectedBlockchainAsset) {
            setSelectedToken(token);
            switch (blockchainAssetToBlockchainType(selectedBlockchainAsset)) {
                case "ICP":
                    setSelectedBlockchainAsset({ ICP: { ledger_principal: Principal.fromText(token.address) } });
                case "Bitcoin":
                    let rune_id: [string] | [] = [];
                    if (!token.isNative) {
                        rune_id = [token.runeMetadata?.id!]
                    }
                    setSelectedBlockchainAsset({ Bitcoin: { rune_id } })
                case "EVM":
                    let token_address: [string] | [] = [];
                    if (!selectedToken?.isNative) {
                        token_address = [token.address]
                    }
                    "EVM" in selectedBlockchainAsset &&
                        setSelectedBlockchainAsset({
                            EVM: {
                                chain_id: selectedBlockchainAsset.EVM.chain_id,
                                token_address
                            }
                        })
            }
        }
    };

    const fetchOfframperFee = async (price: number) => {
        try {
            const fee = await backend.get_offramper_fee(BigInt(Math.round(price * 100)));
            setOfframperFeeCents(Number(fee));
        } catch (error) {
            console.error("Error fetching offramper fee:", error);
        }
    };

    useEffect(() => {
        const fetchPriceRate = async () => {
            setMessage(null);
            setLoadingRate(true);
            const isRune = selectedToken !== null && selectedToken.runeMetadata !== undefined;
            let priceRate = await getExchangeRate(currency, selectedToken!.rateSymbol, isRune);
            if (priceRate) {
                setExchangeRate(Number(priceRate))
            } else {
                setMessage("Could not estimate current price rates. \
                    You can still create the order, the final price is set dynamically when the order is locked.")
            }
            setLoadingRate(false);
        }

        if (selectedToken) {
            fetchPriceRate();
        }
    }, [selectedToken, currency]);

    useEffect(() => {
        if (exchangeRate) {
            let price = cryptoAmount * exchangeRate;
            setEstimatedPrice(price.toFixed(2));
            fetchOfframperFee(price);
        }
    }, [exchangeRate, cryptoAmount]);

    useEffect(() => {
        if (selectedBlockchainAsset && selectedToken && cryptoAmount > 0) {
            const roundedCryptoAmount = cryptoAmount.toFixed(selectedToken.decimals);
            if ('EVM' in selectedBlockchainAsset) {
                setCryptoAmountUnits(
                    selectedToken.isNative ? ethers.parseEther(roundedCryptoAmount)
                        : ethers.parseUnits(roundedCryptoAmount, selectedToken.decimals)
                );
            } else if ('ICP' in selectedBlockchainAsset) {
                setCryptoAmountUnits(BigInt(Math.round(Number(roundedCryptoAmount) * 10 ** selectedToken.decimals)));
            } else if ('Bitcoin' in selectedBlockchainAsset) {
                console.log("roundedCryptoAmoun = ", roundedCryptoAmount);
                if (selectedToken.isNative) {
                    setCryptoAmountUnits(BigInt(Math.round(Number(roundedCryptoAmount) * 10 ** selectedToken.decimals)));
                    console.log("cryptoAmountUnits = ", BigInt(Math.round(Number(roundedCryptoAmount) * 10 ** selectedToken.decimals)));
                } else {
                    setCryptoAmountUnits(BigInt(Math.round(Number(roundedCryptoAmount))));
                    console.log("cryptoAmountUnits = ", BigInt(Math.round(Number(roundedCryptoAmount))))
                }
            }
        }
    }, [cryptoAmount, selectedBlockchainAsset, selectedToken])

    const handleProviderSelection = (provider: PaymentProvider) => {
        if (selectedProviders.length === 0) {
            setSelectedProviders([provider]);
            return
        }
        if ('Revolut' in provider) {
            setMessage("We are waiting for revolut certificates to operate in production.")
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

    useEffect(() => {
        if (message) {
            const timer = setTimeout(() => {
                setMessage(null);
                setTxHash(null);
            }, 20000);

            return () => clearTimeout(timer);
        }
    }, [message]);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setTxHash(null);

        if (!user) {
            setMessage('User Not Found');
            return;
        }
        if (!sessionToken) throw new Error("Please authenticate to get a token session")
        if (isSessionExpired(user)) {
            setMessage('Token Is Expired');
            return;
        }

        if (!selectedBlockchainAsset) throw new Error('No blockchain selected');
        if (!selectedToken) throw new Error('No token selected');
        if (!cryptoAmountUnits) throw new Error('Could not parse crypto amount in native units');

        const providerTuples: [PaymentProviderType, PaymentProvider][] = selectedProviders.map((provider) => {
            const providerType: PaymentProviderType = providerToProviderType(provider);
            return [providerType, provider];
        });

        try {
            setIsLoading(true);
            setLoadingMessage("Creating order");

            const selectedAddress = user.addresses.find(
                addr => Object.keys(selectedBlockchainAsset)[0] in addr.address_type
            );
            if (!selectedAddress) {
                setMessage('No address available for the selected blockchain.');
                setIsLoading(false);
                return;
            }

            let evmOrderInput: [EvmOrderInput] | [] = []
            let bitcoinOrderInput: [BitcoinOrderInput] | [] = []
            const blockchain = blockchainAssetToBlockchainType(selectedBlockchainAsset);
            if (blockchain === 'EVM') {
                if (!chainId) throw new Error('Chain id is not available');
                setLoadingMessage("Estimating order gas");
                try {
                    const gasForCommit = await estimateGasAndGasPrice(
                        chainId,
                        { Commit: null },
                        defaultCommitEvmGas,
                    );
                    let tx_variant = selectedToken.isNative ? { Native: null } : { Token: null };
                    const gasForRelease = await estimateGasAndGasPrice(
                        chainId,
                        { 'Release': tx_variant },
                        defaultReleaseEvmGas,
                    );
                    console.log(`[createOrder] gasCommitEstimate: ${gasForCommit}, gasReleaseEstimate: ${gasForRelease}`);

                    const cryptoFee = await estimateOrderFees(
                        BigInt(chainId),
                        cryptoAmountUnits,
                        selectedToken.isNative ? [] : [selectedToken.address],
                        gasForCommit[0],
                        gasForRelease[0],
                    );
                    console.log(
                        `[estimateOrderFees] Offramper fee = ${offramperFeeCents}, Crypto fee = ${cryptoFee}`,
                    );

                    if (cryptoFee * BigInt(3) >= cryptoAmountUnits) {
                        console.error('[validateOrderFees] Total fees exceed crypto amount');
                        setMessage("Blockchain network gas prices will probably exceed the crypto amount.");
                        setIsLoading(false);
                        return;
                    }

                    setLoadingMessage("Depositing funds to vault")
                    const receipt = await depositInVault(chainId, selectedToken, cryptoAmountUnits);
                    setTxHash(receipt.hash);
                    console.log('Transaction receipt: ', receipt);

                    evmOrderInput = [{
                        estimated_gas_lock: gasForCommit[0],
                        estimated_gas_withdraw: gasForRelease[0],
                        tx_hash: receipt.hash
                    } as EvmOrderInput]
                } catch (e: any) {
                    setMessage(`${e.message || e}`);
                    setIsLoading(false);
                    return;
                }
            } else if (blockchain === 'ICP') {
                try {
                    if (!icpAgent) {
                        setMessage("ICP Agent not found");
                        setIsLoading(false);
                        return;
                    }

                    setLoadingMessage("Transfering funds to vault");
                    const ledgerCanister = Principal.fromText(selectedToken.address);
                    const fees = await fetchIcpTransactionFee(ledgerCanister);

                    const result = await transferICPTokensToCanister(icpAgent!, ledgerCanister, cryptoAmountUnits, fees);
                    console.log('Transaction result:', result);

                    fetchBalances();
                } catch (e: any) {
                    setMessage(`${e.message || e}`);
                    setIsLoading(false);
                    return;
                }
            } else if (blockchain === 'Bitcoin') {
                try {
                    if (!(window as any).unisat) {
                        setMessage("Unisat wallet not detected. Please install it to proceed.");
                        setIsLoading(false);
                        return;
                    }
                    setLoadingMessage("Fetching bitcoin canister address");

                    const bitcoinBackendAddress = await fetchBitcoinCanisterAddress(true);
                    if (!bitcoinBackendAddress) {
                        setMessage("Failed to retrieve Bitcoin canister address.");
                        setIsLoading(false);
                        return;
                    }
                    console.log("bitcoinBackendAddress = ", bitcoinBackendAddress);

                    let txid = "";
                    if (selectedToken.runeMetadata) {
                        setLoadingMessage("Sending Rune to canister");
                        try {
                            txid = await transferRuneToCanister(
                                cryptoAmountUnits,
                                bitcoinBackendAddress,
                                selectedToken.runeMetadata.id
                            );
                            setTxHash(txid);
                        } catch (e: any) {
                            setMessage(`Failed to send Rune`);
                            console.error(e);
                            setIsLoading(false);
                            return;
                        }
                    } else {
                        setLoadingMessage("Sending Bitcoin to canister");
                        try {
                            txid = await transferBitcoinToCanister(
                                cryptoAmountUnits,
                                bitcoinBackendAddress
                            );
                            setTxHash(txid);
                        } catch (e: any) {
                            setMessage(`Failed to send Bitcoin`);
                            console.error(e);
                            setIsLoading(false);
                            return;
                        }
                    }
                    bitcoinOrderInput = [{
                        tx_id: txid,
                        canister_address: bitcoinBackendAddress,
                    } as BitcoinOrderInput]
                    setLoadingMessage("Transaction sent, awaiting confirmation");
                } catch (error) {
                    setMessage(`Error creating Bitcoin order, error: ${error}`);
                    setIsLoading(false);
                    return;
                }
            } else {
                setIsLoading(false);
                throw new Error('Unsupported blockchain selected');
            }

            const result = await backend.create_order(
                sessionToken,
                currency,
                providerTuples,
                selectedBlockchainAsset,
                cryptoAmountUnits,
                selectedAddress,
                user.id,
                evmOrderInput,
                bitcoinOrderInput,
            );

            if ('Ok' in result) {
                setIsLoading(false);
                refetchUser();
                navigate(`/view?offramperId=${user.id}`);
            } else {
                setIsLoading(false);
                const errorMessage = rampErrorToString(result.Err);
                console.log("error  = ", errorMessage);
                setMessage(errorMessage);
            }
        } catch (error) {
            setMessage(`Error creating offramp order, error: ${error}`);
            setIsLoading(false);
        } finally {
            setIsLoading(false);
        }
    };

    const isValidChainId = (chainId: number | undefined): boolean => {
        if (!chainId) return false;

        const validChainIds = Object.values(NetworkIds).map((network) => network.id);
        return validChainIds.includes(chainId);
    };

    const isValidAddressMessage = () => {
        if (blockchainType === 'EVM') {
            if (!chainId || !address) return (
                <div className="my-2 text-red-400">
                    Please connect your Ethereum Wallet.
                </div>
            );
            return address && user?.addresses.some(
                addr => 'EVM' in addr.address_type && addr.address !== address) &&
                <div className="my-2 text-red-400">
                    Wallet address is not registered in your profile.
                </div>
        } else if (blockchainType === 'ICP') {
            if (!icpAgent || !principal) return (
                <div className="my-2 text-red-400">
                    Please connect your Internet Identity.
                </div>
            );
            return principal && user?.addresses.some(
                addr => 'ICP' in addr.address_type && addr.address !== principal?.toString()) &&
                <div className="my-2 text-red-400">
                    Principal connected is not registered in your profile.
                </div>
        } else if (blockchainType === 'Bitcoin') {
            if (!bitcoinAddress || !bitcoinBalance) return (
                <div className="my-2 text-red-400">
                    Please connect your Bitcoin wallet.
                </div>
            );
            return bitcoinAddress && user?.addresses.some(
                addr => 'Bitcoin' in addr.address_type && addr.address !== bitcoinAddress) &&
                <div className="my-2 text-red-400">
                    Please connect a bitcoin address in your profile.
                </div>
        }
    }

    const getAvailableBalance = () => {
        if (blockchainType === 'ICP' && selectedToken && icpBalances) {
            return icpBalances[selectedToken.name];
        } else if (blockchainType === 'EVM' && selectedToken && evmBalances) {
            if (selectedToken.isNative) return evmBalances[selectedToken.name] || '0';
            return evmBalances[selectedToken.address];
        } else if (blockchainType === 'Bitcoin') {
            if (selectedToken?.runeMetadata && bitcoinBalance) {
                return bitcoinBalance?.runes[selectedToken.runeMetadata.id];
            }
            return bitcoinBalance;
        }
        return null
    };

    const validInputs = user !== null
        && selectedBlockchainAsset !== undefined
        && (isValidAddressMessage() === undefined || isValidAddressMessage() === false)
        && selectedProviders.length > 0
        && selectedToken !== null
        && cryptoAmountUnits && cryptoAmountUnits > 0
        && (getAvailableBalance() ? cryptoAmountUnits <= getAvailableBalance()!.raw : true);

    return (
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-2xl w-full mx-auto p-6">
            {isLoading && (
                <div className="absolute inset-0 rounded-lg bg-black bg-opacity-60 flex flex-col items-center justify-center z-40">
                    <div className="w-10 h-10 border-t-4 border-b-4 border-indigo-400 rounded-full animate-spin mb-4"></div>
                    {loadingMessage && (
                        <div className="text-2xl font-bold mt-2">
                            {loadingMessage}<DynamicDots isLoading />
                        </div>
                    )}
                </div>
            )}

            <div className="text-center mb-8">
                <h2 className="text-2xl font-semibold">Create Order</h2>
            </div>

            <form onSubmit={handleSubmit} className="space-y-6">
                <div className="flex justify-between items-center mb-4">

                    {/* Label and Info Icon */}
                    <div className="w-24 flex-none flex items-center justify-center relative">
                        <label>Price:</label>
                        <span className="text-gray-400 group pointer-events-none">
                            <FontAwesomeIcon icon={faInfoCircle} className="cursor-pointer items-center ml-2 pointer-events-auto" />
                            <div className="absolute left-1/2 transform -translate-x-1/2 mt-2 w-56 bg-gray-500 text-sm text-gray-300 p-3 rounded-md shadow-lg opacity-0 group-hover:opacity-100 transition-opacity duration-300 z-10">
                                <p className="mb-2">
                                    Current price calculated using a decentralized oracle (XRC canister).
                                    Prices are always updated to the current market price.
                                </p>
                                {offramperFeeCents ? (
                                    <p className="font-semibold">
                                        You will collect a fee of:
                                        {<span className="text-green-200">
                                            <FontAwesomeIcon icon={CURRENCY_ICON_MAP[currency]} className="ml-2" />
                                            {formatPrice(offramperFeeCents)}
                                        </span>}
                                    </p>
                                ) : (
                                    <p>Introduce token and amount to estimate how much you will earn from fees.</p>
                                )}
                            </div>
                        </span>
                    </div>

                    {/* Price Input and Currency Dropdown */}
                    <div className="flex-grow flex items-center w-full">
                        <input
                            type="number"
                            value={estimatedPrice ? estimatedPrice : "0.00"}
                            className="py-2 px-3 w-full border bg-gray-100 dark:bg-gray-700 border-gray-300 dark:border-gray-600 rounded-l-lg"
                            required
                            disabled
                            style={{ WebkitAppearance: 'none', MozAppearance: 'textfield' }}
                        />
                        <CurrencySelect
                            selected={currency}
                            onChange={setCurrency}
                            className="text-gray-700 dark:text-gray-300 border-gray-300 dark:border-gray-600"
                            buttonClassName="bg-gray-100 dark:bg-gray-700 border-gray-300 dark:border-gray-600 rounded-r-lg"
                            dropdownClassName="bg-gray-100 dark:bg-gray-700 border-gray-300 dark:border-gray-600 hover:bg-gray-200 dark:hover:bg-gray-600"
                        />
                    </div>
                </div>

                <div className="flex justify-between items-center mb-4 relative">
                    <label className="w-24 text-gray-700 dark:text-gray-300">Crypto:</label>
                    <input
                        type="number"
                        value={cryptoAmount}
                        onChange={(e) => setCryptoAmount(selectedToken ? Number(Number(e.target.value).toFixed(selectedToken.decimals)) : Number(e.target.value))}
                        className={`flex-grow py-2 px-3 border ${cryptoAmountUnits && getAvailableBalance() && cryptoAmountUnits > getAvailableBalance()!.raw ? 'border-red-500' : "border-gray-300 dark:border-gray-600"
                            } bg-gray-100 dark:bg-gray-700 outline-none rounded-md focus:ring ${cryptoAmountUnits && getAvailableBalance() && cryptoAmountUnits > getAvailableBalance()!.raw ? 'focus:ring-red-500' : "focus:border-blue-900"
                            } text-gray-700 dark:text-gray-300`}
                        required
                        style={{
                            appearance: 'textfield',
                            WebkitAppearance: 'none',
                            MozAppearance: 'textfield',
                        }}
                    />
                    <span className="absolute right-2 top-1/2 transform -translate-y-1/2 text-gray-400 text-xs">
                        max: {getAvailableBalance() ? getAvailableBalance()!.formatted : "0.00"} {selectedToken?.name}
                    </span>
                </div>

                <div className="flex justify-between items-center mb-4">
                    <label className="w-24 text-gray-700 dark:text-gray-300">Blockchain:</label>
                    <BlockchainSelect
                        selectedBlockchain={blockchainType}
                        onChange={handleBlockchainChange}
                        className="flex-grow flex items-center"
                        buttonClassName="bg-gray-100 dark:bg-gray-700 border-gray-300 dark:border-gray-600 rounded-md"
                    />
                </div>

                {blockchainType &&
                    <div className="flex justify-between items-center mb-4">
                        <label className="w-24 text-gray-700 dark:text-gray-300">Token:</label>
                        <TokenSelect
                            tokenOptions={tokenOptions}
                            selectedToken={selectedToken}
                            onChange={handleTokenChange}
                            className="flex-grow flex items-center"
                            buttonClassName="bg-gray-100 dark:bg-gray-700 border-gray-300 dark:border-gray-600 rounded-md"
                        />
                    </div>
                }

                {loadingRate && (
                    <div className="my-2 flex justify-center items-center space-x-2">
                        <div className="w-6 h-6 border-t-2 border-b-2 border-indigo-400 rounded-full animate-spin"></div>
                        <div className="text-sm font-medium text-gray-700 dark:text-gray-300">Estimating Prices...</div>
                    </div>
                )}

                {isValidAddressMessage()}

                {chainId && selectedBlockchainAsset && Object.keys(selectedBlockchainAsset)[0] === "EVM" && (
                    <div className={`my-2 text-sm font-medium ${isValidChainId(chainId) ? 'text-green-600' : 'text-red-600'}`}>
                        {isValidChainId(chainId) ? `On chain: ${chain?.name}` : 'Please connect to a valid network'}
                    </div>
                )}

                <hr className="border-t border-gray-300 dark:border-gray-600 w-full my-4" />

                <div className="my-4 mx-auto">
                    <label className="block text-gray-700 dark:text-gray-300 mb-2">Payment Providers:</label>
                    {user?.payment_providers.map((provider, index) => {
                        return (
                            <div key={index} className="block mb-2">
                                <input
                                    type="checkbox"
                                    id={`provider-${index}`}
                                    className="mr-2"
                                    checked={selectedProviders!.includes(provider)}
                                    onChange={() => handleProviderSelection(provider)}
                                />
                                <label htmlFor={`provider-${index}`} className="text-gray-700 dark:text-gray-300">
                                    {'PayPal' in provider &&
                                        <>
                                            <span className='font-semibold'>Paypal</span>
                                            <div>{provider.PayPal.id}</div>
                                        </>
                                    }
                                    {'Revolut' in provider &&
                                        <>
                                            <span className='font-semibold'>Revolut</span>
                                            <div>{provider.Revolut.id} (Scheme): ${provider.Revolut.scheme}</div>
                                        </>
                                    }
                                </label>
                            </div>
                        );
                    })}
                </div>

                <hr className="border-t border-gray-300 dark:border-gray-600 w-full my-4" />

                <div className="flex justify-center">
                    <button
                        type="submit"
                        className={`px-4 py-2 rounded-md flex items-center justify-center space-x-2 ${validInputs ?
                            'bg-green-600 text-white hover:bg-green-700 focus:outline-none'
                            : 'bg-gray-500 text-gray-300 cursor-not-allowed'}`
                        }
                        disabled={!validInputs}
                    >
                        {isLoading ? (
                            <>
                                <div className="w-5 h-5 border-t-2 border-b-2 border-white rounded-full animate-spin"></div>
                                <span>Creating<DynamicDots isLoading /></span>
                            </>
                        ) : (
                            <span>Create Order</span>
                        )}
                    </button>
                </div>
            </form>

            {txHash && blockchainType && (
                <div className="text-blue-400 relative mt-4 text-sm font-medium flex items-center justify-center text-center z-50">
                    <a href={`${getExplorerUrls(blockchainType, "", undefined, txHash)?.transaction}`} target="_blank" className="hover:underline z-50">
                        View tx: {truncate(txHash, 6, 6)}
                    </a>
                </div>
            )}
            {!isLoading && message && <p className="mt-4 text-sm font-medium text-red-600">{message}</p>}
        </div>
    );
}

export default CreateOrder;
