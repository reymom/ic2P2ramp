import React, { useState } from 'react';

import { LoginAddress } from '../../declarations/backend/backend.did';
import { generateConfirmationToken, sendRecoverPassword, storeTempResetPasswordData } from '../../model/emailConfirmation';
import { isInvalidPasswordError } from '../../model/error';
import { useUser } from './UserContext';

const ForgotPassword: React.FC = () => {
    const [email, setEmail] = useState('');
    const [message, setMessage] = useState<string>();
    const [isLoading, setIsLoading] = useState(false);

    const { authenticateUser } = useUser();

    const handleForgotPassword = async (event: React.FormEvent<HTMLFormElement>) => {
        event.preventDefault();
        setIsLoading(true);
        setMessage(undefined);

        const confirmationToken = generateConfirmationToken();
        const loginMethod: LoginAddress = { 'Email': { email } };
        try {
            const result = await authenticateUser(loginMethod, { signature: [], password: ["notapassword"] });
            if ('Err' in result && !(isInvalidPasswordError(result.Err))) {
                console.log("result.Err = ", result.Err);
                setMessage('Email is not registered');
                return;
            }
        } catch (error) {
            console.log("error = ", error);
            setMessage(`Internal Erorr: ${error}`);
            return;
        } finally {
            setIsLoading(false);
        }

        storeTempResetPasswordData({ loginMethod, confirmationToken })
        try {
            const response = await sendRecoverPassword(email, confirmationToken);
            if (response.ok) {
                setMessage('Password reset link has been sent to your email.');
            } else {
                setMessage('Failed to send password reset link.');
            }
        } catch (error) {
            setMessage('Error sending password reset email.');
        } finally {
            setIsLoading(false);
        }
    };

    return (
        <div className="bg-gray-700 rounded-xl p-8 max-w-lg mx-auto space-y-4 text-white">
            <div className="text-center mb-8">
                <h2 className="text-2xl font-semibold">Forgot Password</h2>
            </div>
            <form onSubmit={handleForgotPassword} className="flex flex-col space-y-4">
                <input
                    type="email"
                    value={email}
                    onChange={(e) => setEmail(e.target.value)}
                    placeholder="Enter your email"
                    className="px-4 py-2 bg-gray-600 border rounded w-full"
                    required
                />
                <button type="submit" className="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded w-full" disabled={isLoading}>
                    {isLoading ? 'Sending...' : 'Send Password Reset Link'}
                </button>
            </form>
            {isLoading && (
                <div className="flex justify-center items-center space-x-2">
                    <div className="w-6 h-6 border-t-2 border-b-2 border-indigo-400 rounded-full animate-spin"></div>
                    <div className="text-sm font-medium text-gray-300">Loading...</div>
                </div>
            )}
            {message && <p className="text-sm font-medium text-gray-300 break-all">{message}</p>}
        </div>
    );
};

export default ForgotPassword;
