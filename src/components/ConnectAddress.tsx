import React, { useEffect, useState } from 'react';
import { ConnectButton } from '@rainbow-me/rainbowkit';
import { useAccount } from 'wagmi';
import { useUser } from './user/UserContext';
import { AuthenticationData, LoginAddress } from '../declarations/backend/backend.did';
import { useNavigate } from 'react-router-dom';
import { AuthClient } from '@dfinity/auth-client';
import { HttpAgent } from '@dfinity/agent';
import { icpHost, iiUrl } from '../model/icp';
import { validatePassword } from '../model/helper';
import { ethers } from 'ethers';
import { backend } from '../declarations/backend';
import { rampErrorToString } from '../model/error';

const ConnectAddress: React.FC = () => {
    const [email, setEmail] = useState('');
    const [password, setPassword] = useState('');
    const [loadingEmail, setLoadingEmail] = useState(false);
    const [loadingEvm, setLoadingEvm] = useState(false);
    const [loadingIcp, setLoadingIcp] = useState(false);
    const [emailMessage, setEmailMessage] = useState('');
    const [evmMessage, setEvmMessage] = useState('');

    const { isConnected, address } = useAccount();
    const { userType, setLoginMethod, setIcpAgent, authenticateUser, setUser, setPrincipal } = useUser();
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
            const result = await backend.generate_evm_session_nonce({ EVM: { address } });
            if ('Ok' in result) { // user exists, we need to verify the signature
                console.log("result.Ok = ", result.Ok);
                const provider = new ethers.BrowserProvider(window.ethereum);
                const signer = await provider.getSigner();
                const signature = await signer.signMessage(result.Ok);

                const recoveredAddress = ethers.verifyMessage(result.Ok, signature);


                console.log('Expected address:', address);
                console.log('Recovered address:', recoveredAddress);

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
            setEvmMessage(`Failed to generate evm nonce: {error}`);
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
        const authClient = await AuthClient.create();
        await authClient.login({
            identityProvider: iiUrl,
            onSuccess: async () => {
                setLoadingIcp(true);

                const identity = authClient.getIdentity();
                const principal = identity.getPrincipal();
                setPrincipal(principal);
                console.log("Principal connected = ", principal.toString());

                const agent = new HttpAgent({ identity, host: icpHost });
                if (process.env.FRONTEND_ICP_ENV === 'test') {
                    agent.fetchRootKey();
                }
                setIcpAgent(agent);

                const loginAddress: LoginAddress = {
                    ICP: { principal_id: principal.toText() }
                };
                setLoginMethod(loginAddress);

                try {
                    const result = await authenticateUser(loginAddress);
                    if ('Ok' in result) {
                        setUser(result.Ok)
                        if ('Offramper' in result.Ok.user_type) {
                            navigate("/create");
                        } else {
                            navigate("/view");
                        }
                    } else {
                        navigate("/register");
                    }
                } catch (error) {
                    console.error("error authenticating user identity")
                    throw error;
                } finally {
                    setLoadingIcp(false);
                }
            },
            onError: (error) => {
                console.error("Internet Identity login failed:", error);
            },
        });
    };

    return (
        <div className="flex flex-col items-center justify-center text-center h-full rounded">

            <div className="my-4">
                <button onClick={handleInternetIdentityLogin} className="px-2 py-2 bg-green-500 text-white rounded-xl">
                    Log in with Internet Identity
                </button>
                {loadingIcp && (
                    <div className="my-2 flex justify-center items-center space-x-2">
                        <div className="w-4 h-4 border-t-2 border-b-2 border-indigo-600 rounded-full animate-spin"></div>
                        <div className="text-sm font-medium text-gray-700">Checking icp principal...</div>
                    </div>
                )}
            </div>

            <hr className="border-t border-gray-300 w-full my-4" />

            <div className="my-4">
                {!isConnected && <ConnectButton label={"Connect Wallet"} />}
                {isConnected && (
                    <button onClick={handleEvmLogin} className="px-4 py-2 bg-blue-500 text-white rounded">
                        Login with Ethereum
                    </button>
                )}
                {loadingEvm ? (
                    <div className="my-2 flex justify-center items-center space-x-2">
                        <div className="w-4 h-4 border-t-2 border-b-2 border-indigo-600 rounded-full animate-spin"></div>
                        <div className="text-sm font-medium text-gray-700">Checking user address...</div>
                    </div>
                ) : (
                    evmMessage && <p className="my-2 text-sm font-medium text-red-500 break-all">{evmMessage}</p>
                )}
            </div>

            <hr className="border-t border-gray-300 w-full my-4" />

            <div className="my-4 flex flex-col space-y-2 w-full max-w-xs">
                <form onSubmit={handleEmailLogin}>
                    <input
                        type="email"
                        value={email}
                        onChange={(e) => setEmail(e.target.value)}
                        placeholder="Enter your email"
                        className="px-4 py-2 border rounded w-full"
                        required
                    />
                    <input
                        type="password"
                        value={password}
                        onChange={(e) => setPassword(e.target.value)}
                        placeholder="Enter your password"
                        className="px-4 py-2 border rounded w-full"
                        required
                    />
                    <button type="submit" className="my-4 px-4 py-2 bg-amber-500 text-white rounded w-full">
                        Log in with Email
                    </button>
                </form>
                {loadingEmail ? (
                    <div className="my-2 flex justify-center items-center space-x-2">
                        <div className="w-4 h-4 border-t-2 border-b-2 border-indigo-600 rounded-full animate-spin"></div>
                        <div className="text-sm font-medium text-gray-700">Checking email...</div>
                    </div>
                ) : (
                    emailMessage && <p className="my-2 text-sm font-medium text-red-500 break-all">{emailMessage}</p>
                )}
                <div className="mt-2 text-sm text-gray-600">
                    <a href="#" onClick={() => navigate('/forgot-password')} className="underline">
                        Forgot your password?
                    </a>
                </div>
            </div>

            <hr className="border-t border-gray-300 w-full my-4" />

            <button onClick={() => navigate('/view')} className="my-4 px-4 py-2 bg-gray-400 text-white rounded">
                Enter as Guest
            </button>
        </div>
    );
};

export default ConnectAddress;