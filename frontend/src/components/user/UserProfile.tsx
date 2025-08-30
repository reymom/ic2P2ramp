import React, { useEffect, useRef, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAccount } from 'wagmi';
import clsx from 'clsx';
import { ConnectButton } from '@rainbow-me/rainbowkit';

import { PaymentProvider, TransactionAddress } from '@/declarations/icramp_backend/icramp_backend.did';
import { backend } from '@/model/backendProxy';
import { PaymentProviderTypes, providerTypes, revolutSchemeTypes, revolutSchemes } from '@/model/types';
import { isSessionExpired } from '@/model/session';
import { userTypeToString } from '@/model/helpers/types';
import { rampErrorToString } from '@/model/helpers/error';
import { CURRENCY_ICON_MAP } from '@/constants/currencyIconsMap';
import { truncate } from '@/utils/formatters';
import { getExplorerUrls } from '@/utils/explorers';
import CurrencySelect from '@/components/ui/CurrencySelect';
import BalancesDashboard from './BalanceDashboard';
import { useUser } from './UserContext';

import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faRemove, faSpinner, faSync, faCopy, faCheckCircle } from '@fortawesome/free-solid-svg-icons';
import icpLogo from "@/assets/blockchains/icp-logo.svg";
import ethereumLogo from "@/assets/blockchains/ethereum-logo.png";
import bitcoinLogo from "@/assets/blockchains/bitcoin-logo.svg";
import solanaLogo from "@/assets/blockchains/solana-logo.png"

const UserProfile: React.FC = () => {
    const [providerType, setProviderType] = useState<PaymentProviderTypes>();
    const [providerId, setProviderId] = useState('');
    const [selectedAddressType, setSelectedAddressType] = useState<'ICP' | 'EVM' | 'Bitcoin' | 'Solana'>('Bitcoin');
    const [addressDropdownOpen, setAddressDropdownOpen] = useState(false);
    const [revolutScheme, setRevolutScheme] = useState<revolutSchemeTypes>('UK.OBIE.SortCodeAccountNumber');
    const [revolutName, setRevolutName] = useState('');
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
        sessionToken,
        principal,
        bitcoinAddress,
        solanaPubkey,
        connectUnisat,
        connectSolana,
        loginInternetIdentity,
        refetchUser,
        setCurrency,
        logout
    } = useUser();
    const navigate = useNavigate();
    const dropdownRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        if (!user) navigate('/');
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

    const handleAddProvider = async () => {
        if (!sessionToken) throw new Error("Please authenticate to get a token session");

        if (!providerType) return;
        setLoadingAddProvider(true);

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
        } else {
            setMessage('Unknown payment provider');
            return;
        }

        try {
            const result = await backend.add_user_payment_provider(user.id, sessionToken, newProvider);
            if ('Ok' in result) {
                refetchUser();
            } else {
                setMessage(rampErrorToString(result.Err));
            }
        } catch (error) {
            setMessage(`Failed to update provider: ${error}`);
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
                                <ul className="pl-4 mt-2">
                                    {user.payment_providers
                                        .sort((_, b) => ('PayPal' in b ? 1 : -1))
                                        .map((provider, index) => {
                                            if ('PayPal' in provider) {
                                                return (
                                                    <li key={index} className="py-1 relative items-center">
                                                        <span className="flex-1 text-sm text-gray-700 dark:text-gray-300">(PayPal)</span>
                                                        <span className="ml-2 mr-6">{provider.PayPal.id}</span>
                                                        <span className="absolute right-0 my-1">
                                                            <button
                                                                className="text-red-400 text-sm ml-4 w-3 h-3 rounded-full p-2 border border-white border-opacity-40 flex items-center justify-center flex-shrink-0 hover:text-red-600 transition duration-200 ease-in-out shadow-md"
                                                                title="remove"
                                                                onClick={() => handleRemoveProvider(provider)}
                                                                disabled={removing}
                                                            >
                                                                {removing ? <FontAwesomeIcon icon={faSpinner} spin /> : <FontAwesomeIcon icon={faRemove} />}
                                                            </button>
                                                        </span>
                                                    </li>
                                                );
                                            } else if ('Revolut' in provider) {
                                                return (
                                                    <li key={index} className="py-1 relative items-center">
                                                        <div className="flex-1">
                                                            <span className="text-sm text-gray-700 dark:text-gray-300">(Revolut)</span>
                                                            <span className="ml-2 mr-6">{provider.Revolut.id}</span>
                                                        </div>
                                                        <div>{provider.Revolut.scheme}</div>
                                                        {provider.Revolut.name && provider.Revolut.name.length > 0 && (
                                                            <div>Name: {provider.Revolut.name[0]}</div>
                                                        )}
                                                        <span className="absolute right-0 top-1/2 transform -translate-y-1/2">
                                                            <button
                                                                className={clsx(
                                                                    "text-red-600 dark:text-red-400 text-sm ml-4 w-3 h-3 rounded-full p-2 border border-gray-600 dark:border-white border-opacity-40",
                                                                    "flex items-center justify-center flex-shrink-0 dark:hover:text-red-500 transition duration-200 ease-in-out shadow-md"
                                                                )}
                                                                title="remove"
                                                                onClick={() => handleRemoveProvider(provider)}
                                                                disabled={removing}
                                                            >
                                                                {removing ? <FontAwesomeIcon icon={faSpinner} spin /> : <FontAwesomeIcon icon={faRemove} />}
                                                            </button>
                                                        </span>
                                                    </li>
                                                );
                                            } else {
                                                return null;
                                            }
                                        })}
                                </ul>
                            </div>
                            <div className="flex gap-2 text-black dark:text-white">
                                <select
                                    value={providerType}
                                    onChange={(e) => setProviderType(e.target.value as PaymentProviderTypes)}
                                    className="w-1/2 px-3 py-2 border border-gray-200 dark:border-gray-500 bg-gray-300 dark:bg-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-100 dark:focus:ring-blue-900"
                                >
                                    <option value="" selected>Select Provider</option>
                                    {providerTypes.map(type => (
                                        <option value={type}>{type}</option>
                                    ))}
                                </select>
                                <input
                                    type="text"
                                    value={providerId}
                                    onChange={(e) => setProviderId(e.target.value)}
                                    placeholder="ID"
                                    className="w-full px-3 py-2 border border-gray-200 dark:border-gray-500 bg-gray-300 dark:bg-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-100 dark:focus:ring-blue-900"
                                />

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

                                <button
                                    disabled={loadingAddProvider}
                                    onClick={handleAddProvider}
                                    className={clsx(
                                        "px-4 py-2 font-medium rounded-md text-black dark:text-white",
                                        "bg-indigo-300 dark:bg-indigo-700 hover:bg-indigo-200 dark:hover:bg-indigo-800",
                                        loadingAddProvider ? 'cursor-not-allowed' : ''
                                    )}>
                                    {addButtonContent(loadingAddProvider)}
                                </button>
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
