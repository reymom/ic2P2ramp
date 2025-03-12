import { createContext, useState, useContext, ReactNode, useEffect } from 'react';
import { ethers } from 'ethers';
import { useAccount } from 'wagmi';
import { disconnect } from '@wagmi/core';
import { ActorSubclass, HttpAgent } from '@dfinity/agent';
import { IcrcLedgerCanister, BalanceParams } from '@dfinity/ledger-icrc';
import { Principal } from '@dfinity/principal';
import { AuthClient } from '@dfinity/auth-client';

import { config, getChains } from '@/wagmi';
import { AuthenticationData, LoginAddress, Result_1, User, _SERVICE } from '@/declarations/backend/backend.did';
import { backend, createActor } from '@/model/backendProxy';
import { getEvmTokens } from '@/constants/evm_tokens';
import { ICP_TOKENS } from '@/constants/icp_tokens';
import { supportedRuneIds } from '@/constants/runes';
import {
    saveUserSession,
    getUserSession,
    clearUserSession,
    isSessionExpired,
    getSessionToken,
    getUserType,
    getPreferredCurrency,
    savePreferredCurrency
} from '@/model/session';
import { UserTypes } from '@/model/types';
import { icpHost, iiUrl } from '@/model/blockchain/icp';
import { fetchRuneBalances, isCorrectUnisatChain, switchUnisatChain } from '@/model/blockchain/unisat';
import { formatCryptoUnits } from '@/utils/helper';
import bitcoinLogo from '@/assets/blockchains/bitcoin-logo.svg';

export interface Balance {
    raw: bigint;
    formatted: string;
    logo: string;
}

export interface BitcoinBalance {
    raw: bigint;
    formatted: string;
    logo: string;
    runes: { [runeId: string]: { raw: bigint; formatted: string; symbol: string, logo: string, name: string } };
}

interface UserContextProps {
    refetchUser: () => Promise<void>;
    setUser: (user: User | null) => void;
    setLoginMethod: (login: LoginAddress | null, pwd?: string) => void;
    setCurrency: (currency: string) => void;
    user: User | null;
    userType: UserTypes;
    loginMethod: LoginAddress | null;
    currency: string;
    sessionToken: string | null;
    password: string | null;
    bitcoinAddress: string | null;
    connectUnisat: () => Promise<string | null>;
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
    bitcoinBalance: BitcoinBalance | null;
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

    const [bitcoinAddress, setBitcoinAddress] = useState<string | null>(null);
    const [unisatInstalled, setUnisatInstalled] = useState(false);

    const { address, chainId, isConnected } = useAccount();
    const [icpBalances, setIcpBalances] = useState<{ [tokenName: string]: Balance } | null>(null);
    const [evmBalances, setEvmBalances] = useState<{ [tokenAddress: string]: Balance } | null>(null);
    const [bitcoinBalance, setBitcoinBalance] = useState<BitcoinBalance | null>(null);

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

    useEffect(() => {
        if (unisatInstalled && bitcoinAddress !== null) {
            fetchBitcoinBalance();
        }
    }, [unisatInstalled, bitcoinAddress])

    const connectUnisat = async (): Promise<string | null> => {
        if (unisatInstalled) {
            try {
                if ((window as any).unisat) {
                    console.log("[connectUnisat] onCorrectChain?");
                    const onCorrectChain = await isCorrectUnisatChain();

                    if (!onCorrectChain) {
                        const switched = await switchUnisatChain();
                        if (!switched) {
                            console.error("Failed to switch network");
                            return null;
                        }
                    }

                    const accounts = await (window as any).unisat.requestAccounts();
                    if (accounts && accounts.length > 0) {
                        const account = accounts[0];
                        setBitcoinAddress(account);
                        return account;
                    }
                } else {
                    alert("Unisat wallet is not installed. Please install it to connect.");
                }
            } catch (error) {
                console.error("Failed to connect to Unisat", error);
            }
        }
        return null;
    };

