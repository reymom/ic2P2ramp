import React, { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
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

const ConnectAddress: React.FC = () => {
    const [email, setEmail] = useState('');
    const [password, setPassword] = useState('');
    const [isPasswordVisible, setIsPasswordVisible] = useState(false);
    const [loadingEmail, setLoadingEmail] = useState(false);
    const [loadingEvm, setLoadingEvm] = useState(false);
    const [loadingIcp, setLoadingIcp] = useState(false);
    const [emailMessage, setEmailMessage] = useState('');
    const [evmMessage, setEvmMessage] = useState('');
    const [iIMessage, setIIMessage] = useState('');

    const { isConnected, address } = useAccount();
    const {
        userType,
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

    const handleEvmLogin = async () => {
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
                const signature = await signer.signMessage(result.Ok);
                try {
                    const result = await authenticateUser(loginAddress, { signature: [signature], password: [] });
                    if ('Ok' in result) {
                        setUser(result.Ok);
                        navigate('Offramper' in result.Ok.user_type ? "/create" : "/view");
                    } else {
                        setEvmMessage(`Failed to authenticate user: ${rampErrorToString(result.Err)}`)
                    }
                } catch (error) {
                    setEvmMessage(`Failed to authenticate user: ${error}`);
                }
            } else if ("UserNotFound" in result.Err) {
                navigate("/register");
            } else {
                setEvmMessage(`Internal error when generating evm session nonce: ${rampErrorToString(result.Err)}`)
            }
        } catch (error) {
            setEvmMessage(`Failed to generate evm nonce: ${error}`);
            setLoginMethod(null);
        } finally {
            setLoadingEvm(false);
        }
    }

    const handleEmailLogin = async (event: React.FormEvent<HTMLFormElement>) => {
        event.preventDefault();

        const passwordError = validatePassword(password);
        if (passwordError) {
            setEmailMessage(passwordError);
            return;
        }
        if (email && password) {
            setLoadingEmail(true);

            const loginAddress: LoginAddress = {
                Email: { email }
            };
            setLoginMethod(loginAddress, password);

            const authData: AuthenticationData = {
                signature: [],
                password: [password]
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
                    setEmailMessage('Invalid password');
                } else {
                    navigate("/register");
                }
            } catch (error) {
                console.log("error = ", error);
                setEmailMessage("Failed to login user");
            } finally {
                setLoadingEmail(false);
            }
        } else {
            console.error("Email and Password are required");
        }
    };

    const handleInternetIdentityLogin = async () => {
        if (!process.env.CANISTER_ID_BACKEND) throw new Error("Backend Canister ID not in env file");
        try {
            setLoadingIcp(true);
            const [principal, agent] = await loginInternetIdentity();
            if (!principal) throw new Error("Principal not set after II login");
            if (!agent) throw new Error("ICP Agent not set after II login");
            const backendActor = createActor(process.env.CANISTER_ID_BACKEND, { agent });

            const loginAddress: LoginAddress = {
                ICP: { principal_id: principal.toText() }
            };
            setLoginMethod(loginAddress);

            const result = await authenticateUser(loginAddress, undefined, backendActor);
            console.log("[authenticateUser] result = ", result);

            if ('Ok' in result) {
                setUser(result.Ok);
                if ('Offramper' in result.Ok.user_type) {
                    navigate("/create");
                } else {
                    navigate("/view");
                }
            } else if ('UnauthorizedPrincipal' in result.Err) {
                setIIMessage('Could not authorize agent.')
            } else {
                navigate("/register");
            }
        } catch (error) {
            console.error(`Error authenticating user identity: ${error}`);
        } finally {
            setLoadingIcp(false);
        }
    };

    return (
        <div className="bg-gray-700 rounded-xl p-8 max-w-md mx-auto space-y-6">
            <div className="text-center mb-6">
                <h2 className="text-white text-2xl font-semibold">Sign in to icRamp</h2>
            </div>

            <div className="space-y-4">
                <div
                    className="flex items-center space-x-3 px-3 py-2 bg-gray-600 rounded-md hover:bg-gray-500 cursor-pointer"
                    onClick={handleInternetIdentityLogin}
                >
                    <img src={icpLogo} alt="ICP Logo" className="h-6 w-6 mr-2" />
                    <span className="text-white text-lg">Sign in with Internet Identity</span>
                </div>
                {loadingIcp && (
                    <div className="my-2 flex justify-center items-center space-x-2">
                        <div className="w-4 h-4 border-t-2 border-b-2 border-indigo-600 rounded-full animate-spin"></div>
                        <div className="text-sm font-medium text-gray-300">Checking ICP principal...</div>
                    </div>
                )}
                {iIMessage && <p className="my-2 text-sm font-medium text-red-500 break-all">{iIMessage}</p>}

                <div className="flex items-center space-x-3 px-3 py-2 bg-gray-600 rounded-md hover:bg-gray-500 cursor-pointer">
                    <img src={ethereumLogo} alt="Ethereum Logo" className="h-6 w-6 mr-2" />
                    {!isConnected && (
                        <div className="w-full text-left">
                            <ConnectButton.Custom>
                                {({ openConnectModal }) => (
                                    <button
                                        className="text-white w-full text-lg text-left"
                                        onClick={openConnectModal}
                                    >
                                        Connect your wallet
                                    </button>
                                )}
                            </ConnectButton.Custom>
                        </div>
                    )}
                    {isConnected && (
                        <button onClick={handleEvmLogin} className="text-white text-lg w-full text-left">Login with Ethereum</button>
                    )}
                </div>
                {loadingEvm && (
                    <div className="my-2 flex justify-center items-center space-x-2">
                        <div className="w-4 h-4 border-t-2 border-b-2 border-indigo-600 rounded-full animate-spin"></div>
                        <div className="text-sm font-medium text-gray-300">Checking address...</div>
                    </div>
                )}
                {evmMessage && <p className="my-2 text-sm font-medium text-red-500 break-all">{evmMessage}</p>}
            </div>

            <hr className="border-t border-gray-500 w-full my-4" />

            <form onSubmit={handleEmailLogin} className="space-y-4">
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
                <div className="flex items-center space-x-3 px-3 py-2 bg-gray-600 rounded-md text-xl">
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
                <button type="submit" className="w-full bg-amber-800 text-white py-3 rounded-md hover:bg-amber-900">
                    Log in with Email
                </button>
                <div className="text-center -mt-1">
                    <a href="#" onClick={() => navigate('/forgot-password')} className="text-sm text-gray-400 hover:underline">
                        Forgot your password?
                    </a>
                </div>
            </form>

            {loadingEmail && (
                <div className="my-2 flex justify-center items-center space-x-2">
                    <div className="w-4 h-4 border-t-2 border-b-2 border-indigo-600 rounded-full animate-spin"></div>
                    <div className="text-sm font-medium text-gray-300">Checking email...</div>
                </div>
            )}
            {emailMessage && <p className="my-2 text-sm font-medium text-red-500 break-all">{emailMessage}</p>}
        </div>
    );
};

export default ConnectAddress;