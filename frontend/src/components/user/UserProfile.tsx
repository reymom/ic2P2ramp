import React, { useEffect, useMemo, useRef, useState } from 'react';
import { useLocation, useNavigate, useSearchParams } from 'react-router-dom';
import { useAccount } from 'wagmi';
import clsx from 'clsx';
import { ConnectButton } from '@rainbow-me/rainbowkit';

import { PaymentProvider, TransactionAddress } from '@/declarations/icramp_backend/icramp_backend.did';
import { backend } from '@/model/backendProxy';
import { PaymentProviderTypes, revolutSchemeTypes, revolutSchemes } from '@/model/types';
import { isSessionExpired } from '@/model/session';
import { userTypeToString } from '@/model/helpers/types';
import { rampErrorToString } from '@/model/helpers/error';
import { CURRENCY_ICON_MAP } from '@/constants/currencyIconsMap';
import { truncate } from '@/utils/formatters';
import { getExplorerUrls } from '@/utils/explorers';
import { mapCountryToPlatform } from '@/utils/stripe';
import {
    buildEvmProviders,
    buildIcpProvider,
    buildSolProvider,
    EvmGroup,
    getRecvAddr,
    groupEvmProviders,
    hasEvm,
    hasIcp,
    hasSol,
    deepEqual
} from '@/utils/cryptoProviders';
import CurrencySelect from '@/components/ui/CurrencySelect';
import BalancesDashboard from './BalanceDashboard';
import { useUser } from './UserContext';
import { ProviderIcon } from '../ui/ProviderIcon';
import { startStripeKyc } from '@/hooks/useStripeKyc';
import { getRegisteredSolanaTokens, Registry } from '@/model/blockchain/solana';

import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faRemove, faSpinner, faSync, faCopy, faCheckCircle } from '@fortawesome/free-solid-svg-icons';
import icpLogo from "@/assets/blockchains/icp-logo.svg";
import ethereumLogo from "@/assets/blockchains/ethereum-logo.png";
import bitcoinLogo from "@/assets/blockchains/bitcoin-logo.svg";
import solanaLogo from "@/assets/blockchains/solana-logo.png"
import { NetworkIds } from '@/constants/networks';
import { ICP_TOKENS } from '@/constants/icp_tokens';
import { SPL_TOKEN_LOGOS } from '@/constants/solana_logos';

