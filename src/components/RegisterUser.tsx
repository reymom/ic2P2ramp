import React, { useState } from 'react';
import { backend } from '../declarations/backend';
import { PaymentProvider } from '../declarations/backend/backend.did';
import { useAccount } from 'wagmi';
import { pageTypes, PaymentProviderTypes } from '../model/types';

interface RegisterUserProps {
    setCurrentTab: (tab: pageTypes) => void;
}

const RegisterUser: React.FC<RegisterUserProps> = ({ setCurrentTab }) => {
    const [providers, setProviders] = useState<PaymentProvider[]>([
        { PayPal: { 'id': "fdsadfs" } },
        { Revolut: { 'id': "fdsdfsa" } }
    ]);
    const [providerType, setProviderType] = useState<PaymentProviderTypes>("PayPal");
    const [providerId, setProviderId] = useState('');
    const [message, setMessage] = useState('');

    const { address } = useAccount();

    const handleAddProvider = () => {
        const updatedProviders = providers.map((provider) => {
            const key = Object.keys(provider)[0];
            if (key === providerType) {
                return providerType === 'PayPal'
                    ? { PayPal: { id: providerId } }
                    : { Revolut: { id: providerId } };
            }
            return provider;
        });

        if (!providers.some(provider => Object.keys(provider)[0] === providerType)) {
            const newProvider = providerType === 'PayPal'
                ? { PayPal: { id: providerId } }
                : { Revolut: { id: providerId } };
            updatedProviders.push(newProvider);
        }

        setProviders(updatedProviders);
        setProviderId('');
    };

    const handleSubmit = async () => {
        if (providers.length === 0) {
            setMessage('Please add at least one payment provider.');
            return;
        }

        try {
            const result = await backend.register_user(address as string, providers);
            if ('Ok' in result) {
                setCurrentTab(pageTypes.create);
            } else {
                setMessage(result.Err);
            }
        } catch (error) {
            setMessage(`Failed to register user: ${error}`);
        }
    };

    return (
        <div className="p-6 max-w-md mx-auto bg-white rounded-xl shadow-md space-y-4">
            <h2 className="text-lg font-bold mb-4">Register</h2>
            <div className="flex items-center mb-4">
                <label className="block text-gray-700 w-24">Provider:</label>
                <select
                    value={providerType}
                    onChange={(e) => setProviderType(e.target.value as 'PayPal' | 'Revolut')}
                    className="flex-grow px-3 py-2 border rounded"
                >
                    <option value="PayPal">PayPal</option>
                    <option value="Revolut">Revolut</option>
                </select>
            </div>
            <div className="flex items-center mb-4">
                <label className="block text-gray-700 w-24">Provider ID:</label>
                <input
                    type="text"
                    value={providerId}
                    onChange={(e) => setProviderId(e.target.value)}
                    className="flex-grow px-3 py-2 border rounded"
                />
            </div>
            <button onClick={handleAddProvider} className="px-4 py-2 bg-blue-500 text-white rounded">
                Add Provider
            </button>
            <div className="mt-4">
                <ul className="list-disc list-inside bg-gray-100 p-2 rounded">
                    {providers.map((provider, index) => (
                        <li key={index} className="py-1">
                            {Object.keys(provider)[0]}: {Object.values(provider)[0].id}
                        </li>
                    ))}
                </ul>
            </div>
            <div className="flex justify-between mt-4">
                <button
                    onClick={() => setCurrentTab(pageTypes.view)}
                    className="px-4 py-2 bg-gray-400 text-white rounded"
                >
                    Skip
                </button>
                <button onClick={handleSubmit} className="px-4 py-2 bg-green-500 text-white rounded">
                    Register
                </button>
            </div>
            {message && <p className="mt-4 text-sm font-medium text-gray-700">{message}</p>}
        </div>
    );
};

export default RegisterUser;
