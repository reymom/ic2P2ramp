import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAccount } from 'wagmi';

import { backend } from '../declarations/backend';
import { PaymentProvider } from '../declarations/backend/backend.did';
import { PaymentProviderTypes, providerTypes, UserTypes } from '../model/types';
import { stringToUserType, paymentProviderTypeToString, stringToPaymentProviderType } from '../model/utils';
import { useUser } from '../UserContext';

const RegisterUser: React.FC = () => {
    const [userType, setUserType] = useState<UserTypes>("Onramper");
    const [providers, setProviders] = useState<PaymentProvider[]>([]);
    const [providerType, setProviderType] = useState<PaymentProviderTypes>("PayPal");
    const [providerId, setProviderId] = useState('');
    const [message, setMessage] = useState('');
    const [isLoading, setIsLoading] = useState(false);

    const { address } = useAccount();
    const { setUser: setGlobalUser } = useUser();
    const navigate = useNavigate();

    const handleAddProvider = () => {
        const newProvider: PaymentProvider = {
            provider_type: stringToPaymentProviderType(providerType),
            id: providerId,
        }

        const updatedProviders = [...providers, newProvider];
        setProviders(updatedProviders);
        setProviderId('');
    };

    const handleSubmit = async () => {
        if (providers.length === 0) {
            setMessage('Please add at least one payment provider.');
            return;
        }

        setIsLoading(true);
        try {
            const result = await backend.register_user(address as string, stringToUserType(userType), providers);
            if ('Ok' in result) {
                setGlobalUser(result.Ok);
                navigate(userType === "Onramper" ? "/view" : "/create");
            } else {
                setGlobalUser(null);
                setMessage(result.Err.toString());
            }
        } catch (error) {
            setMessage(`Failed to register user: ${error}`);
        } finally {
            setIsLoading(false);
        }
    };

    return (
        <div className="p-6 max-w-md mx-auto bg-white rounded-xl shadow-md space-y-4">
            <h2 className="text-lg font-bold mb-4">Register</h2>
            <div className="flex items-center mb-6">
                <label className="block text-gray-400 w-24">User Type:</label>
                <select
                    value={userType}
                    onChange={(e) => setUserType(e.target.value as 'Offramper' | 'Onramper')}
                    className="flex-grow px-3 py-2 border rounded"
                >
                    <option value="Offramper">Offramper</option>
                    <option value="Onramper">Onramper</option>
                </select>
            </div>
            <div className="flex items-center mb-4">
                <label className="block text-gray-700 w-24">Provider:</label>
                <select
                    value={providerType}
                    onChange={(e) => setProviderType(e.target.value as PaymentProviderTypes)}
                    className="flex-grow px-3 py-2 border rounded"
                >
                    {providerTypes.map(type => (
                        <option value={type}>{type}</option>
                    ))}
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
                            {paymentProviderTypeToString(provider.provider_type)}: {provider.id}
                        </li>
                    ))}
                </ul>
            </div>
            <div className="flex justify-between mt-4">
                <button
                    onClick={() => navigate("/view")}
                    className="px-4 py-2 bg-gray-400 text-white rounded"
                >
                    Skip
                </button>
                <button onClick={handleSubmit} className="px-4 py-2 bg-green-500 text-white rounded">
                    Register
                </button>
            </div>
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
};

export default RegisterUser;
