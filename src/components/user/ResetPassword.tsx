import React, { useState, useEffect } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import { clearTempResetPasswordData, getTempResetPasswordData } from '../../model/emailConfirmation';
import { backend } from '../../declarations/backend';
import { rampErrorToString } from '../../model/error';

const ResetPassword: React.FC = () => {
    const [password, setPassword] = useState('');
    const [confirmPassword, setConfirmPassword] = useState('');
    const [message, setMessage] = useState<string>();
    const [isLoading, setIsLoading] = useState(false);

    const [searchParams] = useSearchParams();
    const navigate = useNavigate();

    useEffect(() => {
        const token = searchParams.get('token');
        if (!token) {
            navigate("/");
        }
    }, [searchParams, navigate]);

    useEffect(() => {
        const resetPasswordData = getTempResetPasswordData();
        if (!resetPasswordData || !resetPasswordData.confirmationToken) {
            setMessage('Your session expired. Please make sure to use the same browser in the whole recovery process.');
        }
    }, [searchParams, navigate, getTempResetPasswordData])

    const handleResetPassword = async (event: React.FormEvent<HTMLFormElement>) => {
        event.preventDefault();
        setMessage(undefined);

        const resetPasswordData = getTempResetPasswordData();
        if (!resetPasswordData) return;

        const token = searchParams.get('token');
        if (token !== resetPasswordData.confirmationToken) {
            setMessage('Invalid or expired confirmation token.')
            return;
        }

        if (password !== confirmPassword) {
            setMessage('Passwords do not match.');
            return;
        }

        setIsLoading(true);
        try {
            const response = await backend.update_password(resetPasswordData.loginMethod, [password]);
            if ('Ok' in response) {
                setMessage('Password has been reset successfully.');
                clearTempResetPasswordData();
                navigate("/login");
            } else {
                setMessage(`Failed to reset password: ${rampErrorToString(response.Err)}`);
            }
        } catch (error) {
            setMessage(`Error resetting password: ${error}`);
        } finally {
            setIsLoading(false);
        }
    };

    return (
        <div className="max-w-md mx-auto rounded-xl">
            <h2 className="text-lg font-bold mb-4">Reset Password</h2>
            <form onSubmit={handleResetPassword} className="flex flex-col space-y-4">
                <input
                    type="password"
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    placeholder="Enter new password"
                    className="px-4 py-2 border rounded w-full"
                    required
                />
                <input
                    type="password"
                    value={confirmPassword}
                    onChange={(e) => setConfirmPassword(e.target.value)}
                    placeholder="Confirm new password"
                    className="px-4 py-2 border rounded w-full"
                    required
                />
                <button type="submit" className="px-4 py-2 bg-blue-500 text-white rounded w-full" disabled={isLoading}>
                    {isLoading ? 'Resetting...' : 'Reset Password'}
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

export default ResetPassword;
