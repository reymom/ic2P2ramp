import React, { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useUser } from '../UserContext';
import { paymentProviderToString, stringToPaymentProvider, userTypeToString } from '../model/utils';
import { PaymentProvider } from '../declarations/backend/backend.did';
import { backend } from '../declarations/backend';
import { PaymentProviderTypes, providerTypes } from '../model/types';
import { truncate } from '../model/helper';

const UserProfile: React.FC = () => {
    const [newProviders, setNewProviders] = useState<{ [key in PaymentProviderTypes]?: string }>({});
    const [message, setMessage] = useState('');
    const { user, setUser } = useUser();
    const navigate = useNavigate();

    useEffect(() => {
        if (!user) {
            navigate('/');
        } else {
            const initialProviders: { [key in PaymentProviderTypes]?: string } = {};
            user.payment_providers.forEach(provider => {
                const key = paymentProviderToString(provider);
                initialProviders[key] = Object.values(provider)[0].id;
            });
            setNewProviders(initialProviders);
        }
    }, [user, navigate]);

    const handleProviderChange = (type: PaymentProviderTypes, value: string) => {
        setNewProviders(prev => ({ ...prev, [type]: value }));
    };

    if (!user) {
        return null;
    }

    const handleProviderSubmit = async (key: PaymentProviderTypes) => {
        const updatedProvider = stringToPaymentProvider(key, newProviders[key]!);
        try {
            const result = await backend.add_payment_provider_for_user(user.evm_address, updatedProvider);
            if ('Ok' in result) {
                const updatedProviders = user.payment_providers.filter(provider => paymentProviderToString(provider) !== key);
                updatedProviders.push(updatedProvider);
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
                {providerTypes.map(type => (
                    <div key={type} className="my-4 flex items-center">
                        <label className="block text-gray-700 w-20">{type}</label>
                        <input
                            type="text"
                            value={newProviders[type] || ''}
                            onChange={(e) => handleProviderChange(type, e.target.value)}
                            className="px-3 py-2 border rounded flex-grow mr-2 w-36"
                        />
                        <button
                            onClick={() => handleProviderSubmit(type)}
                            className="px-2 py-2 bg-blue-500 text-white rounded"
                        >
                            Update
                        </button>
                    </div>
                ))}
            </div>
            {message && <p className="mt-4 text-sm font-medium text-grey-600">{message}</p>}
        </div>
    );
};

export default UserProfile;
