import React, { useState, useEffect, useRef } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAccount } from 'wagmi';
import { Principal } from '@dfinity/principal';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faInfoCircle } from '@fortawesome/free-solid-svg-icons';

import { PaymentProvider, PaymentProviderType, BlockchainAsset, DepositInput, RateAsset, FeeQuote } from '@/declarations/icramp_backend/icramp_backend.did';
import { defaultCommitEvmGas, defaultReleaseEvmGas, getEvmTokens } from '@/constants/evm_tokens';
import { CURRENCY_ICON_MAP } from '@/constants/currencyIconsMap';
import { ICP_TOKENS } from '@/constants/icp_tokens';
import { NetworkIds } from '@/constants/networks';
import { backend } from '@/model/backendProxy';
import { rampErrorToString } from '@/model/helpers/error';
import { BlockchainTypes, TokenOption } from '@/model/types';
import { blockchainAssetToBlockchainType, providerToProviderType } from '@/model/helpers/types';
import { fetchSolanaTokenOptions } from '@/model/blockchain/solana';
import { isSessionExpired } from '@/model/session';
import { fetchBitcoinTokenOptions } from '@/model/blockchain/bitcoin';
import { getExchangeRate } from '@/utils/rates';
import { formatPrice, truncate } from '@/utils/formatters';
import { getExplorerUrls } from '@/utils/explorers';

