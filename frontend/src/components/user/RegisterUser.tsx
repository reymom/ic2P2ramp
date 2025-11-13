import clsx from 'clsx';
import React, { useEffect, useMemo, useState } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';

import { LoginAddress, PaymentProvider } from '@/declarations/icramp_backend/icramp_backend.did';
import { backend } from '@/model/backendProxy';
import { PaymentProviderTypes, revolutSchemeTypes, revolutSchemes, UserTypes } from '@/model/types';
import { clearTempUserData, generateConfirmationToken, getTempUserData, sendConfirmationEmail, storeTempUserData } from '@/model/emailConfirmation';
import { stringToUserType } from '@/model/helpers/types';
import { rampErrorToString } from '@/model/helpers/error';
import { truncate } from '@/utils/formatters';
import { mapCountryToPlatform } from '@/utils/stripe';
import { startStripeKyc } from '@/hooks/useStripeKyc';
import { useUser } from './UserContext';
import DynamicDots from '@/components/ui/DynamicDots';
import { ProviderIcon } from '../ui/ProviderIcon';

const RegisterUser: React.FC = () => {
    const [userType, setUserType] = useState<UserTypes>("Onramper");
    const [providers, setProviders] = useState<PaymentProvider[]>([]);
    const [providerType, setProviderType] = useState<PaymentProviderTypes>("PayPal");
    const [providerId, setProviderId] = useState('');
    const [revolutScheme, setRevolutScheme] = useState<revolutSchemeTypes>();
    const [revolutName, setRevolutName] = useState('');
    const [stripeCountry, setStripeCountry] = useState('ES');
    const [stripeReady, setStripeReady] = useState(false);
    const [stripeInfoMsg, setStripeInfoMsg] = useState('');

    const [message, setMessage] = useState('');
    const [isLoading, setIsLoading] = useState(false);
    const [loadingStripe, setLoadingStripe] = useState(false);

    const { setUser: setGlobalUser, user, loginMethod, setLoginMethod, password, backendActor } = useUser();
    const navigate = useNavigate();
    const [searchParams] = useSearchParams();

    const canRegister = providers.length > 0 && !isLoading;
    const visibleProviderTypes = useMemo<PaymentProviderTypes[]>(() => {
        // Offramper: Stripe/PayPal/Revolut (no Email)
        // Onramper: only Email (no Stripe)
        return userType === 'Offramper'
            ? (['PayPal', 'Revolut', 'Stripe'] as PaymentProviderTypes[])
            : (['PayPal', 'Revolut', 'Email'] as PaymentProviderTypes[]);
    }, [userType]);

    // Default the selector when switching user type
    useEffect(() => {
        if (userType === 'Onramper') {
            setProviders((ps) => ps.filter((p) => !('Stripe' in p)));
        }
    }, [userType]);

    useEffect(() => {
        if (user) {
            console.log(`user: ${user}`)
            navigate("/")
            return;
        }
    }, [user])

    useEffect(() => {
        if (!loginMethod && !getTempUserData()) {
            navigate("/")
            return;
        }
    }, [loginMethod])

    const restoreTempData = () => {
        const t = getTempUserData();
        const acct = searchParams.get('stripe_acct');
        if (t && acct) {
            if (t.providers?.length) setProviders(t.providers);
            if (t.userType) setUserType(t.userType);
            if (t.loginMethod) setLoginMethod(t.loginMethod, t.password ?? null);
            return true;
        } else if (t && t.loginMethod && loginMethod && t.loginMethod !== loginMethod) {
            clearTempUserData();
        }
        return false;
    };

    useEffect(() => {
        restoreTempData();
        const acct = searchParams.get('stripe_acct');
        const platform = searchParams.get('platform');
        if (!acct || !platform) return;

        (async () => {
            try {
                setProviders((prev) => {
                    const next = prev.some((p) => 'Stripe' in p && p.Stripe.account_id === acct)
                        ? prev
                        : [...prev, { Stripe: { account_id: acct, platform: mapCountryToPlatform(stripeCountry) } }];
                    const t = getTempUserData();
                    if (t) storeTempUserData({ ...t, providers: next });
                    return next;
                });
                setStripeReady(true);
                setStripeInfoMsg('Stripe account added.');
            } catch (e) {
                setMessage(`Failed to verify Stripe account: ${e}`);
            }
        })();
    }, [searchParams]);

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
        } else if (providerType === 'Stripe') {
            if (userType === 'Onramper') {
                setMessage('Stripe is just needed for Offrampers');
                return;
            }
            setMessage('Use "Start Stripe Onboarding" to add a Stripe provider.');
            return;
        } else if (providerType === 'Email') {
            if (userType === 'Offramper') {
                setMessage('Email available just for Onrampers');
                return;
            }
            const email = providerId.trim();
            const ok =
                email.length <= 254 &&
                email.includes('@') &&
                !email.includes(' ') &&
                !email.startsWith('@') &&
                !email.endsWith('@');
            if (!ok) {
                setMessage('Invalid email.');
                return;
            }
            newProvider = { Email: { email } };
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

        let login: LoginAddress;
        const t = getTempUserData();
        const acct = searchParams.get('stripe_acct');
        if (t && acct && !loginMethod) {
            setLoginMethod(t.loginMethod, t.password ?? null);
            login = t.loginMethod;
        } else if (!loginMethod) {
            console.log('no login method');
            navigate("/")
            return;
        } else {
            login = loginMethod;
        }

        if ('Email' in login) {
            await handleEmailConfirmation();
            return;
        }

        let tmpActor = backend;
        if ('ICP' in login) {
            if (!backendActor) {
                setMessage("Internet Identity not loaded with backend actor")
                return;
            }
            tmpActor = backendActor;
        }

        setIsLoading(true);
        try {
            let result = await tmpActor.register_user(stringToUserType(userType), providers, login, []);
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

    const startStripeOnboarding = async () => {
        await startStripeKyc({
            userType,
            loginMethod,
            backendActor,
            email: providerId,
            country: stripeCountry,
            providers,
            password: password ?? undefined,
            storeUserData: true,
            setMessage,
            setLoading: setLoadingStripe,
        });
    };

    return (
        <div className="bg-gray-200 dark:bg-gray-700 text-gray-900 dark:text-white rounded-xl p-8 max-w-md mx-auto shadow-lg space-y-4">
            <div className="text-center mb-8">
                <h2 className="text-2xl font-semibold">Register</h2>
            </div>

            {/* User Type Selection */}
            <div className="flex items-center">
                <label className="block w-32">User Type:</label>
                <select
                    value={userType}
                    onChange={(e) => setUserType(e.target.value as 'Offramper' | 'Onramper')}
                    className="flex-grow w-full px-4 py-2 bg-gray-300 dark:bg-gray-600 border border-gray-500 outline-none rounded-md focus:ring focus:border-blue-900"
                >
                    <option value="Offramper">Offramper</option>
                    <option value="Onramper">Onramper</option>
                </select>
            </div>

            {/* Login Address Display */}
            {loginMethod && (
                <div className="flex items-center">
                    <label className="block w-32">Address:</label>
                    <span className="flex-grow w-full px-4 py-2 bg-gray-300 dark:bg-gray-600 border border-gray-500 rounded-md truncate text-left">
                        {(() => {
                            if ('EVM' in loginMethod) {
                                return truncate(loginMethod.EVM.address, 12, 10);
                            } else if ('Bitcoin' in loginMethod) {
                                return truncate(loginMethod.Bitcoin.address, 12, 10);
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
                <label className="block w-32">Provider:</label>
                <select
                    value={providerType}
                    onChange={(e) => setProviderType(e.target.value as PaymentProviderTypes)}
                    className="flex-grow w-full px-4 py-2 bg-gray-300 dark:bg-gray-600 border border-gray-500 outline-none rounded-md focus:ring focus:border-blue-900"
                >
                    {visibleProviderTypes.map(type => (
                        <option value={type}>{type}</option>
                    ))}
                </select>
            </div>

            <div className="flex items-center">
                <label className="block w-32">Email:</label>
                <input
                    type="text"
                    value={providerId}
                    onChange={(e) => setProviderId(e.target.value)}
                    className="flex-grow w-full px-4 py-2 bg-gray-300 dark:bg-gray-600 border border-gray-500 outline-none rounded-md focus:ring focus:border-blue-900"
                />
            </div>

            {providerType === 'Revolut' && (
                <>
                    <div className="flex items-center">
                        <label className="block w-32">Scheme:</label>
                        <select
                            value={revolutScheme}
                            onChange={(e) => setRevolutScheme(e.target.value as revolutSchemeTypes)}
                            className="flex-grow w-full px-4 py-2 bg-gray-300 dark:bg-gray-600 border border-gray-500 outline-none rounded-md focus:ring focus:border-blue-900"
                        >
                            <option value="">Select Scheme</option>
                            {revolutSchemes.map(type => (
                                <option value={type}>{type}</option>
                            ))}
                        </select>
                    </div>
                    {userType === 'Offramper' && (
                        <div className="flex items-center">
                            <label className="block w-32">Name:</label>
                            <input
                                type="text"
                                value={revolutName}
                                onChange={(e) => setRevolutName(e.target.value)}
                                className="flex-grow w-full px-4 py-2 bg-gray-300 dark:bg-gray-600 border border-gray-500 outline-none rounded-md focus:ring focus:border-blue-900"
                            />
                        </div>
                    )}
                </>
            )}

            {providerType === 'Stripe' && (
                <>
                    <div className="flex items-center">
                        <label className="block w-32">Country:</label>
                        <input
                            type="text"
                            value={stripeCountry}
                            onChange={(e) => setStripeCountry(e.target.value.toUpperCase())}
                            placeholder="ES, US, …"
                            className="flex-grow w-full px-4 py-2 bg-gray-300 dark:bg-gray-600 border border-gray-500 outline-none rounded-md focus:ring focus:border-blue-900"
                        />
                    </div>

                    <div
                        className={clsx(
                            "relative w-full flex items-center justify-center px-4 py-2",
                            "bg-purple-600 dark:bg-purple-800 font-semibold rounded-md",
                            (loadingStripe || stripeReady)
                                ? "opacity-60 cursor-not-allowed"
                                : "cursor-pointer hover:bg-purple-500 dark:hover:bg-purple-900",
                            "focus:outline-none focus:ring focus:ring-purple-500"
                        )}
                        onClick={() => !isLoading ? startStripeOnboarding() : undefined}
                    >
                        <span className="text-lg pointer-events-none">
                            {loadingStripe
                                ? <>Setting up Stripe<DynamicDots isLoading={loadingStripe} /></>
                                : stripeReady ? "Stripe Ready" : "Start Stripe Onboarding"}
                        </span>
                        {loadingStripe && (
                            <div className="absolute right-3 w-4 h-4 border-t-2 border-b-2 border-purple-700 rounded-full animate-spin" />
                        )}
                    </div>
                </>
            )}

            {providerType !== "Stripe" && <button
                onClick={handleAddProvider}
                className="w-full px-4 py-2 bg-indigo-600 dark:bg-indigo-800 font-semibold rounded-md hover:bg-indigo-500 dark:hover:bg-indigo-900 focus:outline-none focus:ring focus:ring-indigo-500">
                Add Provider
            </button>
            }

            {(providers.length > 0) && (
                <div className="mt-4 grid grid-cols-1 gap-3">
                    {providers.map((provider, index) => {
                        if ('PayPal' in provider) {
                            return (
                                <div key={index} className="rounded-lg border border-gray-500/40 bg-gray-300/40 dark:bg-gray-800/60 p-3">
                                    <div className="flex items-center gap-2">
                                        <ProviderIcon type="PayPal" />
                                        <span className="text-sm text-gray-600 dark:text-gray-300">PayPal</span>
                                    </div>
                                    <div className="mt-1 font-mono text-sm break-all">{provider.PayPal.id}</div>
                                </div>
                            );
                        } else if ('Revolut' in provider) {
                            return (
                                <div key={index} className="rounded-lg border border-gray-500/40 bg-gray-300/40 dark:bg-gray-800/60 p-3">
                                    <div className="flex items-center gap-2">
                                        <ProviderIcon type="Revolut" />
                                        <span className="text-sm text-gray-600 dark:text-gray-300">Revolut</span>
                                    </div>
                                    <div className="mt-1 font-mono text-sm break-all">{provider.Revolut.id}</div>
                                    <div className="text-xs text-gray-500">Scheme: {provider.Revolut.scheme}</div>
                                    {provider.Revolut.name?.[0] && (
                                        <div className="text-xs text-gray-500">Name: {provider.Revolut.name[0]}</div>
                                    )}
                                </div>
                            );
                        } else if ('Stripe' in provider) {
                            return (
                                <div key={index} className="rounded-lg border border-purple-500/50 bg-purple-500/10 p-3">
                                    <div className="flex items-center gap-2">
                                        <ProviderIcon type="Stripe" />
                                        <span className="text-sm text-gray-600 dark:text-gray-300">Stripe</span>
                                    </div>
                                    <div className="mt-1 font-mono text-sm break-all">{provider.Stripe.account_id}</div>
                                </div>
                            );
                        } else if ('Email' in provider) {
                            return (
                                <div key={index} className="rounded-lg border border-gray-500/40 bg-gray-300/40 dark:bg-gray-800/60 p-3">
                                    <div className="flex items-center gap-2">
                                        <ProviderIcon type="Email" />
                                        <span className="text-sm text-gray-600 dark:text-gray-300">Email</span>
                                    </div>
                                    <div className="mt-1 font-mono text-sm break-all">{provider.Email.email}</div>
                                </div>
                            );
                        }
                        return null;
                    })}

                    {/* Success notice below cards, separate from error messages */}
                    {stripeReady && stripeInfoMsg && (
                        <div className="text-sm text-green-400">{stripeInfoMsg}</div>
                    )}
                </div>
            )}

            <hr className="border-t border-gray-400 dark:border-gray-500 my-6" />

            <div className="flex justify-between items-center">
                <button
                    onClick={() => navigate("/view")}
                    className="px-4 py-2 bg-gray-400 rounded-md hover:bg-gray-500 focus:outline-none focus:ring focus:ring-gray-300"
                >
                    Skip
                </button>

                {isLoading ? (
                    <div className="flex items-center space-x-2">
                        <div className="w-6 h-6 border-t-2 border-b-2 border-indigo-400 rounded-full animate-spin"></div>
                        <div className="text-sm font-medium text-gray-300">Registering<DynamicDots isLoading /></div>
                    </div>
                ) : null}

                <button
                    onClick={handleSubmit}
                    disabled={!canRegister}
                    className="px-4 py-2 bg-green-500 dark:bg-green-700 hover:bg-green-400 dark:hover:bg-green-800 rounded-md focus:outline-none focus:ring focus:ring-green-600 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                    Register
                </button>
            </div>

            {!isLoading && message && <p className="mt-4 text-sm font-medium text-red-600">{message}</p>}
        </div >
    );
};

export default RegisterUser;
