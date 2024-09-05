import React, { useState } from 'react';
import { generateConfirmationToken, sendRecoverPassword, storeTempResetPasswordData } from '../../model/emailConfirmation';
import { LoginAddress } from '../../declarations/backend/backend.did';
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
            if ('Err' in result && !('InvalidPassword' in result.Err)) {
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
        <div className="max-w-md mx-auto rounded-xl">
            <h2 className="text-lg font-bold mb-4">Forgot Password</h2>
            <form onSubmit={handleForgotPassword} className="flex flex-col space-y-4">
                <input
                    type="email"
                    value={email}
                    onChange={(e) => setEmail(e.target.value)}
                    placeholder="Enter your email"
                    className="px-4 py-2 border rounded w-full"
                    required
                />
                <button type="submit" className="px-4 py-2 bg-blue-500 text-white rounded w-full" disabled={isLoading}>
                    {isLoading ? 'Sending...' : 'Send Password Reset Link'}
                </button>
            </form>
            {isLoading && (
                <div className="flex justify-center items-center space-x-2 mt-4">
                    <div className="w-4 h-4 border-t-2 border-b-2 border-indigo-600 rounded-full animate-spin"></div>
                    <div className="text-sm font-medium text-gray-700">Loading...</div>
                </div>
            )}
            {message && <p className="mt-4 text-sm font-medium text-gray-700 break-all">{message}</p>}
        </div>
    );
};

export default ForgotPassword;