import DynamicDots from '@/components/ui/DynamicDots';
import CurrencySelect from '@/components/ui/CurrencySelect';
import TokenSelect from '@/components/ui/TokenSelect';
import BlockchainSelect from '@/components/ui/BlockchainSelect';
import { Balance, useUser } from '@/components/user/UserContext';
import {
    useOrderEvm,
    useOrderSolana,
    useOrderBitcoin,
    useOrderIcp,
    useParsedAmount,
    useAutoClearMessage,
} from '@/components/order/hooks';
import { estimateGasAndGasPrice } from '@/model/blockchain/evm';
import { getFeeQuote } from '@/model/blockchain/fees';

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
    const [feeQuote, setFeeQuote] = useState<FeeQuote | null>(null);

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
        solanaBalance,
        solanaPubkey,
        fetchBalances,
        refetchUser,
        logout
    } = useUser();
    const [currency, setCurrency] = useState<string>(initialCurrency ?? 'USD');
    const navigate = useNavigate();
    const {
        makeSolanaDeposit,
        solanaNetworkLabel,
        isSolflare,
        solflareMismatch,
        waitForSolanaConfirmation
    } = useOrderSolana();
    const { makeEvmDeposit } = useOrderEvm();
    const { makeBitcoinDeposit } = useOrderBitcoin();
    const { makeIcpDeposit } = useOrderIcp();

    const feeReqRef = useRef(0);

    useEffect(() => {
        if (!user) navigate('/');
    }, [user, navigate]);

    if (!user || isSessionExpired(user)) {
        logout();
        navigate('/');
        return;
    }

    useAutoClearMessage(message, () => { setMessage(null); setTxHash(null); });
    useParsedAmount(cryptoAmount, selectedToken, selectedBlockchainAsset, setCryptoAmountUnits);

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
            fetchBitcoinTokenOptions().then(t => setTokenOptions(t))
                .catch((error) => console.error("Error fetching bitcoin tokens:", error));
        } else if (blockchainType === 'Solana') {
            setSelectedBlockchainAsset({ Solana: { spl_token: [] } });
            fetchSolanaTokenOptions().then(t => setTokenOptions(t))
                .catch((err) => console.error("Error fetching solana tokens:", err))
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
            setSelectedBlockchainAsset({ Solana: { spl_token: [] } });
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
                    break;
                case "Bitcoin":
                    let rune_id: [string] | [] = [];
                    if (!token.isNative) rune_id = [token.runeMetadata?.id!]
                    setSelectedBlockchainAsset({ Bitcoin: { rune_id } });
                    break;
                case "Solana":
                    let spl_token: [string] | [] = [];
                    if (!token.isNative) spl_token = [token.address]
                    setSelectedBlockchainAsset({ Solana: { spl_token } });
                    break;
                case "EVM":
                    let token_address: [string] | [] = [];
                    if (!token?.isNative) token_address = [token.address]
                    "EVM" in selectedBlockchainAsset &&
                        setSelectedBlockchainAsset({
                            EVM: {
                                chain_id: selectedBlockchainAsset.EVM.chain_id,
                                token_address
                            }
                        });
                    break;
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
            if (!selectedBlockchainAsset || !selectedToken) return;

            setMessage(null);
            setLoadingRate(true);

            let asset: RateAsset;
            switch (blockchainAssetToBlockchainType(selectedBlockchainAsset)) {
                case "Bitcoin":
                    asset = { Rune: { name: selectedToken.rateSymbol } };
                    break;
                case "Solana":
                    asset = { Solana: { symbol: selectedToken.rateSymbol, mint: [selectedToken.address] } };
                    break;
                default:
                    asset = { Crypto: { symbol: selectedToken.rateSymbol } };
            };

            let priceRate = await getExchangeRate(currency, asset);
            if (priceRate) {
                setExchangeRate(Number(priceRate))
            } else {
                setMessage("Could not estimate current price rates. \
                    You can still create the order, the final price is set dynamically when the order is locked.")
            }
            setLoadingRate(false);
        }

        fetchPriceRate();
    }, [selectedBlockchainAsset, selectedToken, currency]);

    useEffect(() => {
        const myReq = ++feeReqRef.current;
        let alive = true;

        (async () => {
            setFeeQuote(null);
            setMessage(null);
            if (!selectedBlockchainAsset || !selectedToken || !cryptoAmountUnits) return;

            try {
                if ('EVM' in selectedBlockchainAsset) {
                    const gasLock = (await estimateGasAndGasPrice(
                        Number(selectedBlockchainAsset.EVM.chain_id),
                        { Commit: null },
                        defaultCommitEvmGas
                    ))[0];

                    const txVariant = selectedToken.isNative ? { Native: null } : { Token: null };
                    const gasWithdraw = (await estimateGasAndGasPrice(
                        Number(selectedBlockchainAsset.EVM.chain_id),
                        { Release: txVariant },
                        defaultReleaseEvmGas
                    ))[0];

                    const q = await getFeeQuote(
                        selectedBlockchainAsset,
                        cryptoAmountUnits,
                        { estimated_gas_lock: gasLock, estimated_gas_withdraw: gasWithdraw }
                    );
                    if (alive && feeReqRef.current === myReq) setFeeQuote(q);
                } else {
                    const q = await getFeeQuote(selectedBlockchainAsset, cryptoAmountUnits);
                    if (alive && feeReqRef.current === myReq) setFeeQuote(q);
                }
            } catch (e: unknown) {
                console.error('fee quote error', (e as Error)?.message ?? String(e));
                if (alive && feeReqRef.current === myReq) {
                    setFeeQuote(null);
                    setMessage((e as Error)?.message ?? 'Fee quote failed');
                }
            }
        })();

        return () => { alive = false; };
    }, [selectedBlockchainAsset, selectedToken, cryptoAmountUnits]);

    useEffect(() => {
        if (exchangeRate) {
            let price = cryptoAmount * exchangeRate;
            setEstimatedPrice(price.toFixed(2));
            fetchOfframperFee(price);
        }
    }, [exchangeRate, cryptoAmount]);

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
        if (!feeQuote) throw new Error('Could not compute fees');
        if (cryptoAmountUnits - feeQuote?.total_fee < 0) throw new Error('Fees will probably be higher than crypto amount');

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

            let depositInput: [DepositInput] | [] = []
            const blockchain = blockchainAssetToBlockchainType(selectedBlockchainAsset);
            if (blockchain === 'EVM') {
                if (!chainId) throw new Error('Chain id is not available');
                setLoadingMessage('Estimating order gas & depositing');
                try {
                    const { depositInput: evmDep, txHash } =
                        await makeEvmDeposit(chainId, selectedToken, cryptoAmountUnits);
                    setTxHash(txHash);
                    depositInput = evmDep;
                } catch (e: any) {
                    setMessage(e.message || String(e));
                    setIsLoading(false);
                    return;
                }
            } else if (blockchain === 'ICP') {
                try {
                    if (!icpAgent) { setMessage('ICP Agent not found'); setIsLoading(false); return; }
                    setLoadingMessage('Transferring funds to vault');
                    await makeIcpDeposit(icpAgent, selectedToken, cryptoAmountUnits);
                    fetchBalances();
                } catch (e: any) {
                    setMessage(e.message || String(e));
                    setIsLoading(false);
                    return;
                }
            } else if (blockchain === 'Bitcoin') {
                try {
                    setLoadingMessage('Sending funds to Bitcoin vault');
                    const { depositInput: btcDep, txid } =
                        await makeBitcoinDeposit(cryptoAmountUnits, selectedToken);
                    setTxHash(txid);
                    depositInput = btcDep;
                    setLoadingMessage('Transaction sent, awaiting confirmation');
                } catch (e: any) {
                    setMessage(`Error creating Bitcoin order, error: ${e?.message || e}`);
                    setIsLoading(false);
                    return;
                }
            } else if (blockchain === 'Solana') {
                try {
                    setLoadingMessage("Sending funds to Solana vault");
                    const { depositInput: solDeposit, txSig } =
                        await makeSolanaDeposit(cryptoAmountUnits, selectedToken);
                    setTxHash(txSig);
                    await waitForSolanaConfirmation(txSig, { timeoutMs: 90_000 });

                    depositInput = solDeposit;
                    fetchBalances();
                    setLoadingMessage("Transaction sent, awaiting confirmation");
                } catch (e: any) {
                    setMessage(`Error creating Solana order: ${e?.message || e}`);
                    setIsLoading(false);
                    return;
                }
            } else {
                setIsLoading(false);
                throw new Error('Unsupported blockchain selected');
            }

            console.log("selectedBlockchainAsset = ", selectedBlockchainAsset);
            const result = await backend.create_order(
                sessionToken,
                currency,
                providerTuples,
                selectedBlockchainAsset,
                cryptoAmountUnits,
                selectedAddress,
                user.id,
                depositInput,
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
        const mismatch = (
            variant: 'EVM' | 'ICP' | 'Bitcoin' | 'Solana',
            current?: string | null
        ) => !!current && !!user?.addresses?.some(a => (variant in a.address_type) && a.address !== current);

        const cfg = {
            EVM: { ok: !!chainId && !!address, cur: address, miss: 'Please connect your Ethereum Wallet.', notReg: 'Wallet address is not registered in your profile.' },
            ICP: { ok: !!icpAgent && !!principal, cur: principal?.toString(), miss: 'Please connect your Internet Identity.', notReg: 'Principal connected is not registered in your profile.' },
            Bitcoin: { ok: !!bitcoinAddress && !!bitcoinBalance, cur: bitcoinAddress, miss: 'Please connect your Bitcoin wallet.', notReg: 'Please connect a bitcoin address in your profile.' },
            Solana: { ok: !!solanaPubkey && !!solanaBalance, cur: solanaPubkey, miss: 'Please connect your Solana wallet', notReg: 'Please link a solana account in your profile.' },
        } as const;

        const c = cfg[blockchainType as keyof typeof cfg];
        if (!c) return null;
        if (!c.ok) return <div className="my-2 text-red-400">{c.miss}</div>;
        if (mismatch(blockchainType as any, c.cur)) {
            return <div className="my-2 text-red-400">{c.notReg}</div>;
        }
        return null;
    }

    const getAvailableBalance = (): Balance | null => {
        if (blockchainType === 'ICP' && selectedToken && icpBalances) {
            return icpBalances[selectedToken.name] ?? null;
        } else if (blockchainType === 'EVM' && selectedToken && evmBalances) {
            if (selectedToken.isNative) return evmBalances[selectedToken.name] ?? null;
            return evmBalances[selectedToken.address] ?? null;
        } else if (blockchainType === 'Bitcoin') {
            if (selectedToken?.runeMetadata && bitcoinBalance)
                return bitcoinBalance.runes[selectedToken.runeMetadata.id] ?? null;
            return bitcoinBalance?.balance ?? null;
        } else if (blockchainType === 'Solana') {
            if (selectedToken?.isNative) return solanaBalance?.balance ?? null;
            return selectedToken?.address
                ? (solanaBalance?.splTokens[selectedToken.address] ?? null)
                : null;
        }
        return null
    };

    const addrMsg = isValidAddressMessage();
    const validInputs = user !== null
        && selectedBlockchainAsset !== undefined
        && (addrMsg == null)
        && selectedProviders.length > 0
        && selectedToken !== null
        && (() => {
            const bal = getAvailableBalance();
            if (!bal) return true;
            const balRaw = typeof bal.raw === 'bigint' ? bal.raw : BigInt(bal.raw);
            return typeof cryptoAmountUnits === 'bigint'
                ? cryptoAmountUnits <= balRaw
                : Number(cryptoAmountUnits) <= Number(balRaw);
        })();

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

                {addrMsg}

                {chainId && selectedBlockchainAsset && Object.keys(selectedBlockchainAsset)[0] === "EVM" && (
                    <div className={`my-2 text-sm font-medium ${isValidChainId(chainId) ? 'text-green-600' : 'text-red-600'}`}>
                        {isValidChainId(chainId) ? `On chain: ${chain?.name}` : 'Please connect to a valid network'}
                    </div>
                )}

                {selectedBlockchainAsset && Object.keys(selectedBlockchainAsset)[0] === "Solana" && (
                    <div className="my-2 text-sm font-medium">
                        <span className="text-green-600">dApp cluster: {solanaNetworkLabel}</span>
                        {isSolflare && solflareMismatch && (
                            <span className="block text-xs text-red-500 mt-1">
                                Solflare is on a different cluster → Settings → General → Network → {solanaNetworkLabel}
                            </span>
                        )}
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