const UserProfile: React.FC = () => {
    const [providerType, setProviderType] = useState<PaymentProviderTypes>();
    const [providerId, setProviderId] = useState('');
    const [selectedAddressType, setSelectedAddressType] = useState<'ICP' | 'EVM' | 'Bitcoin' | 'Solana'>('Bitcoin');
    const [addressDropdownOpen, setAddressDropdownOpen] = useState(false);
    const [revolutScheme, setRevolutScheme] = useState<revolutSchemeTypes>('UK.OBIE.SortCodeAccountNumber');
    const [revolutName, setRevolutName] = useState('');
    const [stripeCountry, setStripeCountry] = useState('ES');
    const [loadingStripe, setLoadingStripe] = useState(false);

    const [message, setMessage] = useState('');
    const [loadingUnisat, setLoadingUnisat] = useState(false);
    const [loadingAddAddress, setLoadingAddAddress] = useState(false);
    const [loadingAddProvider, setLoadingAddProvider] = useState(false);
    const [isClicked, setIsClicked] = useState(false);
    const [removing, setRemoving] = useState(false);
    const [copiedIndex, setCopiedIndex] = useState<number | null>(null);
    const [activeTab, setActiveTab] = useState<'profile' | 'balances'>('profile');

    const { address, isConnected, chainId } = useAccount();
    const {
        user,
        currency,
        loginMethod,
        sessionToken,
        principal,
        bitcoinAddress,
        solanaPubkey,
        backendActor,
        connectUnisat,
        connectSolana,
        loginInternetIdentity,
        refetchUser,
        setCurrency,
        logout
    } = useUser();
    const navigate = useNavigate();
    const [searchParams] = useSearchParams();
    const dropdownRef = useRef<HTMLDivElement>(null);
    const processedStripeRef = useRef<string | null>(null);
    const location = useLocation();

    const [solRegistry, setSolRegistry] = useState<Registry | null>(null);
    useEffect(() => {
        getRegisteredSolanaTokens().then(setSolRegistry).catch(() => setSolRegistry({}));
    }, []);

    const [cryptoDropdownOpen, setCryptoDropdownOpen] = useState(false);
    const [selectedCryptoType, setSelectedCryptoType] = useState<'evm' | 'sol' | 'icp' | null>(null);
    const cryptoDropdownRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        const handleClickOutside = (event: MouseEvent) => {
            if (cryptoDropdownRef.current && !cryptoDropdownRef.current.contains(event.target as Node)) {
                setCryptoDropdownOpen(false);
            }
        };
        document.addEventListener('mousedown', handleClickOutside);
        return () => document.removeEventListener('mousedown', handleClickOutside);
    }, []);

    useEffect(() => {
        if (!user) {
            navigate('/')
            return;
        };
    }, [user, navigate]);

    if (!user) {
        navigate('/');
        return null
    }

    if (isSessionExpired(user)) {
        logout();
        navigate("/");
        return;
    }

    useEffect(() => {
        function handleClickOutside(event: MouseEvent) {
            if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
                setAddressDropdownOpen(false);
            }
        }

        document.addEventListener('mousedown', handleClickOutside);
        return () => {
            document.removeEventListener('mousedown', handleClickOutside);
        };
    }, [dropdownRef]);

    const copyToClipboard = (text: string, index: number) => {
        navigator.clipboard.writeText(text).then(() => {
            setCopiedIndex(index);
            setTimeout(() => setCopiedIndex(null), 2000);
        }).catch(err => {
            console.error('Failed to copy text: ', err);
        });
    };


    const handleRefresh = async () => {
        setIsClicked(true);
        await refetchUser();
        setTimeout(() => setIsClicked(false), 1000);
    };

    const handleAddressSelectOption = (addressType: 'ICP' | 'EVM' | 'Bitcoin' | 'Solana') => {
        setSelectedAddressType(addressType);
        setAddressDropdownOpen(false);
    };

    const handleInternetIdentityLogin = async () => {
        await loginInternetIdentity();
    };

    const handleConnectUnisat = async () => {
        setLoadingUnisat(true);
        await connectUnisat();
        setLoadingUnisat(false);
    };

    const handleCryptoSelectOption = (type: 'evm' | 'sol' | 'icp') => {
        setSelectedCryptoType(type);
        setCryptoDropdownOpen(false);
    };

    /* ------------------ add-options (one click → many providers) ------------------ */
    const cryptoAddOptions = useMemo(() => {
        if (!currency || (currency !== 'USD' && currency !== 'EUR')) return [];

        const symbol = currency === 'USD' ? 'USDC' : 'EURC';
        const evmAddr = getRecvAddr(user.addresses, 'EVM');
        const solAddr = getRecvAddr(user.addresses, 'Solana');
        const icpAddr = getRecvAddr(user.addresses, 'ICP');

        const opts: {
            key: string;
            title: string;
            enabled: boolean;
            build: () => Promise<PaymentProvider[]>;
        }[] = [];

        // ---- EVM ----
        if (evmAddr) {
            opts.push({
                key: 'evm',
                title: `Add EVM ${symbol}`,
                enabled: true,
                build: async () => await buildEvmProviders(evmAddr, symbol),
            });
        }

        // ---- Solana ----
        if (solAddr) {
            opts.push({
                key: 'sol',
                title: `Add Solana ${symbol}`,
                enabled: true,
                build: async () => await buildSolProvider(solAddr, symbol).then(p => p ? [p] : []),
            });
        }

        // ICP (only USD)
        if (currency === 'USD' && icpAddr) {
            opts.push({
                key: 'icp',
                title: 'Add ICP ckUSD',
                enabled: true,
                build: async () => await buildIcpProvider(icpAddr).then(p => p ? [p] : []),
            });
        }

        return opts;
    }, [currency, user.addresses]);

    /* ------------------ UI groups ------------------ */
    const evmGroup = useMemo(() => groupEvmProviders(user.payment_providers), [user.payment_providers]);

    const solCrypto = useMemo(
        () => user.payment_providers.filter(p => 'Crypto' in p && 'Solana' in p.Crypto.asset),
        [user.payment_providers],
    );

    const icpCrypto = useMemo(
        () => user.payment_providers.filter(p => 'Crypto' in p && 'ICP' in p.Crypto.asset),
        [user.payment_providers],
    );

    const visibleProviderTypes = useMemo<PaymentProviderTypes[]>(() => {
        const baseOff = ['PayPal', 'Revolut', 'Stripe'] as const;
        const baseOn = ['PayPal', 'Revolut', 'Email'] as const;
        const crypto = cryptoAddOptions.length > 0 ? (['Crypto'] as const) : [];
        return 'Offramper' in user.user_type
            ? [...baseOff, ...crypto]
            : [...baseOn, ...crypto];
    }, [user.user_type, cryptoAddOptions]);

    useEffect(() => {
        const acct = searchParams.get('stripe_acct');
        const platform = searchParams.get('platform') ?? mapCountryToPlatform(stripeCountry);
        if (!acct || !platform || !sessionToken) return;

        if (processedStripeRef.current === acct) return;
        processedStripeRef.current = acct;

        setLoadingAddProvider(true);
        (async () => {
            try {
                const provider = { Stripe: { account_id: acct, platform } };
                const r = await backend.add_user_payment_provider(
                    user.id,
                    sessionToken,
                    provider,
                );
                if ('Err' in r) {
                    setMessage(`Failed to add Stripe: ${rampErrorToString(r.Err)}`);
                    return;
                }
                await refetchUser();
            } catch (e: any) {
                setMessage(`Failed to finalize Stripe: ${e?.message ?? String(e)}`);
            } finally {
                setLoadingAddProvider(false);
                navigate(location.pathname, { replace: true });
            }
        })();
    }, [
        searchParams,
        stripeCountry,
        user,
        sessionToken,
        backend,
        refetchUser,
        setMessage,
    ]);

    const startStripeOnboardingProfile = async () => {
        await startStripeKyc({
            userType: userTypeToString(user.user_type) as 'Offramper' | 'Onramper',
            loginMethod: loginMethod ?? user.login,
            backendActor,
            email: providerId,
            country: stripeCountry,
            storeUserData: false,
            setMessage,
            setLoading: setLoadingStripe,
        });
    };

    const handleAddProvider = async () => {
        if (!sessionToken) throw new Error("Please authenticate to get a token session");
        if (!providerType) return;

        setLoadingAddProvider(true);
        setMessage('');

        try {
            if (providerType !== 'Crypto') {
                let newProvider: PaymentProvider;
                if (providerType === 'PayPal') {
                    newProvider = { PayPal: { id: providerId } };
                } else if (providerType === 'Revolut') {
                    if (userTypeToString(user.user_type) === 'Offramper' && !revolutName) {
                        setMessage('Name is required.');
                        setLoadingAddProvider(false);
                        return;
                    }
                    newProvider = { Revolut: { id: providerId, scheme: revolutScheme, name: revolutName ? [revolutName] : [] } };
                } else if (providerType === 'Stripe') {
                    if (userTypeToString(user.user_type) === 'Onramper') {
                        setMessage('Stripe is just needed for Offrampers');
                        return;
                    }
                    setMessage('Use "Start Stripe Onboarding" to add a Stripe provider.');
                    return;
                } else if (providerType === 'Email') {
                    if (userTypeToString(user.user_type) === 'Offramper') {
                        setMessage('Email available just for Onrampers');
                        return;
                    }
                    const email = providerId.trim();
                    const ok =
                        email.length <= 254 &&
                        email.includes('@') &&
                        !email.includes(' ') &&
                        !email.startsWith('@') &&
                        !email.endsWith('@');
                    if (!ok) {
                        setMessage('Invalid email.');
                        return;
                    }
                    newProvider = { Email: { email } };
                } else {
                    setMessage('Unknown payment provider');
                    return;
                }
                const result = await backend.add_user_payment_provider(user.id, sessionToken, newProvider);
                if ('Ok' in result) {
                    refetchUser();
                } else {
                    setMessage(rampErrorToString(result.Err));
                }

            } else {
                if (!selectedCryptoType) {
                    setMessage('Select a blockchain type');
                    setLoadingAddProvider(false);
                    return;
                }
                const optionKey = selectedCryptoType.toLowerCase();
                const option = cryptoAddOptions.find(o => o.key === optionKey);
                if (!option) {
                    setMessage('No crypto option available');
                    setLoadingAddProvider(false);
                    return;
                }
                const providers = await option.build();
                if (providers.length === 0) {
                    setMessage('No valid token found for selected blockchain and currency');
                    setLoadingAddProvider(false);
                    return;
                }
                let added = 0;
                for (const p of providers) {
                    if (!user.payment_providers.some(ex => deepEqual(ex, p))) {
                        const r = await backend.add_user_payment_provider(user.id, sessionToken, p);
                        if ('Err' in r) {
                            setMessage(rampErrorToString(r.Err));
                            setLoadingAddProvider(false);
                            return;
                        }
                        added++;
                    }
                }
                if (added > 0) refetchUser();
                else setMessage('Duplicate payment provider');
            }
        } catch (e: any) {
            setMessage(`Failed to update provider: ${rampErrorToString(e)}`);
        } finally {
            setLoadingAddProvider(false);
        }
    };

    const handleRemoveProvider = async (provider: PaymentProvider) => {
        if (!sessionToken) throw new Error("Please authenticate to get a token session");

        setRemoving(true);
        try {
            const result = await backend.remove_user_payment_provider(user.id, sessionToken, provider);
            if ('Ok' in result) {
                refetchUser();
            } else {
                setMessage(rampErrorToString(result.Err));
            }
        } catch (error) {
            setMessage(`Failed to update provider: ${error}`);
        } finally {
            setRemoving(false);
        }
    }

    const handleRemoveEvmGroup = async (g: EvmGroup) => {
        if (!sessionToken) return;
        setRemoving(true);
        try {
            for (const p of g.providers) {
                const r = await backend.remove_user_payment_provider(user.id, sessionToken, p);
                if ('Err' in r) throw new Error(rampErrorToString(r.Err));
            }
            refetchUser();
        } catch (e: any) {
            setMessage(e?.message ?? 'Failed to remove group');
        } finally {
            setRemoving(false);
        }
    };

    const handleAddAddress = async (addressToAdd: string) => {
        if (!sessionToken) throw new Error("Please authenticate to get a token session")

        if (!selectedAddressType) return;
        setLoadingAddAddress(true);

        const addingAddress = {
            address_type: { [selectedAddressType]: null },
            address: addressToAdd
        } as TransactionAddress;

        try {
            const result = await backend.add_user_transaction_address(user.id, sessionToken, addingAddress);
            if ('Ok' in result) {
                refetchUser();
            } else {
                setMessage(`Failed to update address: ${rampErrorToString(result.Err)}`)
            }
        } catch (error) {
            setMessage(`Failed to update address: ${error}`);
        } finally {
            setLoadingAddAddress(false);
        }
    };

    const isAddressInUserAddresses = (addressToCheck: string): boolean => {
        return user.addresses.some(addr => addr.address === addressToCheck);
    };

    const isSameAddress = (addr: TransactionAddress) => {
        if ('EVM' in user.login && 'EVM' in addr.address_type) {
            return user.login.EVM.address === addr.address;
        } else if ('ICP' in user.login && 'ICP' in addr.address_type) {
            return user.login.ICP.principal_id === addr.address;
        } else if ('Bitcoin' in user.login && 'Bitcoin' in addr.address_type) {
            return user.login.Bitcoin.address === addr.address;
        } else if ('Solana' in user.login && 'Solana' in addr.address_type) {
            return user.login.Solana.address === addr.address;
        }
        return false;
    };

    const addButtonContent = (loadingButton: boolean) =>
        loadingButton ? (
            <div className="w-5 h-5 px-3 py-3 border-t border-b rounded-full animate-spin"></div>
        ) : (
            "Add"
        )

    const tabClasses = ((tab: string) => clsx(
        "px-4 py-2 rounded-md",
        activeTab === tab ? "bg-gray-300 dark:bg-gray-800" : "bg-gray-200 dark:bg-gray-700"
    ));

    return (
        <>
            <div className="mb-4 text-gray-600 dark:text-white mx-auto px-8 max-w-lg">
                <button
                    className={tabClasses('profile')}
                    onClick={() => setActiveTab('profile')}
                >
                    Profile
                </button>
                <button
                    className={tabClasses('balances')}
                    onClick={() => setActiveTab('balances')}
                >
                    Balances
                </button>
            </div>
            <div className="bg-white dark:bg-gray-800 rounded-xl p-8 max-w-lg mx-auto shadow-lg relative">
                <button
                    className={clsx(
                        "absolute top-4 right-4 p-2 rounded-full flex items-center justify-center",
                        "bg-gray-100 dark:bg-gray-700 transition duration-200 ease-in-out",
                        isClicked ? 'outline outline-2 outline-blue-500' : 'hover:bg-gray-200 dark:hover:bg-gray-600'
                    )}
                    onClick={handleRefresh}
                    title="Refresh Profile"
                    disabled={isClicked}
                >
                    <FontAwesomeIcon icon={faSync} spin={isClicked} />
                </button>

                {activeTab === 'profile' ? (
                    <>
                        <div className="text-center">
                            <h2 className="text-2xl font-semibold">Profile</h2>
                        </div>

                        <div className="space-y-4">
                            <div className="space-y-2">
                                <div className="flex justify-between items-center">
                                    <span className="font-medium text-gray-600 dark:text-gray-200">User Type:</span>
                                    <span className="font-semibold">{userTypeToString(user.user_type)}</span>
                                </div>

                                <div className="flex justify-between items-center">
                                    <span className="font-medium text-gray-600 dark:text-gray-200">Score:</span>
                                    <span className={`font-semibold ${user.score > 0 ? "text-green-400" : "text-red-400"}`}>{user.score}</span>
                                </div>

                                <div className="flex justify-between items-center">
                                    <span className="font-medium text-gray-600 dark:text-gray-200">Preferred currency:</span>
                                    <CurrencySelect
                                        selected={currency}
                                        onChange={setCurrency}
                                        className="w-auto text-sm border-gray-300 dark:border-gray-600"
                                        buttonClassName="rounded-md bg-gray-200 dark:bg-gray-800 hover:bg-gray-300 dark:hover:bg-gray-700 border-gray-300 dark:border-gray-600"
                                        dropdownClassName="bg-gray-200 dark:bg-gray-800 hover:bg-gray-300 dark:hover:bg-gray-700"
                                    />
                                </div>

                                {/* Ramped Amounts */}
                                {user.fiat_amounts.length > 0 && (
                                    <div className="flex justify-between items-start">
                                        <span className="font-medium text-gray-600 dark:text-gray-200">Ramped Amount:</span>
                                        <div className="space-y-2 flex flex-col items-end">
                                            {user.fiat_amounts.map(([currency, amount]) => (
                                                <div key={currency} className="flex items-center space-x-2">
                                                    <span className="font-semibold">{(Number(amount) / 100).toFixed(2)}</span>
                                                    <span className="border border-gray-600 dark:border-white bg-amber-600 rounded-full h-5 w-5 flex items-center justify-center text-sm leading-none">
                                                        <FontAwesomeIcon icon={CURRENCY_ICON_MAP[currency]} />
                                                    </span>
                                                </div>
                                            ))}
                                        </div>
                                    </div>
                                )}
                            </div>

                            <hr className="border-t border-gray-300 dark:border-gray-600 w-full" />

                            <div>
                                <div className="flex justify-between items-center">
                                    <span className="font-medium">Addresses:</span>
                                </div>
                                <ul className="pl-4 mt-2">
                                    {user.addresses.map((addr, index) => {
                                        const isEmail = 'Email' in addr.address_type;
                                        const addressType = Object.keys(addr.address_type)[0];
                                        const truncatedAddress = addr.address.length > 20 ? truncate(addr.address, 10, 10) : addr.address;
                                        const explorerUrl = getExplorerUrls(addressType, addr.address, addressType === 'EVM' && chainId ? BigInt(chainId) : undefined);
                                        return (
                                            <li key={index} className={`py-1 ${isSameAddress(addr) ? "text-blue-400" : "text-gray-200"}`}>
                                                <span className="flex-1 text-sm text-gray-700 dark:text-gray-300">({addressType})</span>
                                                <span className="ml-2 text-gray-500 dark:text-gray-100">
                                                    {explorerUrl ? (() => {
                                                        return (
                                                            <a href={explorerUrl.address} target="_blank" rel="noopener noreferrer">
                                                                {truncatedAddress}
                                                            </a>
                                                        );
                                                    })() : (
                                                        truncatedAddress
                                                    )}
                                                </span>
                                                <span className="relative">
                                                    {!isEmail && (
                                                        <button
                                                            className="ml-2 text-gray-400 hover:text-gray-200 "
                                                            title="Copy address"
                                                            onClick={() => copyToClipboard(addr.address, index)}
                                                        >
                                                            <FontAwesomeIcon icon={copiedIndex === index ? faCheckCircle : faCopy} />
                                                        </button>
                                                    )}
                                                    {copiedIndex === index && (
                                                        <span className={clsx(
                                                            "absolute left-8 -top-1.5 px-2 py-1 rounded-md shadow-md text-sm text-green-400 dark:text-green-200",
                                                            "bg-gray-200 dark:bg-gray-700 border border-gray-400 dark:border-gray-500"
                                                        )}>
                                                            Copied!
                                                        </span>
                                                    )}
                                                </span>
                                            </li>
                                        );
                                    })}
                                </ul>
                            </div >

                            <div className="flex gap-2 items-center justify-between w-full">
                                <div className="relative w-1/6" ref={dropdownRef}>
                                    <button
                                        className="w-full pl-3 pr-0.5 py-2 border border-gray-400 dark:border-gray-500 bg-gray-300 dark:bg-gray-600 rounded-md focus:outline-none flex items-center justify-between"
                                        onClick={() => setAddressDropdownOpen(!addressDropdownOpen)}
                                    >
                                        {selectedAddressType === 'EVM' ? (
                                            <img src={ethereumLogo} alt="Ethereum Logo" className="h-6 w-6 inline" />
                                        ) : selectedAddressType === 'ICP' ? (
                                            <img src={icpLogo} alt="ICP Logo" className="h-6 w-6 inline" />
                                        ) : selectedAddressType === 'Bitcoin' ? (
                                            <img src={bitcoinLogo} alt="Bitcoin Logo" className="h-6 w-6 inline" />
                                        ) : selectedAddressType === 'Solana' ? (
                                            <img src={solanaLogo} alt="Solana Logo" className="h-6 w-6 inline" />
                                        ) : <span>Select Address Type</span>}
                                        <svg
                                            className={`w-3 h-3 transition-transform ${addressDropdownOpen ? 'rotate-180' : ''}`}
                                            fill="none"
                                            stroke="currentColor"
                                            viewBox="0 0 24 24"
                                            xmlns="http://www.w3.org/2000/svg"
                                        >
                                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M19 9l-7 7-7-7"></path>
                                        </svg>
                                    </button>
                                    {addressDropdownOpen && (() => {
                                        const optionClass = "flex items-center px-3 py-2 hover:bg-gray-300 dark:hover:bg-gray-500 cursor-pointer";
                                        return (
                                            <div className="absolute bg-gray-100 dark:bg-gray-700 rounded-md mt-2 w-full shadow-lg z-10">
                                                <div
                                                    className={clsx(
                                                        optionClass,
                                                        selectedAddressType === 'EVM' ? 'bg-gray-300 dark:bg-gray-500' : ''
                                                    )}
                                                    onClick={() => handleAddressSelectOption('EVM')}
                                                >
                                                    <img src={ethereumLogo} alt="Ethereum Logo" className="h-6 w-6" />
                                                </div>
                                                <div
                                                    className={clsx(
                                                        optionClass,
                                                        selectedAddressType === 'ICP' ? 'bg-gray-300 dark:bg-gray-500' : ''
                                                    )}
                                                    onClick={() => handleAddressSelectOption('ICP')}
                                                >
                                                    <img src={icpLogo} alt="ICP Logo" className="h-6 w-6" />
                                                </div>
                                                <div
                                                    className={clsx(
                                                        optionClass,
                                                        selectedAddressType === 'Bitcoin' ? 'bg-gray-300 dark:bg-gray-500' : ''
                                                    )}
                                                    onClick={() => handleAddressSelectOption('Bitcoin')}
                                                >
                                                    <img src={bitcoinLogo} alt="Bitcoin Logo" className="h-6 w-6" />
                                                </div>
                                                <div
                                                    className={clsx(
                                                        optionClass,
                                                        selectedAddressType === 'Solana' ? 'bg-gray-300 dark:bg-gray-500' : ''
                                                    )}
                                                    onClick={() => handleAddressSelectOption('Solana')}
                                                >
                                                    <img src={solanaLogo} alt="Solana Logo" className="h-6 w-6" />
                                                </div>
                                            </div>
                                        );
                                    })()}
                                </div>
                                {selectedAddressType === 'EVM' ? (
                                    isConnected ? (
                                        <div className="flex-grow flex items-center">
                                            <input
                                                type="text"
                                                value={address}
                                                readOnly
                                                className="px-3 py-2 border border-gray-400 dark:border-gray-500 w-full rounded-md bg-gray-300 dark:bg-gray-600"
                                            />
                                            <button
                                                disabled={!address || isAddressInUserAddresses(address)}
                                                onClick={() => handleAddAddress(address!)}
                                                className={clsx(
                                                    "ml-2 px-4 py-2 font-semibold rounded-md w-1/4 flex justify-center items-center",
                                                    !address || isAddressInUserAddresses(address) ? 'bg-gray-400 dark:bg-gray-500 cursor-not-allowed' : 'bg-indigo-300 dark:bg-indigo-700 hover:bg-indigo-200 dark:hover:bg-indigo-800'
                                                )}
                                            >
                                                {addButtonContent(loadingAddAddress)}
                                            </button>
                                        </div>
                                    ) : (
                                        <div className="flex-grow">
                                            <ConnectButton.Custom>
                                                {({ openConnectModal }) => (
                                                    <button
                                                        className="w-full text-lg bg-amber-200 dark:bg-amber-800 hover:bg-amber-100 dark:hover:bg-amber-900 cursor-pointer px-3 py-2 rounded-md"
                                                        onClick={openConnectModal}
                                                    >
                                                        Connect wallet
                                                    </button>
                                                )}
                                            </ConnectButton.Custom>
                                        </div>

                                    )
                                ) : selectedAddressType === 'ICP' ? (
                                    (principal !== null) ? (
                                        <div className="flex-grow flex items-center">
                                            <input
                                                type="text"
                                                value={principal.toString()}
                                                readOnly
                                                className="px-3 py-2 border border-gray-400 dark:border-gray-500 w-full rounded-md bg-gray-300 dark:bg-gray-600"
                                            />
                                            <button
                                                disabled={isAddressInUserAddresses(principal.toString()) || loadingAddAddress}
                                                onClick={() => handleAddAddress(principal.toString())}
                                                className={clsx(
                                                    "ml-2 px-4 py-2  w-1/4 font-semibold rounded-md flex justify-center items-center",
                                                    !principal || isAddressInUserAddresses(principal.toString())
                                                        ? 'bg-gray-400 dark:bg-gray-500 cursor-not-allowed'
                                                        : 'bg-indigo-200 dark:bg-indigo-700 hover:bg-indigo-100 dark:hover:bg-indigo-800',
                                                    loadingAddAddress ? 'cursor-not-allowed' : ''
                                                )}>
                                                {addButtonContent(loadingAddAddress)}
                                            </button>
                                        </div>
                                    ) : (
                                        <div className="flex-grow">
                                            <button
                                                onClick={handleInternetIdentityLogin}
                                                className="px-4 py-2 bg-amber-200 dark:bg-amber-800 text-lg font-bold rounded-md cursor-pointer w-full"
                                            >
                                                Connect ICP
                                            </button>
                                        </div>
                                    )
                                ) : selectedAddressType === 'Bitcoin' ? (
                                    bitcoinAddress ? (
                                        <div className="flex-grow flex items-center">
                                            <input
                                                type="text"
                                                value={bitcoinAddress}
                                                readOnly
                                                className="px-3 py-2 border border-gray-400 dark:border-gray-500 w-full rounded-md bg-gray-300 dark:bg-gray-600"
                                            />
                                            <button
                                                disabled={isAddressInUserAddresses(bitcoinAddress) || loadingAddAddress}
                                                onClick={() => handleAddAddress(bitcoinAddress)}
                                                className={clsx(
                                                    "ml-2 px-4 py-2  w-1/4 font-semibold rounded-md flex justify-center items-center",
                                                    !bitcoinAddress || isAddressInUserAddresses(bitcoinAddress)
                                                        ? 'bg-gray-400 dark:bg-gray-500 cursor-not-allowed'
                                                        : 'bg-indigo-200 dark:bg-indigo-700 hover:bg-indigo-100 dark:hover:bg-indigo-800',
                                                    loadingAddAddress ? 'cursor-not-allowed' : ''
                                                )}>
                                                {addButtonContent(loadingAddAddress)}
                                            </button>
                                        </div>
                                    ) : (
                                        <div className="flex-grow">
                                            <button
                                                onClick={handleConnectUnisat}
                                                disabled={loadingUnisat}
                                                className={clsx(
                                                    "px-4 py-2 font-bold rounded-md w-full cursor-pointer text-lg",
                                                    loadingUnisat
                                                        ? 'bg-gray-400 dark:bg-gray-500 cursor-not-allowed'
                                                        : 'bg-indigo-200 dark:bg-indigo-700 hover:bg-indigo-100 dark:hover:bg-indigo-800'
                                                )}>
                                                {loadingUnisat ? "Connecting..." : "Connect Unisat"}
                                            </button>
                                        </div>
                                    )
                                ) : selectedAddressType === 'Solana' ? (
                                    solanaPubkey ? (
                                        <div className="flex-grow flex items-center">
                                            <input
                                                type="text"
                                                value={solanaPubkey}
                                                readOnly
                                                className="px-3 py-2 border border-gray-400 dark:border-gray-500 w-full rounded-md bg-gray-300 dark:bg-gray-600"
                                            />
                                            <button
                                                disabled={isAddressInUserAddresses(solanaPubkey) || loadingAddAddress}
                                                onClick={() => handleAddAddress(solanaPubkey)}
                                                className={clsx(
                                                    "ml-2 px-4 py-2  w-1/4 font-semibold rounded-md flex justify-center items-center",
                                                    !solanaPubkey || isAddressInUserAddresses(solanaPubkey)
                                                        ? 'bg-gray-400 dark:bg-gray-500 cursor-not-allowed'
                                                        : 'bg-indigo-200 dark:bg-indigo-700 hover:bg-indigo-100 dark:hover:bg-indigo-800',
                                                    loadingAddAddress ? 'cursor-not-allowed' : ''
                                                )}>
                                                {addButtonContent(loadingAddAddress)}
                                            </button>
                                        </div>
                                    ) : (
                                        <div className="flex-grow">
                                            <button
                                                onClick={connectSolana}
                                                className={clsx(
                                                    "px-4 py-2 font-bold rounded-md w-full cursor-pointer text-lg",
                                                    'bg-indigo-200 dark:bg-indigo-700 hover:bg-indigo-100 dark:hover:bg-indigo-800'
                                                )}>
                                                Connect Solana
                                            </button>
                                        </div>
                                    )
                                ) : null}
                            </div>

                            <hr className="border-t border-gray-300 dark:border-gray-600 w-full" />

                            <div>
                                <div className="flex justify-between items-center">
                                    <span className="font-medium">Payment Providers:</span>
                                </div>

                                <div className="mt-3 grid grid-cols-1 gap-3">
                                    {user.payment_providers
                                        .sort((_, b) => ('PayPal' in b ? 1 : -1))
                                        .map((provider, index) => {
                                            const baseCard =
                                                "relative rounded-xl border p-3 bg-gray-300/40 dark:bg-gray-800/60 border-gray-500/40";
                                            const stripeCard =
                                                "relative rounded-xl border p-3 bg-purple-500/10 border-purple-500/50";
                                            const badgeBase =
                                                "text-[10px] uppercase tracking-wide px-2 py-0.5 rounded border";
                                            const badgeGray = "border-gray-400/30 text-gray-400 bg-gray-400/10";
                                            const badgePurple =
                                                "border-purple-400/30 text-purple-300 bg-purple-400/10";
                                            const badgeBlue =
                                                "border-sky-400/30 text-sky-300 bg-sky-400/10";

                                            // --- Render per type ---
                                            if ("PayPal" in provider) {
                                                return (
                                                    <div key={index} className={baseCard}>
                                                        <div className="flex items-start justify-between gap-3">
                                                            <div className="flex items-center gap-2">
                                                                <ProviderIcon type="PayPal" />
                                                                <span className={`${badgeBase} ${badgeGray}`}>PayPal</span>
                                                            </div>
                                                            <button
                                                                className="text-red-600/80 dark:text-red-400 text-sm w-7 h-7 rounded-full border border-white/30 flex items-center justify-center hover:bg-red-500/10 transition"
                                                                title="remove"
                                                                aria-label="remove PayPal provider"
                                                                onClick={() => handleRemoveProvider(provider)}
                                                                disabled={removing}
                                                            >
                                                                {removing ? (
                                                                    <FontAwesomeIcon icon={faSpinner} spin />
                                                                ) : (
                                                                    <FontAwesomeIcon icon={faRemove} />
                                                                )}
                                                            </button>
                                                        </div>
                                                        <div className="mt-2 font-mono text-sm break-all text-gray-800 dark:text-gray-200">
                                                            {provider.PayPal.id}
                                                        </div>
                                                    </div>
                                                );
                                            }

                                            if ("Revolut" in provider) {
                                                return (
                                                    <div key={index} className={baseCard}>
                                                        <div className="flex items-start justify-between gap-3">
                                                            <div className="flex items-center gap-2">
                                                                <ProviderIcon type="Revolut" />
                                                                <span className={`${badgeBase} ${badgeBlue}`}>Revolut</span>
                                                            </div>
                                                            <button
                                                                className="text-red-600/80 dark:text-red-400 text-sm w-7 h-7 rounded-full border border-white/30 flex items-center justify-center hover:bg-red-500/10 transition"
                                                                title="remove"
                                                                aria-label="remove Revolut provider"
                                                                onClick={() => handleRemoveProvider(provider)}
                                                                disabled={removing}
                                                            >
                                                                {removing ? (
                                                                    <FontAwesomeIcon icon={faSpinner} spin />
                                                                ) : (
                                                                    <FontAwesomeIcon icon={faRemove} />
                                                                )}
                                                            </button>
                                                        </div>
                                                        <div className="mt-2 font-mono text-sm break-all text-gray-800 dark:text-gray-200">
                                                            {provider.Revolut.id}
                                                        </div>
                                                        <div className="text-xs text-gray-600 dark:text-gray-400">
                                                            Scheme: {provider.Revolut.scheme}
                                                        </div>
                                                        {provider.Revolut.name?.[0] && (
                                                            <div className="text-xs text-gray-600 dark:text-gray-400">
                                                                Name: {provider.Revolut.name[0]}
                                                            </div>
                                                        )}
                                                    </div>
                                                );
                                            }

                                            if ("Stripe" in provider) {
                                                return (
                                                    <div key={index} className={stripeCard}>
                                                        <div className="flex items-start justify-between gap-3">
                                                            <div className="flex items-center gap-2">
                                                                <ProviderIcon type="Stripe" />
                                                                <span className={`${badgeBase} ${badgePurple}`}>Stripe</span>
                                                            </div>
                                                            <button
                                                                className="text-red-600/80 dark:text-red-400 text-sm w-7 h-7 rounded-full border border-white/20 flex items-center justify-center hover:bg-red-500/10 transition"
                                                                title="remove"
                                                                aria-label="remove Stripe provider"
                                                                onClick={() => handleRemoveProvider(provider)}
                                                                disabled={removing}
                                                            >
                                                                {removing ? (
                                                                    <FontAwesomeIcon icon={faSpinner} spin />
                                                                ) : (
                                                                    <FontAwesomeIcon icon={faRemove} />
                                                                )}
                                                            </button>
                                                        </div>
                                                        <div className="mt-2 font-mono text-sm break-all text-purple-200">
                                                            {provider.Stripe.account_id}
                                                        </div>
                                                        {"platform" in provider.Stripe && provider.Stripe.platform && (
                                                            <div className="text-xs text-purple-300/80">
                                                                Platform: {provider.Stripe.platform}
                                                            </div>
                                                        )}
                                                    </div>
                                                );
                                            }

                                            if ("Email" in provider) {
                                                return (
                                                    <div key={index} className={baseCard}>
                                                        <div className="flex items-start justify-between gap-3">
                                                            <div className="flex items-center gap-2">
                                                                <ProviderIcon type="Email" />
                                                                <span className={`${badgeBase} ${badgeBlue}`}>Email</span>
                                                            </div>
                                                            <button
                                                                className="text-red-600/80 dark:text-red-400 text-sm w-7 h-7 rounded-full border border-white/30 flex items-center justify-center hover:bg-red-500/10 transition"
                                                                title="remove"
                                                                aria-label="remove Email provider"
                                                                onClick={() => handleRemoveProvider(provider)}
                                                                disabled={removing}
                                                            >
                                                                {removing ? (
                                                                    <FontAwesomeIcon icon={faSpinner} spin />
                                                                ) : (
                                                                    <FontAwesomeIcon icon={faRemove} />
                                                                )}
                                                            </button>
                                                        </div>
                                                        <div className="mt-2 font-mono text-sm break-all text-gray-800 dark:text-gray-200">
                                                            {provider.Email.email}
                                                        </div>
                                                    </div>
                                                );
                                            }

                                            return null;
                                        })
                                    }

                                    {evmGroup.map((g, gIdx) => (
                                        <div key={`evm-${gIdx}`} className="relative rounded-xl border p-3 bg-emerald-500/10 border-emerald-500/40">
                                            <div className="flex items-start justify-between gap-3">
                                                <div className="flex items-center gap-2">
                                                    <ProviderIcon type="Crypto" crypto='EVM' />
                                                    <span className="text-[10px] uppercase tracking-wide px-2 py-0.5 rounded border border-emerald-400/30 text-emerald-300 bg-emerald-400/10">
                                                        Crypto
                                                    </span>
                                                    <img src={g.logo} alt={g.symbol.toString()} className="h-5 rounded-md w-auto" />
                                                </div>
                                                <button
                                                    className="text-red-600/80 dark:text-red-400 text-sm w-7 h-7 rounded-full border border-white/30 flex items-center justify-center hover:bg-red-500/10 transition"
                                                    title="remove"
                                                    aria-label="remove EVM crypto group"
                                                    onClick={() => handleRemoveEvmGroup(g)}
                                                    disabled={removing}
                                                >
                                                    {removing ? <FontAwesomeIcon icon={faSpinner} spin /> : <FontAwesomeIcon icon={faRemove} />}
                                                </button>
                                            </div>
                                            <div className="mt-2 font-mono text-sm break-all text-gray-800 dark:text-gray-200">
                                                {g.address.address}
                                            </div>
                                            <div className="mt-2 flex flex-wrap gap-2">
                                                {g.chains.map(cid => (
                                                    <span key={cid} className="text-[11px] px-2 py-0.5 rounded border border-emerald-400/20 text-emerald-200/90 bg-emerald-400/5">
                                                        {Object.values(NetworkIds).find(n => n.id === cid)?.name ?? cid}
                                                    </span>
                                                ))}
                                            </div>
                                        </div>
                                    ))}
                                    {solCrypto.map((p, sIdx) => {
                                        const mint = "Crypto" in p && "Solana" in p.Crypto.asset ? p.Crypto.asset.Solana.spl_token?.[0] ?? '' : 0;
                                        const symbol = solRegistry?.[mint]?.symbol ?? 'Unknown';
                                        return (
                                            <div key={`sol-${sIdx}`} className="relative rounded-xl border p-3 bg-fuchsia-500/10 border-fuchsia-500/40">
                                                <div className="flex items-start justify-between gap-3">
                                                    <div className="flex items-center gap-2">
                                                        <ProviderIcon type="Crypto" crypto='Solana' />
                                                        <span className="text-[10px] uppercase tracking-wide px-2 py-0.5 rounded border border-fuchsia-400/30 text-fuchsia-300 bg-fuchsia-400/10">
                                                            Crypto
                                                        </span>
                                                        <img src={SPL_TOKEN_LOGOS[symbol]} alt={mint.toString()} className="h-5 rounded-md w-auto" />
                                                    </div>
                                                    <button
                                                        className="text-red-600/80 dark:text-red-400 text-sm w-7 h-7 rounded-full border border-white/30 flex items-center justify-center hover:bg-red-500/10 transition"
                                                        title="remove"
                                                        aria-label="remove Solana crypto"
                                                        onClick={() => handleRemoveProvider(p)}
                                                        disabled={removing}
                                                    >
                                                        {removing ? <FontAwesomeIcon icon={faSpinner} spin /> : <FontAwesomeIcon icon={faRemove} />}
                                                    </button>
                                                </div>
                                                <div className="mt-2 font-mono text-sm break-all text-gray-800 dark:text-gray-200">
                                                    {"Crypto" in p && p.Crypto.address.address}
                                                </div>
                                            </div>
                                        );
                                    })}
                                    {icpCrypto.map((p, iIdx) => {
                                        const principal = "Crypto" in p && "ICP" in p.Crypto.asset ? p.Crypto.asset.ICP.ledger_principal : "";
                                        const token = ICP_TOKENS.find(t => t.address === principal);
                                        const symbol = token?.name ?? 'ckUSD';
                                        return (
                                            <div key={`icp-${iIdx}`} className="relative rounded-xl border p-3 bg-cyan-500/10 border-cyan-500/40">
                                                <div className="flex items-start justify-between gap-3">
                                                    <div className="flex items-center gap-2">
                                                        <ProviderIcon type="Crypto" crypto='ICP' />
                                                        <span className="text-[10px] uppercase tracking-wide px-2 py-0.5 rounded border border-cyan-400/30 text-cyan-300 bg-cyan-400/10">
                                                            Crypto
                                                        </span>
                                                        <img src={ICP_TOKENS.find((t) => t.name.toUpperCase() === symbol.toUpperCase())?.logo} alt={symbol} className="h-5 rounded-md w-auto" />
                                                    </div>
                                                    <button
                                                        className="text-red-600/80 dark:text-red-400 text-sm w-7 h-7 rounded-full border border-white/30 flex items-center justify-center hover:bg-red-500/10 transition"
                                                        title="remove"
                                                        aria-label="remove ICP crypto"
                                                        onClick={() => handleRemoveProvider(p)}
                                                        disabled={removing}
                                                    >
                                                        {removing ? <FontAwesomeIcon icon={faSpinner} spin /> : <FontAwesomeIcon icon={faRemove} />}
                                                    </button>
                                                </div>
                                                <div className="mt-2 font-mono text-sm break-all text-gray-800 dark:text-gray-200">
                                                    {"Crypto" in p && p.Crypto.address.address}
                                                </div>
                                            </div>
                                        );
                                    })}

                                </div>
                            </div>

                            <div className="flex gap-2 text-black dark:text-white">
                                <select
                                    value={providerType}
                                    onChange={(e) => setProviderType(e.target.value as PaymentProviderTypes)}
                                    className="w-1/2 px-3 py-2 border border-gray-200 dark:border-gray-500 bg-gray-300 dark:bg-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-100 dark:focus:ring-blue-900"
                                >
                                    <option value="" disabled selected>Select Provider</option>
                                    {visibleProviderTypes.map(type => (<option key={type} value={type}>{type}</option>))}
                                </select>

                                {providerType !== 'Crypto' && (
                                    <input
                                        type="text"
                                        value={providerId}
                                        onChange={(e) => setProviderId(e.target.value)}
                                        placeholder="Email"
                                        className="w-full px-3 py-2 border border-gray-200 dark:border-gray-500 bg-gray-300 dark:bg-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-100 dark:focus:ring-blue-900"
                                    />
                                )}

                                {providerType === 'Revolut' && (
                                    <>
                                        <select
                                            value={revolutScheme}
                                            onChange={(e) => setRevolutScheme(e.target.value as revolutSchemeTypes)}
                                            className="w-full px-3 py-2 border border-gray-200 dark:border-gray-500 bg-gray-300 dark:bg-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-100 dark:focus:ring-blue-900"
                                        >
                                            <option value="" selected>Scheme</option>
                                            {revolutSchemes.map(type => (
                                                <option value={type}>{type}</option>
                                            ))}
                                        </select>
                                        {userTypeToString(user.user_type) === 'Offramper' && (
                                            <input
                                                type="text"
                                                value={revolutName}
                                                onChange={(e) => setRevolutName(e.target.value)}
                                                placeholder="Name"
                                                className="w-full px-3 py-2 border border-gray-200 dark:border-gray-500 bg-gray-300 dark:bg-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-100 dark:focus:ring-blue-900"
                                            />
                                        )}
                                    </>
                                )}

                                {providerType === 'Stripe' && userTypeToString(user.user_type) === 'Offramper' && (
                                    <>
                                        <input
                                            type="text"
                                            value={stripeCountry}
                                            onChange={(e) => setStripeCountry(e.target.value.toUpperCase())}
                                            placeholder="Country (ES, US, …)"
                                            className="w-full px-3 py-2 border border-gray-200 dark:border-gray-500 bg-gray-300 dark:bg-gray-600 rounded-md focus:outline-none"
                                        />
                                        <button
                                            disabled={loadingStripe || !stripeCountry || !providerId}
                                            onClick={startStripeOnboardingProfile}
                                            className={clsx(
                                                "px-4 py-2 font-medium rounded-md text-white",
                                                "bg-purple-600 dark:bg-purple-800 hover:bg-purple-500 dark:hover:bg-purple-900",
                                                (loadingStripe || !stripeCountry || !providerId) && "opacity-60 cursor-not-allowed"
                                            )}
                                        >
                                            {loadingStripe ? 'Loading…' : 'Onboard'}
                                        </button>
                                    </>
                                )}

                                {/* ----- crypto “Add” buttons (one per family) ----- */}
                                {providerType === 'Crypto' && (
                                    <div className="relative w-2/3" ref={cryptoDropdownRef}>
                                        <button
                                            className="w-full pl-3 pr-0.5 py-2 border border-gray-400 dark:border-gray-500 bg-gray-300 dark:bg-gray-600 rounded-md focus:outline-none flex items-center justify-between"
                                            onClick={() => setCryptoDropdownOpen(!cryptoDropdownOpen)}
                                        >
                                            {selectedCryptoType === 'evm' ? (
                                                <div className="flex gap-2">
                                                    <img src={ethereumLogo} alt="Ethereum Logo" className="h-6 w-6 inline" />
                                                    <span>{truncate(getRecvAddr(user.addresses, 'EVM')?.address ?? "", 5, 5)}</span>
                                                </div>
                                            ) : selectedCryptoType === 'sol' ? (
                                                <div className="flex gap-2">
                                                    <img src={solanaLogo} alt="Solana Logo" className="h-6 w-6 inline" />
                                                    <span>{truncate(getRecvAddr(user.addresses, 'Solana')?.address ?? "", 5, 5)}</span>
                                                </div>
                                            ) : selectedCryptoType === 'icp' ? (
                                                <div className="flex gap-2">
                                                    <img src={icpLogo} alt="ICP Logo" className="h-6 w-6 inline" />
                                                    <span>{truncate(getRecvAddr(user.addresses, 'ICP')?.address ?? "", 5, 5)}</span>
                                                </div>
                                            ) : (
                                                <span>Select Blockchain</span>
                                            )}
                                            <svg
                                                className={`w-3 h-3 transition-transform ${cryptoDropdownOpen ? 'rotate-180' : ''}`}
                                                fill="none"
                                                stroke="currentColor"
                                                viewBox="0 0 24 24"
                                                xmlns="http://www.w3.org/2000/svg"
                                            >
                                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M19 9l-7 7-7-7"></path>
                                            </svg>
                                        </button>
                                        {cryptoDropdownOpen && (
                                            <div className="absolute bg-gray-100 dark:bg-gray-700 rounded-md mt-2 w-full shadow-lg z-10">
                                                {hasEvm(user.addresses) && (
                                                    <div
                                                        className={clsx(
                                                            "flex gap-2 items-center px-3 py-2 hover:bg-gray-300 dark:hover:bg-gray-500 cursor-pointer",
                                                            selectedCryptoType === 'evm' ? 'bg-gray-300 dark:bg-gray-500' : ''
                                                        )}
                                                        onClick={() => handleCryptoSelectOption('evm')}
                                                    >
                                                        <img src={ethereumLogo} alt="Ethereum Logo" className="h-6 w-6" />
                                                        <span>{truncate(getRecvAddr(user.addresses, 'EVM')?.address ?? "", 5, 5)}</span>
                                                    </div>
                                                )}
                                                {hasSol(user.addresses) && (
                                                    <div
                                                        className={clsx(
                                                            "flex gap-2 items-center px-3 py-2 hover:bg-gray-300 dark:hover:bg-gray-500 cursor-pointer",
                                                            selectedCryptoType === 'sol' ? 'bg-gray-300 dark:bg-gray-500' : ''
                                                        )}
                                                        onClick={() => handleCryptoSelectOption('sol')}
                                                    >
                                                        <img src={solanaLogo} alt="Solana Logo" className="h-6 w-6" />
                                                        <span>{truncate(getRecvAddr(user.addresses, 'Solana')?.address ?? "", 5, 5)}</span>
                                                    </div>
                                                )}
                                                {hasIcp(user.addresses) && currency === 'USD' && (
                                                    <div
                                                        className={clsx(
                                                            "flex gap-2 items-center px-3 py-2 hover:bg-gray-300 dark:hover:bg-gray-500 cursor-pointer",
                                                            selectedCryptoType === 'icp' ? 'bg-gray-300 dark:bg-gray-500' : ''
                                                        )}
                                                        onClick={() => handleCryptoSelectOption('icp')}
                                                    >
                                                        <img src={icpLogo} alt="ICP Logo" className="h-6 w-6" />
                                                        <span>{truncate(getRecvAddr(user.addresses, 'ICP')?.address ?? "", 5, 5)}</span>
                                                    </div>
                                                )}
                                            </div>
                                        )}
                                    </div>
                                )}

                                {providerType !== 'Stripe' && (
                                    <button
                                        disabled={loadingAddProvider}
                                        onClick={handleAddProvider}
                                        className={clsx(
                                            "px-4 py-2 font-medium rounded-md text-black dark:text-white",
                                            "bg-indigo-300 dark:bg-indigo-700 hover:bg-indigo-200 dark:hover:bg-indigo-800",
                                            loadingAddProvider ? 'cursor-not-allowed' : ''
                                        )}
                                    >
                                        {addButtonContent(loadingAddProvider)}
                                    </button>
                                )}
                            </div>

                            <hr className="border-t border-gray-300 dark:border-gray-500 w-full" />

                            {message && <p className="text-sm font-medium text-red-600 break-all">{message}</p>}
                        </div>
                    </>
                ) : (
                    <BalancesDashboard />
                )}
            </div>
        </>
    );
};

export default UserProfile;