    const checkInternetIdentity = async () => {
        try {
            const authClient = await AuthClient.create();
            if (await authClient.isAuthenticated()) {
                const identity = authClient.getIdentity();
                const principal = identity.getPrincipal();
                setPrincipal(principal);
                console.log("[checkII] ICP Principal = ", principal.toString());

                const agent = await HttpAgent.create({ identity, host: icpHost });
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

        if (typeof window !== "undefined" && (window as any).unisat) {
            setUnisatInstalled(true);
        }
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

                            const agent = await HttpAgent.create({ identity, host: icpHost });
                            if (process.env.FRONTEND_ICP_ENV === 'test') {
                                agent.fetchRootKey();
                            }
                            setIcpAgent(agent);

                            if (!process.env.CANISTER_ID_BACKEND_FUSION) throw new Error("Backend Canister ID not in env file");
                            const actor = createActor(process.env.CANISTER_ID_BACKEND_FUSION, { agent });
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
        if ('Bitcoin' in login && (!authData || !authData.signature || !authData.pubkey)) throw new Error("Bitcoin Signature and Public Key are required");

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

    const refetchUser = async (): Promise<void> => {
        if (!user) return;
        if (!sessionToken) return;

        backend.refetch_user(user.id, sessionToken)
            .then(async (result) => {
                if ('Ok' in result) {
                    const updatedUser = result.Ok;
                    setUser(updatedUser);
                    await fetchBalances();

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

            if (unisatInstalled) {
                await (window as any).unisat.disconnect();
            }

            if (address && chainId) {
                await disconnect(config);
            }
        } catch (error) {
            console.error("Error logging out from Internet Identity:", error);
        } finally {
            setUser(null);
            setLoginMethod(null);
            setSessionToken(null);
            clearUserSession();
            setIcpAgent(null);
            setPrincipal(null);
            setBitcoinAddress(null);
            setIcpBalances(null);
            setEvmBalances(null);
            setBitcoinBalance(null);
        }
    };

    const fetchBalances = async () => {
        await fetchIcpBalances();
        await fetchEvmBalances();
        await fetchBitcoinBalance();
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
                balances[token.name] = { raw: balanceResult, formatted: formatCryptoUnits(balanceFloat), logo: token.logo }
            }

            setIcpBalances(balances);
        } catch (err: any) {
            console.error('Failed to fetch ICP balances: ', err);
            setIcpBalances(null);
        }
    };

    const fetchBitcoinBalance = async () => {
        if (!unisatInstalled || !bitcoinAddress) return;

        try {
            let res = await (window as any).unisat.getBalance();
            const bitcoinBalances: BitcoinBalance = {
                raw: BigInt(res.total),
                formatted: formatCryptoUnits(res.total / 10 ** 8),
                logo: bitcoinLogo,
                runes: {}
            };

            console.log("[fetchBitcoinBalance] unisatGetBalance res = ", res);

            bitcoinBalances.runes = await fetchRuneBalances(bitcoinAddress, supportedRuneIds);

            setBitcoinBalance(bitcoinBalances);
        } catch (e) {
            console.log('Failed to fetch Bitcoin balance: ', e);
            setBitcoinBalance(null);
        }
    }

    const fetchEvmBalances = async () => {
        if (!window.ethereum || !chainId || !address || !isConnected) return;

        if (!getChains().some((chain) => chain.id === chainId)) return;

        try {
            const provider = new ethers.BrowserProvider(window.ethereum);
            await provider.send('eth_requestAccounts', []);
            const signer = await provider.getSigner();

            const balances: { [tokenAddress: string]: Balance } = {};
            for (const token of getEvmTokens(chainId)) {
                if (token.isNative) {
                    const nativeBalance = await provider.getBalance(address);
                    balances[token.name] = { raw: nativeBalance, formatted: formatCryptoUnits(Number(ethers.formatEther(nativeBalance))), logo: token.logo };
                } else {
                    const tokenContract = new ethers.Contract(
                        token.address,
                        ['function balanceOf(address) view returns (uint256)'],
                        signer,
                    );
                    const balance = await tokenContract.balanceOf(signer.address);
                    balances[token.address] = { raw: balance, formatted: formatCryptoUnits(Number(ethers.formatUnits(balance, token.decimals))), logo: token.logo };
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
            bitcoinBalance,
            bitcoinAddress,
            connectUnisat,
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
