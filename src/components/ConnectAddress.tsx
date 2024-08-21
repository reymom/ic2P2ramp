import React, { useEffect, useState } from 'react';
import { ConnectButton } from '@rainbow-me/rainbowkit';
import { useAccount } from 'wagmi';
import { useUser } from '../UserContext';
import { LoginAddress, RampError } from '../declarations/backend/backend.did';
import { useNavigate } from 'react-router-dom';
import { AuthClient } from '@dfinity/auth-client';
import { HttpAgent } from '@dfinity/agent';
import { icpHost, iiUrl } from '../model/icp';

const ConnectAddress: React.FC = () => {
    const [email, setEmail] = useState('');
    const [password, setPassword] = useState('');
    const [message, setMessage] = useState('');
    const [isLoading, setIsLoading] = useState(false);
    const [loadingEvm, setLoadingEvm] = useState(false);
    const [loadingIcp, setLoadingIcp] = useState(false);

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
        if (address) {
            setLoadingEvm(true);

            const loginAddress: LoginAddress = {
                EVM: { address }
            };
            setLoginMethod(loginAddress)

            try {
                const result = await authenticateUser(loginAddress);
                if ('Ok' in result) {
                    setUser(result.Ok);

                    if ('Offramper' in result.Ok.user_type) {
                        navigate("/create");
                    } else {
                        navigate("/view");
                    }
                } else {
                    navigate("/login");
                }
            } catch (error) {
                setMessage("Failed to search user");
            } finally {
                setLoadingEvm(false);
            }

            navigate("/login");
        } else {
            console.error("EVM Address is undefined")
        }
    }

    const handleEmailLogin = async () => {
        if (email && password) {
            setIsLoading(true);

            const loginAddress: LoginAddress = {
                Email: { email }
            };
            setLoginMethod(loginAddress);

            try {
                const result = await authenticateUser(loginAddress, password);
                if ('Ok' in result) {
                    setUser(result.Ok)
                    if ('Offramper' in result.Ok.user_type) {
                        navigate("/create");
                    } else {
                        navigate("/view");
                    }
                } else if ('Err' in result && 'InvalidPassword' in result.Err) {
                    setMessage('Invalid password');
                } else {
                    navigate("/login");
                }
            } catch (error) {
                setMessage("Failed to login user");
            } finally {
                setIsLoading(false);
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
                if (process.env.FRONTEND_ENV === 'test') {
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
                        navigate("/login");
                    }
                } catch (error) {
                    setMessage("Failed to login user");
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
                {loadingEvm && (
                    <div className="my-2 flex justify-center items-center space-x-2">
                        <div className="w-4 h-4 border-t-2 border-b-2 border-indigo-600 rounded-full animate-spin"></div>
                        <div className="text-sm font-medium text-gray-700">Checking user address...</div>
                    </div>
                )}
            </div>

            <hr className="border-t border-gray-300 w-full my-4" />

            <div className="my-4 flex flex-col space-y-2 w-full max-w-xs">
                <input
                    type="email"
                    value={email}
                    onChange={(e) => setEmail(e.target.value)}
                    placeholder="Enter your email"
                    className="px-4 py-2 border rounded w-full"
                />
                <input
                    type="password"
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    placeholder="Enter your password"
                    className="px-4 py-2 border rounded w-full"
                />
                <button onClick={handleEmailLogin} className="my-4 px-4 py-2 bg-amber-500 text-white rounded w-full">
                    Log in with Email
                </button>
                {isLoading ? (
                    <div className="my-2 flex justify-center items-center space-x-2">
                        <div className="w-4 h-4 border-t-2 border-b-2 border-indigo-600 rounded-full animate-spin"></div>
                        <div className="text-sm font-medium text-gray-700">Checking email...</div>
                    </div>
                ) : (
                    message && <p className="my-2 text-sm font-medium text-red-500 break-all">{message}</p>
                )}
            </div>

            <hr className="border-t border-gray-300 w-full my-4" />

            <button onClick={() => navigate('/view')} className="my-4 px-4 py-2 bg-gray-400 text-white rounded">
                Enter as Guest
            </button>
        </div>
    );
};

export default ConnectAddress;