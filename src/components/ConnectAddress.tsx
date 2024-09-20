import React, { useEffect, useState } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import { ethers } from 'ethers';
import { ConnectButton } from '@rainbow-me/rainbowkit';
import { useAccount } from 'wagmi';

import { backend, createActor } from '../declarations/backend';
import { AuthenticationData, LoginAddress } from '../declarations/backend/backend.did';
import { useUser } from './user/UserContext';
import { validatePassword } from '../model/helper';
import { rampErrorToString } from '../model/error';

// Icons
import icpLogo from "../assets/blockchains/icp-logo.svg";
import ethereumLogo from "../assets/blockchains/ethereum-logo.png";
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faEnvelope, faKey, faEye, faEyeSlash } from '@fortawesome/free-solid-svg-icons';
import { handleWeb3Error } from '../model/evm';
import DynamicDots from './ui/DynamicDots';

const ConnectAddress: React.FC = () => {
    const [email, setEmail] = useState('');
    const [password, setPassword] = useState('');
    const [isPasswordVisible, setIsPasswordVisible] = useState(false);
    const [evmDefaultConnect, setEvmDefaultConnect] = useState(false);
    const [loginAttempt, setLoginAttempt] = useState(false);
    const [loadingEmail, setLoadingEmail] = useState(false);
    const [loadingEvm, setLoadingEvm] = useState(false);
    const [loadingIcp, setLoadingIcp] = useState(false);
    const [emailMessage, setEmailMessage] = useState<string | null>(null);
    const [evmMessage, setEvmMessage] = useState<string | null>(null);
    const [iIMessage, setIIMessage] = useState<string | null>(null);

    const [searchParams] = useSearchParams();
    const { isConnected, address } = useAccount();
    const {
        userType,
        loginMethod,
        icpAgent,
        principal,
        setLoginMethod,
        setUser,
        loginInternetIdentity,
        authenticateUser
    } = useUser();
    const navigate = useNavigate();

    useEffect(() => {
        if (userType == "Onramper") {
            navigate("/view");
        } else if (userType == "Offramper") {
            navigate("/create");
        }
    }, [userType]);

    useEffect(() => {
        const isAuth = searchParams.get('auth') === 'true';
        const pwd = searchParams.get('pwd');
        const email = searchParams.get('email');

        const performLogin = async () => {
            setLoginAttempt(true);
            try {
                if (loginMethod && 'EVM' in loginMethod) {
                    await handleEvmLogin();
                } else if (loginMethod && 'ICP' in loginMethod) {
                    await handleInternetIdentityLogin(true);
                } else if (pwd && email) {
                    await handleEmailLogin(email, pwd);
                } else {
                    console.error("unknown login method");
                }
            } catch (error) {
                console.error("Error during login:", error);
            }
        }

        if (isAuth && !loginAttempt) {
            performLogin();
        }
    }, [loginMethod])

    useEffect(() => {
        if (isConnected && evmDefaultConnect && !(loadingEvm || loadingEmail || loadingIcp)) {
            handleEvmLogin();
        }
    }, [isConnected, evmDefaultConnect]);

    const cleanMessages = () => {
        setEmailMessage(null);
        setEvmMessage(null);
        setIIMessage(null);
        setLoadingEmail(false);
        setLoadingEvm(false);
        setLoadingIcp(false);
    };

    const handleEvmLogin = async () => {
        cleanMessages();

        if (!window.ethereum) throw new Error('No crypto wallet found.');

        if (!address) {
            setEvmMessage("Undefined evm address. Please connect your wallet.");
            return;
        }

        const loginAddress: LoginAddress = { EVM: { address } };
        setLoginMethod(loginAddress)

        setLoadingEvm(true);
        try {
            const result = await backend.generate_evm_auth_message({ EVM: { address } });

            if ('Ok' in result) { // user exists, we need to verify the signature
                const provider = new ethers.BrowserProvider(window.ethereum);
                const signer = await provider.getSigner();

                signer.signMessage(result.Ok)
                    .then(async (signature) => {
                        try {
                            const result = await authenticateUser(loginAddress, { signature: [signature], password: [] });
                            if ('Ok' in result) {
                                navigate('Offramper' in result.Ok.user_type ? "/create" : "/view");
                            } else {
                                setEvmMessage(`Failed to authenticate user: ${rampErrorToString(result.Err)}`);
                                setLoginMethod(null);
                                setLoadingEvm(false);
                            }
                        } catch (authError: any) {
                            setEvmMessage(authError.message || 'Unknown authentication error occurred');
                            setLoginMethod(null);
                            setLoadingEvm(false);
                        }
                    })
                    .catch((error) => {
                        setEvmMessage(handleWeb3Error(error));
                        setLoginMethod(null);
                        setLoadingEvm(false);
                    });
            } else if ("UserNotFound" in result.Err) {
                navigate("/register");
            } else {
                setEvmMessage(`Internal error when generating evm auth session message: ${rampErrorToString(result.Err)}`)
                setLoginMethod(null);
                setLoadingEvm(false);
            }
        } catch (error: any) {
            setEvmMessage(`An unexpected error occurred: ${error.message || 'Unknown error'}`);
            setLoginMethod(null);
            setLoadingEvm(false);
        }
    }

    const handleEmailLogin = async (loginEmail: string, loginPassword: string) => {
        cleanMessages();

        const passwordError = validatePassword(loginPassword);
        if (passwordError) {
            setEmailMessage(passwordError);
            return;
        }
        if (loginEmail && loginPassword) {
            setLoadingEmail(true);

            const loginAddress: LoginAddress = {
                Email: { email: loginEmail }
            };
            setLoginMethod(loginAddress, loginPassword);

            const authData: AuthenticationData = {
                signature: [],
                password: [loginPassword]
            }
            try {
                const result = await authenticateUser(loginAddress, authData);
                if ('Ok' in result) {
                    setUser(result.Ok)
                    if ('Offramper' in result.Ok.user_type) {
                        navigate("/create");
                    } else {
                        navigate("/view");
                    }
                } else if ('Err' in result && 'InvalidPassword' in result.Err) {
                    setEmailMessage('Invalid password.');
                    setLoginMethod(null);
                    setLoadingEmail(false);
                } else {
                    navigate("/register");
                }
            } catch (error) {
                console.log("error = ", error);
                setEmailMessage("Failed to login user.");
                setLoginMethod(null);
                setLoadingEmail(false);
            }
        } else {
            console.error("Email and Password are required");
            setEmailMessage('Please introduce your email and password.');
            setLoginMethod(null);
            setLoadingEmail(false);
        }
    };

    const handleInternetIdentityLogin = async (autoLogin?: boolean) => {
        cleanMessages();

        if (!process.env.CANISTER_ID_BACKEND) throw new Error("Backend Canister ID not in env file");
        try {
            setLoadingIcp(true);

            let loginPrincipal = principal;
            let loginAgent = icpAgent;
            if (!(principal && icpAgent) || !autoLogin) {
                [loginPrincipal, loginAgent] = await loginInternetIdentity();
            }
            if (!loginPrincipal) throw new Error("Principal not set after II login");
            if (!loginAgent) throw new Error("ICP Agent not set after II login");

            const backendActor = createActor(process.env.CANISTER_ID_BACKEND, { agent: loginAgent });
            const loginAddress: LoginAddress = {
                ICP: { principal_id: loginPrincipal.toText() }
            };
            setLoginMethod(loginAddress);

            const result = await authenticateUser(loginAddress, undefined, backendActor);
            if ('Ok' in result) {
                if ('Offramper' in result.Ok.user_type) {
                    navigate("/create");
                } else {
                    navigate("/view");
                }
            } else if ('UnauthorizedPrincipal' in result.Err) {
                setIIMessage('Could not authorize agent.');
                setLoadingIcp(false);
                setLoginMethod(null);
            } else {
                navigate("/register");
            }
        } catch (error) {
            const errMessage = `Error authenticating user identity: ${error}`;
            console.error(errMessage);
            setIIMessage(errMessage);
            setLoadingIcp(false);
            setLoginMethod(null);
        }
    };

    return (
        <div className="bg-gray-700 rounded-xl p-8 max-w-md mx-auto">
            <div className="text-center mb-6">
                <h2 className="text-white text-2xl font-semibold">Sign in to icRamp</h2>
            </div>

            {/* <div className="space-y-4"> */}
            {/* Internet Identity Login */}
            <div
                className={`flex items-center justify-between px-3 py-3 bg-gray-600 rounded-md
                        ${loadingEmail || loadingEvm || loadingIcp ? 'cursor-not-allowed' : 'cursor-pointer hover:bg-gray-500'}`}
                onClick={() => !(loadingEmail || loadingEvm || loadingIcp) ? handleInternetIdentityLogin(false) : undefined}
            >
                <div className="flex items-center space-x-3">
                    <img src={icpLogo} alt="ICP Logo" className="h-6 w-6 mr-2" />

                    <span className="text-white text-lg">
                        {loadingIcp ?
                            <span>Checking Internet Identity<DynamicDots isLoading={loadingIcp} /></span>
                            : <span>Sign in with Internet Identity</span>
                        }
                    </span>
                </div>
                {loadingIcp && <div className="w-4 h-4 border-t-2 border-b-2 border-indigo-400 rounded-full animate-spin ml-3"></div>}
            </div>
            {iIMessage && <p className="mt-1 text-sm font-medium text-red-500 break-all">{iIMessage}</p>}

            {/* Wallet Login */}
            <ConnectButton.Custom>
                {({ openConnectModal }) => (
                    <div
                        className={`mt-4 flex items-center justify-between px-3 py-3 bg-gray-600 rounded-md 
                                ${loadingEmail || loadingEvm || loadingIcp ? 'cursor-not-allowed' : 'cursor-pointer hover:bg-gray-500'}
                            `}
                        onClick={() => {
                            if (!(loadingEmail || loadingEvm || loadingIcp)) {
                                if (isConnected) {
                                    handleEvmLogin()
                                } else {
                                    setEvmDefaultConnect(true);
                                    openConnectModal();
                                }
                            }
                        }}>
                        <div
                            className="flex items-center space-x-3"
                        >
                            <img src={ethereumLogo} alt="Ethereum Logo" className="h-6 w-6 mr-2" />
                            <span className="text-white text-lg w-full text-left">
                                {loadingEvm ?
                                    <span>Checking address<DynamicDots isLoading={loadingEvm} /></span>
                                    : <span>Login with Ethereum</span>
                                }
                            </span>

                        </div>
                        {loadingEvm && <div className="w-4 h-4 border-t-2 border-b-2 border-indigo-400 rounded-full animate-spin ml-3"></div>}
                    </div>
                )}
            </ConnectButton.Custom>
            {evmMessage && <p className="mt-1 text-sm font-medium text-red-500 break-all">{evmMessage}</p>}
            {/* </div > */}

            <hr className="border-t border-gray-500 w-full my-6" />

            {/* Email Login */}
            <form
                onSubmit={(event) => {
                    event.preventDefault();
                    !(loadingEmail || loadingEvm || loadingIcp) && password ? handleEmailLogin(email, password) : undefined
                }}
            >
                <div className="space-y-4">
                    <div className="flex items-center space-x-3 px-3 py-2 bg-gray-600 rounded-md">
                        <FontAwesomeIcon icon={faEnvelope} className="text-white h-5 w-5" />
                        <input
                            type="email"
                            value={email}
                            onChange={(e) => setEmail(e.target.value)}
                            placeholder="Enter your email"
                            className="px-3 py-1 bg-transparent text-white border-none outline-none w-full"
                            required
                        />
                    </div>
                    <div className="flex items-center space-x-3 px-3 py-2 bg-gray-600 rounded-md">
                        <FontAwesomeIcon icon={faKey} className="text-white h-5 w-5" />
                        <input
                            type={isPasswordVisible ? 'text' : 'password'}
                            value={password}
                            onChange={(e) => setPassword(e.target.value)}
                            placeholder="Enter your password"
                            className="px-3 py-1 bg-transparent text-white border-none outline-none w-full"
                            required
                        />
                        <button
                            type="button"
                            className="px-2 py-1"
                            onClick={() => setIsPasswordVisible(!isPasswordVisible)}
                        >
                            <FontAwesomeIcon icon={isPasswordVisible ? faEyeSlash : faEye} className="text-gray-300 h-5 w-5" />
                        </button>
                        <style>
                            {`
                            input:-webkit-autofill {
                                background-color: #4b5563 !important; /* bg-gray-600 */
                                -webkit-text-fill-color: white !important;
                                transition: background-color 5000s ease-in-out 0s;
                            }
                            input:-webkit-autofill:focus {
                                background-color: #4b5563 !important;
                                -webkit-text-fill-color: white !important;
                            }
                        `}
                        </style>
                    </div>
                    <button
                        type="submit"
                        disabled={loadingEmail || loadingEvm || loadingIcp}
                        className={`w-full py-3 bg-amber-800 text-white rounded-md hover:bg-amber-900 focus:outline-none focus:ring focus:ring-amber-400 
                        ${loadingEmail || loadingEvm || loadingIcp
                                ? 'cursor-not-allowed' : ''
                            }`}>
                        {loadingEmail ? (
                            <div className="flex items-center justify-center space-x-2 relative text-base">
                                <span>Checking email<DynamicDots isLoading={loadingEmail} /></span>
                                <div className="absolute right-3 w-4 h-4 border-t-2 border-b-2 border-white rounded-full animate-spin"></div>
                            </div>
                        ) : (
                            <div className="text-base">Login with Email</div>
                        )}
                    </button>
                </div>
                <div className="text-center mt-2">
                    <a
                        href="#"
                        onClick={() => !(loadingEmail || loadingEvm || loadingIcp) && navigate('/forgot-password')}
                        className="text-sm text-gray-400 hover:underline"
                    >
                        Forgot your password?
                    </a>
                </div>
            </form>

            {emailMessage && <p className="mt-2 text-sm font-medium text-red-500 break-all">{emailMessage}</p>}
        </div >
    );
};

export default ConnectAddress;