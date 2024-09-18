import React, { useEffect, useRef, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAccount } from 'wagmi';
import { ConnectButton } from '@rainbow-me/rainbowkit';

import { backend } from '../../declarations/backend';
import { PaymentProvider, TransactionAddress } from '../../declarations/backend/backend.did';
import { userTypeToString } from '../../model/utils';
import { PaymentProviderTypes, providerTypes, revolutSchemeTypes, revolutSchemes } from '../../model/types';
import { truncate } from '../../model/helper';
import { isSessionExpired, saveUserSession } from '../../model/session';
import { rampErrorToString } from '../../model/error';
import { useUser } from './UserContext';

import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faSyncAlt, faSync } from '@fortawesome/free-solid-svg-icons';
import icpLogo from "../../assets/blockchains/icp-logo.svg";
import ethereumLogo from "../../assets/blockchains/ethereum-logo.png";

const UserProfile: React.FC = () => {
    const [providerType, setProviderType] = useState<PaymentProviderTypes>();
    const [providerId, setProviderId] = useState('');
    const [selectedAddressType, setSelectedAddressType] = useState<'ICP' | 'EVM'>('EVM');
    const [addressDropdownOpen, setAddressDropdownOpen] = useState(false);
    const [revolutScheme, setRevolutScheme] = useState<revolutSchemeTypes>('UK.OBIE.SortCodeAccountNumber');
    const [revolutName, setRevolutName] = useState('');
    const [message, setMessage] = useState('');
    const [loadingAddAddress, setLoadingAddAddress] = useState(false);
    const [loadingAddProvider, setLoadingAddProvider] = useState(false);

    const { address, isConnected } = useAccount();
    const { user, sessionToken, principal, setUser, loginInternetIdentity, refetchUser, logout } = useUser();
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

    const handleAddressSelectOption = (addressType: 'ICP' | 'EVM') => {
        setSelectedAddressType(addressType);
        setAddressDropdownOpen(false);
    };

    const handleInternetIdentityLogin = async () => {
        await loginInternetIdentity();
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

    return (
        <div className="bg-gray-700 rounded-xl p-8 max-w-lg mx-auto shadow-lg relative">
            <button
                className="absolute top-4 right-4 text-gray-200 p-2 rounded-full flex items-center justify-center hover:bg-gray-500 transition duration-200 ease-in-out"
                onClick={refetchUser}
                title="Refresh Profile"
            >
                <FontAwesomeIcon icon={faSync} />
            </button>

            <div className="text-center">
                <h2 className="text-white text-2xl font-semibold">Profile</h2>
            </div>

            <div className="space-y-4">
                <div className="text-white space-y-2">
                    <div className="flex justify-between items-center">
                        <span className="font-medium text-gray-200">User Type:</span>
                        <span className="font-semibold">{userTypeToString(user.user_type)}</span>
                    </div>
                    <div className="flex justify-between items-center">
                        <span className="font-medium text-gray-200">Ramped Amount:</span>
                        <span className="font-medium flex items-center space-x-2">
                            <span className="font-semibold">{(Number(user.fiat_amount) / 100).toFixed(2)}</span>
                            <span className="border border-white bg-amber-600 rounded-full h-5 w-5 flex items-center justify-center text-sm leading-none">
                                $
                            </span>
                        </span>
                    </div>
                    <div className="flex justify-between items-center">
                        <span className="font-medium text-gray-200">Score:</span>
                        <span className={`font-semibold ${user.score > 0 ? "text-green-400" : "text-red-400"}`}>{user.score}</span>
                    </div>
                </div>

                <hr className="border-t border-gray-500 w-full" />

                <div className="text-white">
                    <div className="flex justify-between items-center">
                        <span className="font-medium">Addresses:</span>
                    </div>
                    <ul className="pl-4 mt-2">
                        {user.addresses.map((addr, index) => {
                            return (
                                <li key={index} className={`py-1 ${isSameAddress(addr) ? "text-blue-400" : "text-gray-200"}`}>
                                    <span className="flex-1 text-sm text-gray-200">({Object.keys(addr.address_type)[0]})</span>
                                    <span className="ml-2">{addr.address.length > 20 ? truncate(addr.address, 10, 10) : addr.address}</span>
                                </li>
                            );
                        })}
                    </ul>
                </div >

                <div className="flex gap-2 items-center justify-between w-full">
                    <div className="relative w-1/6" ref={dropdownRef}>
                        <button
                            className="w-full pl-3 pr-0.5 py-2 border border-gray-500 bg-gray-600 text-white rounded-md focus:outline-none flex items-center justify-between"
                            onClick={() => setAddressDropdownOpen(!addressDropdownOpen)}
                        >
                            {selectedAddressType === 'EVM' ? (
                                <img src={ethereumLogo} alt="Ethereum Logo" className="h-6 w-6 inline" />
                            ) : selectedAddressType === 'ICP' ? (
                                <img src={icpLogo} alt="ICP Logo" className="h-6 w-6 inline" />
                            ) : <span>Select Address Type'</span>}
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
                        {addressDropdownOpen && (
                            <div className="absolute bg-gray-600 rounded-md mt-2 w-full shadow-lg z-10">
                                <div
                                    className="flex items-center px-3 py-2 hover:bg-gray-500 cursor-pointer"
                                    onClick={() => handleAddressSelectOption('EVM')}
                                >
                                    <img src={ethereumLogo} alt="Ethereum Logo" className="h-6 w-6" />
                                </div>
                                <div
                                    className="flex items-center px-3 py-2 hover:bg-gray-500 cursor-pointer"
                                    onClick={() => handleAddressSelectOption('ICP')}
                                >
                                    <img src={icpLogo} alt="ICP Logo" className="h-6 w-6" />
                                </div>
                            </div>
                        )}
                    </div>
                    {selectedAddressType === 'EVM' ? (
                        isConnected ? (
                            <div className="flex-grow flex items-center">
                                <input
                                    type="text"
                                    value={address}
                                    readOnly
                                    className="px-3 py-2 border border-gray-500 rounded-md bg-gray-600 w-full text-white"
                                />
                                <button
                                    disabled={!address || isAddressInUserAddresses(address)}
                                    onClick={() => handleAddAddress(address!)}
                                    className={
                                        `ml-2 px-4 py-2 text-white font-semibold rounded-md w-1/4 flex justify-center items-center 
                                    ${!address || isAddressInUserAddresses(address) ? 'bg-gray-500 cursor-not-allowed' : 'bg-indigo-700 hover:bg-indigo-800'}`
                                    }
                                >
                                    {addButtonContent(loadingAddAddress)}
                                </button>
                            </div>
                        ) : (
                            <div className="flex-grow">
                                <ConnectButton.Custom>
                                    {({ openConnectModal }) => (
                                        <button
                                            className="text-white w-full text-lg bg-amber-800 hover:bg-amber-900 cursor-pointer px-3 py-2 rounded-md"
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
                                    className="px-3 py-2 h-full border text-white border-gray-500 w-full rounded-md bg-gray-600"
                                />
                                <button
                                    disabled={isAddressInUserAddresses(principal.toString()) || (isAddressInUserAddresses(principal.toString()) || loadingAddAddress)}
                                    onClick={() => handleAddAddress(principal.toString())}
                                    className={`ml-2 px-4 py-2 text-white w-1/4 font-semibold rounded-md flex justify-center items-center ${!principal || isAddressInUserAddresses(principal.toString())
                                        ? 'bg-gray-500 cursor-not-allowed'
                                        : 'bg-indigo-700 hover:bg-indigo-800'} 
                                        ${loadingAddAddress ? 'cursor-not-allowed' : ''}`
                                    }>
                                    {addButtonContent(loadingAddAddress)}
                                </button>
                            </div>
                        ) : (
                            <div className="flex-grow">
                                <button
                                    onClick={handleInternetIdentityLogin}
                                    className="px-4 py-2 bg-amber-800 text-white text-lg font-bold rounded-md cursor-pointer w-full"
                                >
                                    Connect ICP
                                </button>
                            </div>
                        )
                    ) : null}
                </div>

                <hr className="border-t border-gray-500 w-full" />

                <div className="text-white">
                    <div className="flex justify-between items-center">
                        <span className="font-medium">Payment Providers:</span>
                    </div>
                    <ul className="pl-4 mt-2">
                        {user.payment_providers.map((provider, index) => {
                            if ('PayPal' in provider) {
                                return (
                                    <li key={index} className="py-">
                                        <span className="flex-1 text-sm text-gray-200">(PayPal)</span>
                                        <span className="ml-2">{provider.PayPal.id}</span>
                                    </li>
                                );
                            } else if ('Revolut' in provider) {
                                return (
                                    <li key={index} className="py-1">
                                        <span className="flex-1 text-sm text-gray-200">(Revolut)</span>
                                        <span className="ml-2">{provider.Revolut.id}</span>
                                        <div className="ml-2">
                                            <span>{provider.Revolut.scheme}</span>
                                            {provider.Revolut.name && provider.Revolut.name.length > 0 && (
                                                <div>Name: {provider.Revolut.name[0]}</div>
                                            )}
                                        </div>
                                    </li>
                                );
                            } else {
                                return null;
                            }
                        })}
                    </ul>
                </div>
                <div className="flex gap-2 text-white">
                    <select
                        value={providerType}
                        onChange={(e) => setProviderType(e.target.value as PaymentProviderTypes)}
                        className="w-1/2 px-3 py-2 border border-gray-500 bg-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-900"
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
                        className="w-full px-3 py-2 border border-gray-500 rounded-md bg-gray-600 focus:outline-none focus:ring-2 focus:ring-blue-900"
                    />

                    {providerType === 'Revolut' && (
                        <>
                            <select
                                value={revolutScheme}
                                onChange={(e) => setRevolutScheme(e.target.value as revolutSchemeTypes)}
                                className="w-full px-3 py-2 border border-gray-500 bg-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-900"
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
                                    className="w-full px-3 py-2 border border-gray-500 bg-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-900"
                                />
                            )}
                        </>
                    )}

                    <button
                        disabled={loadingAddProvider}
                        onClick={handleAddProvider}
                        className={`px-4 py-2 bg-indigo-700 text-white font-medium rounded-md hover:bg-indigo-800 ${loadingAddProvider
                            ? 'cursor-not-allowed' : ''}`
                        }>
                        {addButtonContent(loadingAddProvider)}
                    </button>
                </div>

                <hr className="border-t border-gray-500 w-full" />

                {message && <p className="text-sm font-medium text-red-600 break-all">{message}</p>}
            </div>
        </div>
    );
};

export default UserProfile;
