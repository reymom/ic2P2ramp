import React, { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useUser } from '../UserContext';
import { userTypeToString } from '../model/utils';
import { backend } from '../declarations/backend';
import { AddressTypes, PaymentProviderTypes, providerTypes, revolutScheme, revolutSchemes } from '../model/types';
import { truncate } from '../model/helper';
import { Address, PaymentProvider } from '../declarations/backend/backend.did';
import { rampErrorToString } from '../model/error';

const UserProfile: React.FC = () => {
    const [providerType, setProviderType] = useState<PaymentProviderTypes>();
    const [providerId, setProviderId] = useState('');
    const [newAddress, setNewAddress] = useState('');
    const [selectedAddressType, setSelectedAddressType] = useState<AddressTypes>();
    const [revolutScheme, setRevolutScheme] = useState<revolutScheme>('UK.OBIE.SortCodeAccountNumber');
    const [revolutName, setRevolutName] = useState('');
    const [message, setMessage] = useState('');

    const { user, setUser } = useUser();
    const navigate = useNavigate();

    useEffect(() => {
        if (!user) {
            navigate('/');
        }
    }, [user, navigate]);


    if (!user) {
        return null;
    }

    const handleAddProvider = async () => {
        if (!providerType) return;

        let newProvider: PaymentProvider;
        if (providerType === 'PayPal') {
            newProvider = { PayPal: { id: providerId } };
        } else if (providerType === 'Revolut') {
            if (userTypeToString(user.user_type) === 'Offramper' && !revolutName) {
                setMessage('Name is required.');
                return;
            }
            newProvider = { Revolut: { id: providerId, scheme: revolutScheme, name: revolutName ? [revolutName] : [] } };
        } else {
            setMessage('Unknown payment provider');
            return;
        }

        try {
            const result = await backend.add_payment_provider_for_user(user.login_method, newProvider);
            if ('Ok' in result) {
                const updatedProviders = [...user.payment_providers, newProvider]
                setUser({ ...user, payment_providers: updatedProviders });
                setMessage('Provider updated successfully');
            } else {
                setMessage(rampErrorToString(result.Err));
            }
        } catch (error) {
            setMessage(`Failed to update provider: ${error}`);
        }
    };

    const handleAddAddress = async () => {
        if (!selectedAddressType) return;

        const address = {
            address_type: { [selectedAddressType]: null },
            address: newAddress
        } as Address;

        try {
            const result = await backend.add_address_for_user(user.login_method, address);
            if ('Ok' in result) {
                const updatedAddresses = [...user.addresses, address];
                setUser({ ...user, addresses: updatedAddresses });
                setMessage('Address updated successfully');
            } else {
                setMessage(result.Err.toString());
            }
        } catch (error) {
            setMessage(`Failed to update address: ${error}`);
        }
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

            <hr />

            <div className="my-4">
                <div className="flex justify-between items-center">
                    <span className="font-medium">Addresses:</span>
                </div>
                <ul className="pl-4 mt-2">
                    {user.addresses.map((addr, index) => (
                        <li key={index} className="py-1 text-gray-700" style={{
                            color: addr.address === user.login_method.address ? 'blue' : ''
                        }}>
                            <span className="flex-1 text-sm text-gray-500">({Object.keys(addr.address_type)[0]})</span>
                            <span className="ml-2">{truncate(addr.address, 8, 8)}</span>
                        </li>
                    ))}
                </ul>
            </div >

            <div className="flex mb-4 gap-2">
                <select
                    value={selectedAddressType}
                    onChange={(e) => setSelectedAddressType(e.target.value as AddressTypes)}
                    className="w-1/3 px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                >
                    <option value="" selected>Select Address Type</option>
                    <option value="EVM">EVM</option>
                    <option value="ICP">ICP</option>
                    <option value="Solana">Solana</option>
                </select>
                <input
                    type="text"
                    value={newAddress}
                    onChange={(e) => setNewAddress(e.target.value)}
                    placeholder="Enter Address"
                    className="w-2/3 px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
                <button onClick={handleAddAddress} className="ml-2 px-4 py-2 bg-blue-600 text-white font-medium rounded-lg hover:bg-blue-700">
                    Add
                </button>
            </div>

            <hr />

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
                            onChange={(e) => setRevolutScheme(e.target.value as revolutScheme)}
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

            <hr />

            {message && <p className="mt-4 text-sm font-medium text-grey-600">{message}</p>}
        </div >
    );
};

export default UserProfile;
