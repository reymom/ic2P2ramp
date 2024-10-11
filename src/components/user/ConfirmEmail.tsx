import React, { useEffect, useState } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';

import { backend } from '../../model/backendProxy';
import { getTempUserData, clearTempUserData } from '../../model/emailConfirmation';
import { rampErrorToString } from '../../model/error';
import { stringToUserType } from '../../model/utils';

const ConfirmEmail: React.FC = () => {
    const [message, setMessage] = useState('');
    const [isLoading, setIsLoading] = useState(false);
    const [manualToken, setManualToken] = useState('');

    const navigate = useNavigate();
    const [searchParams] = useSearchParams();

    useEffect(() => {
        const token = searchParams.get('token');
        if (token) {
            confirmEmail(token);
        }
    }, [searchParams]);

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
                        const email = 'Email' in tempUserData.loginMethod && tempUserData.loginMethod.Email.email
                        navigate(`/?auth=true&email=${email}&pwd=${tempUserData.password}`);
                        clearTempUserData();
                    } else {
                        setMessage(rampErrorToString(result.Err));
                    }
                })
                .catch((error) => {
                    setMessage(`Failed to confirm email: ${error}`);
                })
                .finally(() => {
                    setIsLoading(false);
                    clearTempUserData();
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
        <div className="max-w-md mx-auto rounded-xl p-8 text-white bg-gray-700">
            <div className="text-center mb-8">
                <h2 className="text-2xl font-semibold mb-4">Email Confirmation</h2>
            </div>
            {isLoading ? (
                <div className="flex justify-center items-center space-x-2">
                    <div className="w-6 h-6 border-t-2 border-b-2 border-indigo-400 rounded-full animate-spin"></div>
                    <div className="text-sm font-medium text-gray-300">Confirming your email...</div>
                </div>
            ) : message ? (
                <p className="mt-4 text-sm font-medium text-red-500 break-all">{message}</p>
            ) : (
                <form onSubmit={handleSubmit} className="mt-4">
                    <label htmlFor="token" className="block text-sm font-medium text-gray-200">
                        Enter Confirmation Token received in your email
                    </label>
                    <input
                        id="token"
                        type="text"
                        value={manualToken}
                        onChange={(e) => setManualToken(e.target.value)}
                        placeholder="Enter token"
                        className="mt-2 p-2 border rounded bg-gray-600 w-full"
                        required
                    />
                    <button
                        type="submit"
                        className="mt-4 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded w-full"
                    >
                        Confirm Email
                    </button>
                </form>
            )}
        </div>
    );
};

export default ConfirmEmail;
