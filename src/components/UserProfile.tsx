import React, { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useUser } from '../UserContext';
import { paymentProviderTypeToString, stringToPaymentProviderType, userTypeToString } from '../model/utils';
import { backend } from '../declarations/backend';
import { PaymentProviderTypes, providerTypes } from '../model/types';
import { truncate } from '../model/helper';
import { PaymentProvider } from '../declarations/backend/backend.did';

const UserProfile: React.FC = () => {
    const [providerType, setProviderType] = useState<PaymentProviderTypes>();
    const [providerId, setProviderId] = useState('');
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

        const newProvider: PaymentProvider = {
            provider_type: stringToPaymentProviderType(providerType),
            id: providerId
        }
        try {
            const result = await backend.add_payment_provider_for_user(user.evm_address, newProvider);
            if ('Ok' in result) {
                const updatedProviders = [...user.payment_providers, newProvider]
                setUser({ ...user, payment_providers: updatedProviders });
                setMessage('Provider updated successfully');
            } else {
                setMessage(result.Err.toString());
            }
        } catch (error) {
            setMessage(`Failed to update provider: ${error}`);
        }
    };

    return (
        <div className="p-6 max-w-lg mx-auto bg-white rounded-xl shadow-md space-y-4">
            <h2 className="text-lg font-bold mb-4">User Profile</h2>
            <div>
                <strong>Address:</strong> {truncate(user.evm_address, 8, 8)}
            </div>
            <div>
                <strong>User Type:</strong> {userTypeToString(user.user_type)}
            </div>
            <div>
                <strong>Fiat Amount:</strong> {user.fiat_amount.toString()}
            </div>
            <div>
                <strong>Score:</strong> {user.score}
            </div>
            <div>
                <strong>Payment Providers:</strong>
                {user.payment_providers.map((provider, index) => (
                    <li key={index} className="py-1">
                        {paymentProviderTypeToString(provider.provider_type)}: {provider.id}
                    </li>
                ))}
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
            {message && <p className="mt-4 text-sm font-medium text-grey-600">{message}</p>}
        </div>
    );
};

export default UserProfile;
