import React, { useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useUser } from '../UserContext';
import { userTypeToString } from '../model/utils';

const UserProfile: React.FC = () => {
    const { user } = useUser();
    const navigate = useNavigate();

    useEffect(() => {
        if (!user) {
            navigate('/');
        }
    }, [user, navigate]);

    if (!user) {
        return null;
    }

    return (
        <div className="p-6 max-w-md mx-auto bg-white rounded-xl shadow-md space-y-4">
            <h2 className="text-lg font-bold mb-4">User Profile</h2>
            <div>
                <strong>Address:</strong> {user.evm_address}
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
                <ul className="list-disc list-inside bg-gray-100 p-2 rounded">
                    {user.payment_providers.map((provider, index) => (
                        <li key={index}>
                            {Object.keys(provider)[0]}: {Object.values(provider)[0].id}
                        </li>
                    ))}
                </ul>
            </div>
        </div>
    );
};

export default UserProfile;
