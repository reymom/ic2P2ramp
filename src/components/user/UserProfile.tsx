import React, { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useUser } from './UserContext';
import { userTypeToString } from '../../model/utils';
import { backend } from '../../declarations/backend';
import { PaymentProviderTypes, providerTypes, revolutSchemeTypes, revolutSchemes } from '../../model/types';
import { truncate } from '../../model/helper';
import { PaymentProvider, TransactionAddress } from '../../declarations/backend/backend.did';
import { rampErrorToString } from '../../model/error';
import { useAccount } from 'wagmi';
import { AuthClient } from '@dfinity/auth-client';
import { icpHost, iiUrl } from '../../model/icp';
import { ConnectButton } from '@rainbow-me/rainbowkit';
import { HttpAgent } from '@dfinity/agent';

const UserProfile: React.FC = () => {
    const [providerType, setProviderType] = useState<PaymentProviderTypes>();
    const [providerId, setProviderId] = useState('');
    const [selectedAddressType, setSelectedAddressType] = useState<'ICP' | 'EVM'>('EVM');
    const [revolutScheme, setRevolutScheme] = useState<revolutSchemeTypes>('UK.OBIE.SortCodeAccountNumber');
    const [revolutName, setRevolutName] = useState('');
    const [message, setMessage] = useState('');
    const [loadingAddAddress, setLoadingAddAddress] = useState(false);
    const [loadingAddProvider, setLoadingAddProvider] = useState(false);

    const { address, isConnected } = useAccount();

    const { user, sessionToken, principal, setUser, setIcpAgent, setPrincipal } = useUser();
    const navigate = useNavigate();

    useEffect(() => {
        if (!user) {
            navigate('/');
        }
    }, [user, navigate]);


    if (!user) {
        return null;
    }

    const handleInternetIdentityLogin = async () => {
        const authClient = await AuthClient.create();
        await authClient.login({
            identityProvider: iiUrl,
            onSuccess: async () => {
                const identity = authClient.getIdentity();
                const principal = identity.getPrincipal();
                setPrincipal(principal);

                const agent = new HttpAgent({ identity, host: icpHost });
                if (process.env.FRONTEND_ICP_ENV === 'test') {
                    agent.fetchRootKey();
                }
                setIcpAgent(agent);
            },
            onError: (error) => {
                console.error("Internet Identity login failed:", error);
            },
        });
    };

    const handleAddProvider = async () => {
        if (!sessionToken) throw new Error("Please authenticate to get a token session")

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
                const updatedProviders = [...user.payment_providers, newProvider]
                setUser({ ...user, payment_providers: updatedProviders });
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
                const updatedAddresses = [...user.addresses, addingAddress];
                setUser({ ...user, addresses: updatedAddresses });
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

    return (
        <div className="p-6 max-w-lg mx-auto">
            <h2 className="text-xl font-semibold mb-2">User Profile</h2>

            <div className="mb-4">
                <div className="flex justify-between items-center">
                    <span className="font-medium">User Type:</span>
                    <span className="text-gray-700">{userTypeToString(user.user_type)}</span>
                </div>
                <div className="flex justify-between items-center mt-2">
                    <span className="font-medium">Ramped Amount:</span>
                    <span className="text-gray-700">{(Number(user.fiat_amount) / 100).toFixed(2)} $</span>
                </div>
                <div className="flex justify-between items-center mt-2">
                    <span className="font-medium">Score:</span>
                    <span className="text-gray-700">{user.score}</span>
                </div>
            </div>

            <hr className="border-t border-gray-300 w-full my-4" />

            <div className="my-4">
                <div className="flex justify-between items-center">
                    <span className="font-medium">Addresses:</span>
                </div>
                <ul className="pl-4 mt-2">
                    {user.addresses.map((addr, index) => {
                        return (
                            <li key={index} className="py-1 text-gray-700" style={{
                                color: isSameAddress(addr) ? 'blue' : ''
                            }}>
                                <span className="flex-1 text-sm text-gray-500">({Object.keys(addr.address_type)[0]})</span>
                                <span className="ml-2">{addr.address.length > 20 ? truncate(addr.address, 10, 10) : addr.address}</span>
                            </li>
                        );
                    })}
                </ul>
            </div >

            <div className="flex mb-4 gap-2 items-center justify-between">
                <select
                    value={selectedAddressType}
                    onChange={(e) => setSelectedAddressType(e.target.value as 'ICP' | 'EVM')}
                    className="w-1/3 px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                >
                    <option value="EVM">EVM</option>
                    <option value="ICP">ICP</option>
                    {/* <option value="Solana">Solana</option> */}
                </select>
                {selectedAddressType === 'EVM' ? (
                    isConnected ? (
                        <>
                            <input
                                type="text"
                                value={address}
                                readOnly
                                className="ml-2 px-3 py-2 border rounded w-full bg-gray-100"
                            />
                            <button
                                disabled={!address || isAddressInUserAddresses(address)}
                                onClick={() => handleAddAddress(address!)}
                                className={`ml-2 px-4 py-2 bg-blue-600 text-white font-medium rounded-lg ${!address || isAddressInUserAddresses(address) ? 'bg-gray-400 cursor-not-allowed' : 'bg-blue-600 hover:bg-blue-700'}`}>
                                Add
                            </button>
                        </>
                    ) : (
                        <div>
                            <ConnectButton />
                        </div>

                    )
                ) : selectedAddressType === 'ICP' ? (
                    (principal !== null) ? (
                        <>
                            <input
                                type="text"
                                value={principal.toString()}
                                readOnly
                                className="ml-2 px-3 py-2 border rounded w-full bg-gray-100"
                            />
                            <button
                                disabled={isAddressInUserAddresses(principal.toString()) || (isAddressInUserAddresses(principal.toString()))}
                                onClick={() => handleAddAddress(principal.toString())}
                                className={`ml-2 px-4 py-2 bg-blue-600 text-white font-medium rounded-lg ${!principal || isAddressInUserAddresses(principal.toString()) ? 'bg-gray-400 cursor-not-allowed' : 'bg-blue-600 hover:bg-blue-700'}`}>
                                Add
                            </button>
                        </>
                    ) : (
                        <button
                            onClick={handleInternetIdentityLogin}
                            className="px-4 py-2 bg-blue-600 text-white font-bold rounded-xl"
                        >
                            Connect ICP
                        </button>
                    )
                ) : null}
            </div>

            {loadingAddAddress && (
                <div className="flex justify-center items-center space-x-2">
                    <div className="w-4 h-4 border-t-2 border-b-2 border-indigo-600 rounded-full animate-spin"></div>
                    <div className="text-sm font-medium text-gray-700">Adding Address...</div>
                </div>
            )}

            <hr className="border-t border-gray-300 w-full my-4" />

            <div className="my-4">
                <div className="flex justify-between items-center">
                    <span className="font-medium">Payment Providers:</span>
                </div>
                <ul className="pl-4 mt-2">
                    {user.payment_providers.map((provider, index) => {
                        if ('PayPal' in provider) {
                            return (
                                <li key={index} className="py-1 text-gray-700">
                                    <span className="flex-1 text-sm text-gray-500">(PayPal)</span>
                                    <span className="ml-2">{provider.PayPal.id}</span>
                                </li>
                            );
                        } else if ('Revolut' in provider) {
                            return (
                                <li key={index} className="py-1 text-gray-700">
                                    <span className="flex-1 text-sm text-gray-500">(Revolut)</span>
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
            <div className="flex mb-4 gap-2">
                <select
                    value={providerType}
                    onChange={(e) => setProviderType(e.target.value as PaymentProviderTypes)}
                    className="w-1/2 px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
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
                    className="w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                />

                {providerType === 'Revolut' && (
                    <>
                        <select
                            value={revolutScheme}
                            onChange={(e) => setRevolutScheme(e.target.value as revolutSchemeTypes)}
                            className="w-full px-3 py-2 border rounded"
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
                                className="w-full px-3 py-2 border rounded"
                            />
                        )}
                    </>
                )}

                <button onClick={handleAddProvider} className="px-4 py-2 bg-blue-600 text-white font-medium rounded-lg hover:bg-blue-700">
                    Add
                </button>
            </div>

            {loadingAddProvider && (
                <div className="flex justify-center items-center space-x-2">
                    <div className="w-4 h-4 border-t-2 border-b-2 border-indigo-600 rounded-full animate-spin"></div>
                    <div className="text-sm font-medium text-gray-700">Adding Provider...</div>
                </div>
            )}

            <hr className="border-t border-gray-300 w-full my-4" />

            {message && <p className="mt-4 text-sm font-medium text-grey-600 break-all">{message}</p>}
        </div >
    );
};

export default UserProfile;
