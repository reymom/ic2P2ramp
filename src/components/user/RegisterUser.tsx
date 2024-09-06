import React, { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';

import { backend } from '../../declarations/backend';
import { PaymentProvider } from '../../declarations/backend/backend.did';
import { PaymentProviderTypes, providerTypes, revolutSchemeTypes, revolutSchemes, UserTypes } from '../../model/types';
import { stringToUserType } from '../../model/utils';
import { useUser } from './UserContext';
import { rampErrorToString } from '../../model/error';
import { truncate } from '../../model/helper';
import { generateConfirmationToken, sendConfirmationEmail, storeTempUserData } from '../../model/emailConfirmation';
import { ethers } from 'ethers';

const RegisterUser: React.FC = () => {
    const [userType, setUserType] = useState<UserTypes>("Onramper");
    const [providers, setProviders] = useState<PaymentProvider[]>([]);
    const [providerType, setProviderType] = useState<PaymentProviderTypes>("PayPal");
    const [providerId, setProviderId] = useState('');
    const [revolutScheme, setRevolutScheme] = useState<revolutSchemeTypes>();
    const [revolutName, setRevolutName] = useState('');
    const [message, setMessage] = useState('');
    const [isLoading, setIsLoading] = useState(false);

    const { authenticateUser, setUser: setGlobalUser, user, loginMethod, password, backendActor } = useUser();
    const navigate = useNavigate();

    useEffect(() => {
        if (user) {
            navigate("/")
            return;
        }
    }, [user])

    useEffect(() => {
        if (!loginMethod) {
            navigate("/")
            return;
        }
    }, [loginMethod])

    const handleAddProvider = () => {
        let newProvider: PaymentProvider;
        if (providerType === 'PayPal') {
            newProvider = { PayPal: { id: providerId } };
        } else if (providerType === 'Revolut') {
            if (userType === 'Offramper' && !revolutName) {
                setMessage('Name is required.');
                return;
            }
            if (!revolutScheme) {
                setMessage('Select a Revolut Scheme');
                return;
            }
            newProvider = { Revolut: { id: providerId, scheme: revolutScheme, name: revolutName ? [revolutName] : [] } };
        } else {
            setMessage('Unknown payment provider');
            return;
        }

        const updatedProviders = [...providers, newProvider];
        setProviders(updatedProviders);
        setProviderId('');
        setRevolutScheme('UK.OBIE.SortCodeAccountNumber');
        setRevolutName('');
    };

    const handleSubmit = async () => {
        if (providers.length === 0) {
            setMessage('Please add at least one payment provider.');
            return;
        }

        if (!loginMethod) {
            navigate("/")
            return;
        }

        if ('Email' in loginMethod) {
            await handleEmailConfirmation();
            return;
        }

        setIsLoading(true);
        try {
            let result = await backendActor.register_user(stringToUserType(userType), providers, loginMethod, []);
            if ('Err' in result) {
                setGlobalUser(null);
                setMessage(`Error registering user: ${rampErrorToString(result.Err)}`)
            }
            if ('Ok' in result) {
                if ('EVM' in loginMethod) {
                    await handleEvmSignature();
                } else {
                    try {
                        const result = await authenticateUser(loginMethod, { signature: [], password: [] });
                        if ('Err' in result) setMessage(`Failed to authenticate user: ${rampErrorToString(result.Err)}`);
                        if ('Ok' in result) {
                            setGlobalUser(result.Ok);
                            navigate("Offramper" in result.Ok.user_type ? "/create" : "/view");
                        }
                    } catch (error) {
                        setMessage(`Failed to authenticate user: ${error}`);
                    }
                    navigate(userType === "Onramper" ? "/view" : "/create");
                }

            }
        } catch (error) {
            setMessage(`Failed to register user: ${error}`);
        } finally {
            setIsLoading(false);
        }
    };

    const handleEvmSignature = async () => {
        try {
            const result = await backend.generate_evm_auth_message(loginMethod!);

            if ('Err' in result) setMessage(`Failed to generate evm nonce ${rampErrorToString(result.Err)}`);
            if ('Ok' in result) {
                const provider = new ethers.BrowserProvider(window.ethereum);
                const signer = await provider.getSigner();
                const signature = await signer.signMessage(result.Ok);
                try {
                    const result = await authenticateUser(loginMethod, { signature: [signature], password: [] });
                    if ('Err' in result) setMessage(`Failed to authenticate user: ${rampErrorToString(result.Err)}`);
                    if ('Ok' in result) {
                        setGlobalUser(result.Ok);
                        navigate("Offramper" in result.Ok.user_type ? "/create" : "/view");
                    }
                } catch (error) {
                    setMessage(`Failed to authenticate user: ${error}`);
                }
            }
        } catch (error) {
            setMessage(`Failed to generate evm nonce: ${error}`);
        }
    }

    const handleEmailConfirmation = async () => {
        if (!loginMethod || !password || !('Email' in loginMethod)) {
            navigate("/")
            return;
        }

        const confirmationToken = generateConfirmationToken();
        storeTempUserData({
            password,
            providers,
            userType,
            loginMethod,
            confirmationToken
        });

        try {
            sendConfirmationEmail(loginMethod.Email.email, confirmationToken);
        } catch (error) {
            setMessage(`Failed to send confirmation email: ${error}`)
            return;
        }
        navigate("/confirm-email");
    };

    return (
        <div className="max-w-md mx-auto rounded-xl">
            <h2 className="text-lg font-bold mb-4">Register</h2>
            <div className="flex items-center mb-6">
                <label className="block text-gray-400 w-32">User Type:</label>
                <select
                    value={userType}
                    onChange={(e) => setUserType(e.target.value as 'Offramper' | 'Onramper')}
                    className="flex-grow px-3 py-2 border rounded"
                >
                    <option value="Offramper">Offramper</option>
                    <option value="Onramper">Onramper</option>
                </select>
            </div>
            {loginMethod && (
                <div className="flex items-center mb-4">
                    <label className="block text-gray-700 w-32">Login Address:</label>
                    <span className="flex-grow px-3 py-2 border rounded bg-gray-100 truncate text-left">
                        {(() => {
                            if ('EVM' in loginMethod) {
                                return truncate(loginMethod.EVM.address, 12, 10);
                            } else if ('ICP' in loginMethod) {
                                return truncate(loginMethod.ICP.principal_id, 12, 10);
                            } else if ('Email' in loginMethod) {
                                return truncate(loginMethod.Email.email, 16, 14);
                            } else if ('Solana' in loginMethod) {
                                return truncate(loginMethod.Solana.address, 12, 10);
                            }
                            return '';
                        })()}
                    </span>
                </div>
            )}

            <hr className="border-t border-gray-300 w-full my-4" />

            <div className="flex items-center">
                <label className="block text-gray-700 w-32">Provider:</label>
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
            <div className="flex items-center mt-1">
                <label className="block text-gray-700 w-32">Provider ID:</label>
                <input
                    type="text"
                    value={providerId}
                    onChange={(e) => setProviderId(e.target.value)}
                    className="flex-grow px-3 py-2 border rounded"
                />
            </div>

            {providerType === 'Revolut' && (
                <>
                    <div className="flex items-center mt-1">
                        <label className="block text-gray-700 w-32">Scheme:</label>
                        <select
                            value={revolutScheme}
                            onChange={(e) => setRevolutScheme(e.target.value as revolutSchemeTypes)}
                            className="flex-grow px-3 py-2 border rounded"
                        >
                            <option value="" selected>Select Scheme</option>
                            {revolutSchemes.map(type => (
                                <option value={type}>{type}</option>
                            ))}
                        </select>
                    </div>
                    {userType === 'Offramper' && (
                        <div className="flex items-center mt-1">
                            <label className="block text-gray-700 w-32 mt-1">Name:</label>
                            <input
                                type="text"
                                value={revolutName}
                                onChange={(e) => setRevolutName(e.target.value)}
                                className="flex-grow px-3 py-2 border rounded"
                            />
                        </div>
                    )}
                </>
            )}
            <button onClick={handleAddProvider} className="mt-4 px-4 py-2 bg-blue-500 text-white rounded w-full">
                Add Provider
            </button>
            <div className="mt-4">
                <ul className="list-disc list-inside bg-gray-100 p-2 rounded">
                    {providers.map((provider, index) => {
                        if ('PayPal' in provider) {
                            return (
                                <li key={index} className="py-1">
                                    PayPal: {provider.PayPal.id}
                                </li>
                            );
                        } else if ('Revolut' in provider) {
                            return (
                                <li key={index} className="py-1">
                                    Revolut: {provider.Revolut.id}
                                    <div>Scheme: {provider.Revolut.scheme}</div>
                                    {provider.Revolut.name && provider.Revolut.name.length > 0 && (
                                        <div>Name: {provider.Revolut.name[0]}</div>
                                    )}
                                </li>
                            );
                        } else {
                            return null;
                        }
                    })}
                </ul>
            </div>

            <hr className="border-t border-gray-300 w-full my-4" />

            <div className="flex justify-between">
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
                message && <p className="mt-4 text-sm font-medium text-gray-700 break-all">{message}</p>
            )}
        </div>
    );
};

export default RegisterUser;
