import { createContext, useState, useContext, ReactNode, useEffect } from 'react';
import { ethers } from 'ethers';
import { useAccount } from 'wagmi';
import { ActorSubclass, HttpAgent } from '@dfinity/agent';
import { IcrcLedgerCanister, BalanceParams, IcrcTokens } from '@dfinity/ledger-icrc';
import { Principal } from '@dfinity/principal';
import { AuthClient } from '@dfinity/auth-client';

import { backend, createActor } from '../../declarations/backend';
import { AuthenticationData, LoginAddress, Result_1, User, _SERVICE } from '../../declarations/backend/backend.did';
import { getEvmTokens } from '../../constants/evm_tokens';
import { ICP_TOKENS } from '../../constants/icp_tokens';
import {
    saveUserSession,
    getUserSession,
    clearUserSession,
    isSessionExpired,
    getSessionToken,
    getUserType,
    getPreferredCurrency,
    savePreferredCurrency
} from '../../model/session';
import { UserTypes } from '../../model/types';
import { icpHost, iiUrl } from '../../model/icp';
import { formatCryptoUnits } from '../../model/helper';

export interface Balance {
    raw: bigint;
    formatted: string;
}

interface UserContextProps {
    refetchUser: () => void;
    setUser: (user: User | null) => void;
    setLoginMethod: (login: LoginAddress | null, pwd?: string) => void;
    setCurrency: (currency: string) => void;
    user: User | null;
    userType: UserTypes;
    loginMethod: LoginAddress | null;
    currency: string;
    sessionToken: string | null;
    password: string | null;
    loginInternetIdentity: () => Promise<[Principal, HttpAgent]>;
    authenticateUser: (
        login: LoginAddress | null,
        authData?: AuthenticationData,
        backendActor?: ActorSubclass<_SERVICE>
    ) => Promise<Result_1>;
    logout: () => Promise<void>;


    icpAgent: HttpAgent | null;
    backendActor: ActorSubclass<_SERVICE> | null,
    principal: Principal | null;
    icpBalances: { [tokenName: string]: Balance } | null;
    evmBalances: { [tokenAddress: string]: Balance } | null;
    fetchBalances: () => Promise<void>;
}

const UserContext = createContext<UserContextProps | undefined>(undefined);

