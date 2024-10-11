import React, { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';

import { backend } from '../../model/backendProxy';
import { PaymentProvider } from '../../declarations/backend/backend.did';
import { PaymentProviderTypes, providerTypes, revolutSchemeTypes, revolutSchemes, UserTypes } from '../../model/types';
import { stringToUserType } from '../../model/utils';
import { rampErrorToString } from '../../model/error';
import { truncate } from '../../model/helper';
import { generateConfirmationToken, sendConfirmationEmail, storeTempUserData } from '../../model/emailConfirmation';
import { useUser } from './UserContext';
import DynamicDots from '../ui/DynamicDots';

const RegisterUser: React.FC = () => {
    const [userType, setUserType] = useState<UserTypes>("Onramper");
    const [providers, setProviders] = useState<PaymentProvider[]>([]);
    const [providerType, setProviderType] = useState<PaymentProviderTypes>("PayPal");
    const [providerId, setProviderId] = useState('');
    const [revolutScheme, setRevolutScheme] = useState<revolutSchemeTypes>();
    const [revolutName, setRevolutName] = useState('');
    const [message, setMessage] = useState('');
    const [isLoading, setIsLoading] = useState(false);

    const { setUser: setGlobalUser, user, loginMethod, password, backendActor } = useUser();
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

        let tmpActor = backend;
        if ('ICP' in loginMethod) {
            if (!backendActor) {
                setMessage("Internet Identity not loaded with backend actor")
                return;
            }
            tmpActor = backendActor;
        }

        setIsLoading(true);
        try {
            let result = await tmpActor.register_user(stringToUserType(userType), providers, loginMethod, []);
            if ('Err' in result) {
                setGlobalUser(null);
                setMessage(`Could not register user: ${rampErrorToString(result.Err)}`)
            }
            if ('Ok' in result) {
                navigate('/?auth=true');
            }
        } catch (error) {
            setMessage(`Failed to register user: ${error}`);
        } finally {
            setIsLoading(false);
            setMessage("");
        }
    };

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
        <div className="bg-gray-700 rounded-xl p-8 max-w-md mx-auto shadow-lg space-y-4">
            <div className="text-center mb-8">
                <h2 className="text-white text-2xl font-semibold">Register</h2>
            </div>

            {/* User Type Selection */}
            <div className="flex items-center">
                <label className="block text-white w-32">User Type:</label>
                <select
                    value={userType}
                    onChange={(e) => setUserType(e.target.value as 'Offramper' | 'Onramper')}
                    className="flex-grow w-full px-4 py-2 border border-gray-500 bg-gray-600 outline-none rounded-md focus:ring focus:border-blue-900 text-white"
                >
                    <option value="Offramper">Offramper</option>
                    <option value="Onramper">Onramper</option>
                </select>
            </div>

            {/* Login Address Display */}
            {loginMethod && (
                <div className="flex items-center">
                    <label className="block text-white w-32">Address:</label>
                    <span className="flex-grow w-full px-4 py-2 border border-gray-500 bg-gray-600 rounded-md truncate text-left text-white">
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

            <hr className="border-t border-gray-500 w-full" />

            <div className="flex items-center">
                <label className="block text-white w-32">Provider:</label>
                <select
                    value={providerType}
                    onChange={(e) => setProviderType(e.target.value as PaymentProviderTypes)}
                    className="flex-grow w-full px-4 py-2 border border-gray-500 bg-gray-600 outline-none rounded-md focus:ring focus:border-blue-900 text-white"
                >
                    {providerTypes.map(type => (
                        <option value={type}>{type}</option>
                    ))}
                </select>
            </div>
            <div className="flex items-center">
                <label className="block text-white w-32">ID:</label>
                <input
                    type="text"
                    value={providerId}
                    onChange={(e) => setProviderId(e.target.value)}
                    className="flex-grow w-full px-4 py-2 border border-gray-500 bg-gray-600 outline-none rounded-md focus:ring focus:border-blue-900 text-white"
                />
            </div>

            {providerType === 'Revolut' && (
                <>
                    <div className="flex items-center">
                        <label className="block text-white w-32">Scheme:</label>
                        <select
                            value={revolutScheme}
                            onChange={(e) => setRevolutScheme(e.target.value as revolutSchemeTypes)}
                            className="flex-grow w-full px-4 py-2 border border-gray-500 bg-gray-600 outline-none rounded-md focus:ring focus:border-blue-900 text-white"
                        >
                            <option value="" selected>Select Scheme</option>
                            {revolutSchemes.map(type => (
                                <option value={type}>{type}</option>
                            ))}
                        </select>
                    </div>
                    {userType === 'Offramper' && (
                        <div className="flex items-center">
                            <label className="block text-white w-32">Name:</label>
                            <input
                                type="text"
                                value={revolutName}
                                onChange={(e) => setRevolutName(e.target.value)}
                                className="flex-grow w-full px-4 py-2 border border-gray-500 bg-gray-600 outline-none rounded-md focus:ring focus:border-blue-900 text-white"
                            />
                        </div>
                    )}
                </>
            )}
            <button
                onClick={handleAddProvider}
                className="w-full px-4 py-2 bg-indigo-700 text-white font-semibold rounded-md hover:bg-indigo-800 focus:outline-none focus:ring focus:ring-indigo-500"
            >
                Add Provider
            </button>

            {providers.length > 0 && (
                <div className="mt-4">
                    <ul className="list-none bg-gray-600 p-4 rounded-md text-white">
                        {providers.map((provider, index) => {
                            if ('PayPal' in provider) {
                                return (
                                    <li key={index} className="py-1">
                                        <span className="text-gray-300">(PayPal)</span>
                                        <div>{provider.PayPal.id}</div>
                                    </li>
                                );
                            } else if ('Revolut' in provider) {
                                return (
                                    <li key={index} className="py-1">
                                        <span className="text-gray-300">(Revolut)</span>
                                        <div>{provider.Revolut.id}</div>
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
            )}

            <hr className="border-t border-gray-500 my-6" />

            <div className="flex justify-between items-center">
                <button
                    onClick={() => navigate("/view")}
                    className="px-4 py-2 bg-gray-500 text-white rounded-md hover:bg-gray-600 focus:outline-none focus:ring focus:ring-gray-300"
                >
                    Skip
                </button>

                {isLoading ? (
                    <div className="flex items-center space-x-2">
                        <div className="w-6 h-6 border-t-2 border-b-2 border-indigo-400 rounded-full animate-spin"></div>
                        <div className="text-sm font-medium text-gray-300">Registering<DynamicDots isLoading /></div>
                    </div>
                ) : null}

                <button onClick={handleSubmit} className="px-4 py-2 bg-green-800 text-white rounded-md hover:bg-green-900 focus:outline-none focus:ring focus:ring-green-600">
                    Register
                </button>
            </div>

            {!isLoading && message && <p className="mt-4 text-sm font-medium text-red-600">{message}</p>}
        </div >
    );
};

export default RegisterUser;
