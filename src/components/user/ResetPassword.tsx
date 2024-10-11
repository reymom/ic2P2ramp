import React, { useState, useEffect } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faEye, faEyeSlash } from '@fortawesome/free-solid-svg-icons';

import { backend } from '../../model/backendProxy';
import { rampErrorToString } from '../../model/error';
import { validatePassword } from '../../model/helper';
import { clearTempResetPasswordData, getTempResetPasswordData } from '../../model/emailConfirmation';

const ResetPassword: React.FC = () => {
    const [password, setPassword] = useState('');
    const [confirmPassword, setConfirmPassword] = useState('');
    const [isPasswordVisible, setIsPasswordVisible] = useState(false);
    const [isConfirmPasswordVisible, setIsConfirmPasswordVisible] = useState(false);
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

        const passwordError = validatePassword(password);
        if (passwordError) {
            setMessage(passwordError);
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
                navigate("/register");
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
        <div className="bg-gray-700 rounded-xl p-8 max-w-md mx-auto text-white">
            <div className="text-center mb-8">
                <h2 className="text-2xl font-semibold">Reset Password</h2>
            </div>
            <form onSubmit={handleResetPassword} className="flex flex-col space-y-4">
                <div className="relative">
                    <input
                        type={isPasswordVisible ? 'text' : 'password'}
                        value={password}
                        onChange={(e) => setPassword(e.target.value)}
                        placeholder="Enter new password"
                        className="px-4 py-2 bg-gray-600 border rounded w-full"
                        required
                    />
                    <button
                        type="button"
                        className="absolute inset-y-0 right-0 px-3 py-2"
                        onClick={() => setIsPasswordVisible(!isPasswordVisible)}
                    >
                        <FontAwesomeIcon icon={isPasswordVisible ? faEyeSlash : faEye} className="text-gray-300" />
                    </button>
                </div>
                <div className="relative">
                    <input
                        type={isConfirmPasswordVisible ? 'text' : 'password'}
                        value={confirmPassword}
                        onChange={(e) => setConfirmPassword(e.target.value)}
                        placeholder="Confirm new password"
                        className="px-4 py-2 bg-gray-600 border rounded w-full"
                        required
                    />
                    <button
                        type="button"
                        className="absolute inset-y-0 right-0 px-3 py-2"
                        onClick={() => setIsConfirmPasswordVisible(!isConfirmPasswordVisible)}
                    >
                        <FontAwesomeIcon icon={isConfirmPasswordVisible ? faEyeSlash : faEye} className="text-gray-300" />
                    </button>
                </div>
                <button type="submit" className="px-4 py-2 bg-indigo-700 hover:bg-indigo-800 rounded w-full" disabled={isLoading}>
                    {isLoading ? 'Resetting...' : 'Reset Password'}
                </button>
            </form>
            {isLoading && (
                <div className="flex justify-center items-center space-x-2 mt-4">
                    <div className="w-6 h-6 border-t-2 border-b-2 border-indigo-400 rounded-full animate-spin"></div>
                    <div className="text-sm font-medium text-gray-300">Loading...</div>
                </div>
            )}
            {message && <p className="mt-4 text-sm font-medium text-red-600 break-all">{message}</p>}
        </div>
    );
};

export default ResetPassword;