export const UserProvider = ({ children }: { children: ReactNode }) => {
    const [user, setUser] = useState<User | null>(getUserSession());
    const [sessionToken, setSessionToken] = useState<string | null>(getSessionToken(user));
    const [loginMethod, setLoginMethod] = useState<LoginAddress | null>(null);
    const [password, setPassword] = useState<string | null>(null);
    const [icpAgent, setIcpAgent] = useState<HttpAgent | null>(null);
    const [backendActor, setBackendActor] = useState<ActorSubclass<_SERVICE> | null>(null);
    const [principal, setPrincipal] = useState<Principal | null>(null);
    const [currency, setCurrency] = useState<string>(getPreferredCurrency() ?? 'USD');

    const { address, chainId, isConnected } = useAccount();
    const [icpBalances, setIcpBalances] = useState<{ [tokenName: string]: Balance } | null>(null);
    const [evmBalances, setEvmBalances] = useState<{ [tokenAddress: string]: Balance } | null>(null);

    const userType = getUserType(user);

    // I want to refetch if user is loaded from localStorage
    const [hasRefetched, setHasRefetched] = useState(false);
    useEffect(() => {
        if (!hasRefetched) {
            refetchUser();
            setHasRefetched(true);
        }
    }, [sessionToken, hasRefetched]);

    useEffect(() => {
        if (!user || (user && isSessionExpired(user))) {
            logout();
        } else {
            setSessionToken(getSessionToken(user));
        }
    }, [user]);

    useEffect(() => {
        if (principal && icpAgent) {
            fetchIcpBalances();
        }
    }, [principal, icpAgent]);

    useEffect(() => {
        if (chainId && address && isConnected) {
            fetchEvmBalances();
        }
    }, [chainId, address, isConnected])

    const checkInternetIdentity = async () => {
        try {
            const authClient = await AuthClient.create();
            if (await authClient.isAuthenticated()) {
                const identity = authClient.getIdentity();
                const principal = identity.getPrincipal();
                setPrincipal(principal);
                console.log("[checkII] ICP Principal = ", principal.toString());

                const agent = new HttpAgent({ identity, host: icpHost });
                if (process.env.FRONTEND_ICP_ENV === 'test') {
                    agent.fetchRootKey();
                }
                setIcpAgent(agent);
            }
        } catch (error) {
            console.error("Error checking Internet Identity authentication:", error);
        }
    };

    useEffect(() => {
        checkInternetIdentity();
    }, []);

    const loginInternetIdentity = async (): Promise<[Principal, HttpAgent]> => {
        try {
            const authClient = await AuthClient.create();
            return new Promise((resolve, reject) => {
                authClient.login({
                    identityProvider: iiUrl,
                    onSuccess: async () => {
                        try {
                            const identity = authClient.getIdentity();
                            const principal = identity.getPrincipal();
                            setPrincipal(principal);
                            console.log("[loginII] ICP Principal = ", principal.toString());

                            const agent = new HttpAgent({ identity, host: icpHost });
                            if (process.env.FRONTEND_ICP_ENV === 'test') {
                                agent.fetchRootKey();
                            }
                            setIcpAgent(agent);

                            if (!process.env.CANISTER_ID_BACKEND) throw new Error("Backend Canister ID not in env file");
                            const actor = createActor(process.env.CANISTER_ID_BACKEND, { agent });
                            setBackendActor(actor)

                            resolve([principal, agent]);
                        } catch (error) {
                            console.error("Error during Internet Identity login success handling:", error);
                            reject(error);
                        }
                    },
                    onError: (error) => {
                        console.error("Internet Identity login failed:", error);
                        reject(error);
                    },
                });
            });
        } catch (error) {
            console.error("Error creating AuthClient:", error);
            throw error;
        }
    }

    const authenticateUser = async (
        login: LoginAddress | null,
        authData?: AuthenticationData,
        actor?: ActorSubclass<_SERVICE>
    ): Promise<Result_1> => {
        if (!login) throw new Error("Login method is not defined");
        if ('Email' in login && (!authData || !authData.password)) throw new Error("Password is required");
        if ('EVM' in login && (!authData || !authData.signature)) throw new Error("EVM Signature is required");

        try {
            let tmpActor = backend;
            if (actor) {
                setBackendActor(actor);
                tmpActor = actor;
            } else if (backendActor) {
                tmpActor = backendActor;
            }
            const result = await tmpActor.authenticate_user(login, authData ? [authData] : []);

            if ('Ok' in result) {
                setHasRefetched(true);
                setUser(result.Ok);
                const session = result.Ok.session.length > 0 ? result.Ok.session[0] : null;
                if (session) {
                    saveUserSession(result.Ok);
                } else {
                    throw new Error("Session Token is not properly set in the backend");
                }
            }
            return result;
        } catch (error) {
            console.error('Failed to fetch user: ', error);
            throw error;
        }
    }

    const refetchUser = async () => {
        if (!user) return;
        if (!sessionToken) return;

        backend.refetch_user(user.id, sessionToken)
            .then((result) => {
                if ('Ok' in result) {
                    const updatedUser = result.Ok;
                    setUser(updatedUser);
                    fetchBalances();

                    saveUserSession(updatedUser);
                    console.log("User refetched and updated.");
                } else {
                    console.error("Error refetching user:", result.Err);
                    logout();
                }
            })
            .catch((error) => {
                console.error("Error while refetching user:", error);
            });
    }

    const logout = async (): Promise<void> => {
        try {
            const authClient = await AuthClient.create();
            if (authClient && await authClient.isAuthenticated()) {
                await authClient.logout({
                    returnTo: process.env.FRONTEND_BASE_URL || window.location.origin,
                });
            }
        } catch (error) {
            console.error("Error logging out from Internet Identity:", error);
        } finally {
            setUser(null);
            setLoginMethod(null);
            clearUserSession();
            setIcpAgent(null);
            setPrincipal(null);
        }
    };

    const fetchBalances = async () => {
        await fetchIcpBalances();
        await fetchEvmBalances();
    };

    const fetchIcpBalances = async () => {
        if (!icpAgent || !principal) return;

        try {
            const balances: { [tokenName: string]: Balance } = {};

            for (const token of ICP_TOKENS) {
                const ledger = IcrcLedgerCanister.create({
                    agent: icpAgent,
                    canisterId: Principal.fromText(token.address),
                })

                const balanceParams: BalanceParams = {
                    owner: principal,
                };
                const balanceResult = await ledger.balance(balanceParams);

                const balanceFloat = Number(balanceResult) / 10 ** token.decimals;
                balances[token.name] = { raw: balanceResult, formatted: formatCryptoUnits(balanceFloat) }
            }

            setIcpBalances(balances);
        } catch (err: any) {
            console.error('Failed to fetch ICP balances: ', err);
            setIcpBalances(null);
        }
    };

    const fetchEvmBalances = async () => {
        if (!window.ethereum || !chainId || !address || !isConnected) return;

        try {
            const provider = new ethers.BrowserProvider(window.ethereum);
            await provider.send('eth_requestAccounts', []);
            const signer = await provider.getSigner();

            const balances: { [tokenAddress: string]: Balance } = {};
            for (const token of getEvmTokens(chainId)) {
                if (token.isNative) {
                    const nativeBalance = await provider.getBalance(address);
                    balances[token.name] = { raw: nativeBalance, formatted: formatCryptoUnits(Number(ethers.formatEther(nativeBalance))) };
                } else {
                    const tokenContract = new ethers.Contract(
                        token.address,
                        ['function balanceOf(address) view returns (uint256)'],
                        signer,
                    );
                    const balance = await tokenContract.balanceOf(signer.address);
                    balances[token.address] = { raw: balance, formatted: formatCryptoUnits(Number(ethers.formatUnits(balance, token.decimals))) };
                }
            }

            setEvmBalances(balances);
        } catch (err) {
            console.error('Failed to fetch EVM token balances: ', err);
            setEvmBalances(null);
        }
    };

    return (
        <UserContext.Provider value={{
            user,
            userType,
            currency,
            loginMethod,
            sessionToken,
            password,
            icpAgent,
            backendActor,
            principal,
            icpBalances,
            evmBalances,
            setUser,
            setLoginMethod: (login: LoginAddress | null, pwd?: string) => {
                setLoginMethod(login);
                setPassword(pwd || null);
            },
            setCurrency: (currency: string) => {
                savePreferredCurrency(currency);
                setCurrency(currency);
            },
            loginInternetIdentity,
            authenticateUser,
            refetchUser,
            fetchBalances,
            logout,
        }}>
            {children}
        </UserContext.Provider>
    );
};

export const useUser = () => {
    const context = useContext(UserContext);
    if (context === undefined) {
        throw new Error("useUser must be used within a UserProvider");
    }
    return context;
};
