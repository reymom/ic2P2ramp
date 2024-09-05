import React, { useEffect, useState } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';

import { backend } from '../../declarations/backend';
import { getTempUserData, clearTempUserData } from '../../model/emailConfirmation';
import { rampErrorToString } from '../../model/error';
import { stringToUserType } from '../../model/utils';
import { useUser } from '../user/UserContext';

const ConfirmEmail: React.FC = () => {
    const [message, setMessage] = useState('');
    const [isLoading, setIsLoading] = useState(false);
    const [manualToken, setManualToken] = useState('');

    const navigate = useNavigate();
    const [searchParams] = useSearchParams();
    const { setUser: setGlobalUser } = useUser();

    useEffect(() => {
        const token = searchParams.get('token');
        if (token) {
            confirmEmail(token);
        }
    }, [searchParams]);

    useEffect(() => {
        const tempUserData = getTempUserData();
        if (!tempUserData) {
            navigate("/");
            return;
        }
    }, [navigate])

    const confirmEmail = (token: string) => {
        const tempUserData = getTempUserData();

        if (tempUserData && tempUserData.confirmationToken === token) {
            setIsLoading(true);

            backend.register_user(
                stringToUserType(tempUserData.userType),
                tempUserData.providers,
                tempUserData.loginMethod,
                tempUserData.password ? [tempUserData.password] : []
            )
                .then((result) => {
                    if ('Ok' in result) {
                        setGlobalUser(result.Ok);
                        clearTempUserData();
                        navigate(tempUserData.userType === "Onramper" ? "/view" : "/create");
                    } else {
                        setMessage(rampErrorToString(result.Err));
                    }
                })
                .catch((error) => {
                    setMessage(`Failed to confirm email: ${error}`);
                })
                .finally(() => {
                    setIsLoading(false);
                });
        } else {
            setMessage('Invalid or expired confirmation token.');
        }
    };

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        confirmEmail(manualToken);
    };

    return (
        <div className="max-w-md mx-auto rounded-xl">
            <h2 className="text-lg font-bold mb-4">Email Confirmation</h2>
            {isLoading ? (
                <div className="flex justify-center items-center space-x-2">
                    <div className="w-4 h-4 border-t-2 border-b-2 border-indigo-600 rounded-full animate-spin"></div>
                    <div className="text-sm font-medium text-gray-700">Confirming your email...</div>
                </div>
            ) : message ? (
                <p className="mt-4 text-sm font-medium text-red-400-700 break-all">{message}</p>
            ) : (
                <form onSubmit={handleSubmit} className="mt-4">
                    <label htmlFor="token" className="block text-sm font-medium text-gray-700">
                        Enter Confirmation Token received in your email
                    </label>
                    <input
                        id="token"
                        type="text"
                        value={manualToken}
                        onChange={(e) => setManualToken(e.target.value)}
                        placeholder="Enter token"
                        className="mt-2 p-2 border rounded w-full"
                        required
                    />
                    <button
                        type="submit"
                        className="mt-4 px-4 py-2 bg-blue-500 text-white rounded w-full"
                    >
                        Confirm Email
                    </button>
                </form>
            )}
        </div>
    );
};

export default ConfirmEmail;
