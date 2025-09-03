import { createContext, useState, useContext, ReactNode, useEffect, useRef } from 'react';
import { ethers } from 'ethers';
import { useAccount } from 'wagmi';
import { disconnect } from '@wagmi/core';
import { ActorSubclass, HttpAgent } from '@dfinity/agent';
import { IcrcLedgerCanister, BalanceParams } from '@dfinity/ledger-icrc';
import { Principal } from '@dfinity/principal';
import { AuthClient } from '@dfinity/auth-client';
import { Connection, PublicKey } from '@solana/web3.js';
import { useWallet } from '@solana/wallet-adapter-react';
import { useWalletModal } from '@solana/wallet-adapter-react-ui';
import type { Adapter, MessageSignerWalletAdapter } from '@solana/wallet-adapter-base';

import { config, getChains } from '@/wagmi';
import { AuthenticationData, LoginAddress, Result_1, User, _SERVICE } from '@/declarations/icramp_backend/icramp_backend.did';
import { getBackendCanisterId } from '@/constants/canisters';
import { getEvmTokens } from '@/constants/evm_tokens';
import { ICP_TOKENS } from '@/constants/icp_tokens';
import { supportedRuneIds } from '@/constants/runes';
import { SPL_TOKEN_LOGOS } from '@/constants/solana_logos';
import { backend, createActor } from '@/model/backendProxy';
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
import { getRegisteredSolanaTokens, SOLANA_RPC_URL } from '@/model/blockchain/solana';
import { fetchRuneBalances, isCorrectUnisatChain, switchUnisatChain } from '@/model/blockchain/unisat';
import { formatCryptoUnits } from '@/utils/formatters';

import bitcoinLogo from '@/assets/blockchains/bitcoin-logo.svg';
import solanaLogo from '@/assets/blockchains/solana-logo.png';
import splGenericIcon from '@/assets/spl_tokens/spl-generic-icon.png';
import { TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID } from '@solana/spl-token';

export interface Balance {
    raw: bigint | number;
    formatted: string;
    logo: string;
}

export interface BitcoinBalance {
    balance: Balance;
    runes: { [runeId: string]: Balance & { symbol: string, name: string } };
}

export interface SolanaBalance {
    balance: Balance;
    splTokens: {
        [mint: string]: Balance & { symbol?: string; decimals?: number; }
    }
};

interface UserContextProps {
    user: User | null;
    userType: UserTypes;
    refetchUser: () => Promise<void>;
    setUser: (user: User | null) => void;
    loginMethod: LoginAddress | null;
    setLoginMethod: (login: LoginAddress | null, pwd?: string) => void;

    currency: string;
    setCurrency: (currency: string) => void;

    sessionToken: string | null;
    password: string | null;
    authenticateUser: (
        login: LoginAddress | null,
        authData?: AuthenticationData,
        backendActor?: ActorSubclass<_SERVICE>
    ) => Promise<Result_1>;
    logout: () => Promise<void>;

    icpAgent: HttpAgent | null;
    backendActor: ActorSubclass<_SERVICE> | null,
    principal: Principal | null;
    loginInternetIdentity: () => Promise<[Principal, HttpAgent]>;

    icpBalances: { [tokenName: string]: Balance } | null;
    evmBalances: { [tokenAddress: string]: Balance } | null;
    solanaBalance: SolanaBalance | null;
    bitcoinBalance: BitcoinBalance | null;
    fetchBalances: () => Promise<void>;

    bitcoinAddress: string | null;
    connectUnisat: () => Promise<string | null>;

    solanaPubkey: string | null;
    connectSolana: () => Promise<string>;
    getSolanaMessageSigner: () => Promise<(msg: Uint8Array) => Promise<Uint8Array>>;
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
    const [bitcoinAddress, setBitcoinAddress] = useState<string | null>(null);
    const [unisatInstalled, setUnisatInstalled] = useState(false);
    const [bitcoinBalance, setBitcoinBalance] = useState<BitcoinBalance | null>(null);
    const [solanaPubkey, setSolanaPubkey] = useState<string | null>(null);
    const [solanaBalance, setSolanaBalance] = useState<SolanaBalance | null>(null);

    const { wallet, disconnect: disconnectSol } = useWallet();
    const { setVisible } = useWalletModal();
    const walletRef = useRef(wallet);
    useEffect(() => { walletRef.current = wallet; }, [wallet]);

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

    useEffect(() => {
        const pk = walletRef.current?.adapter.publicKey?.toBase58?.();
        if (pk && pk !== solanaPubkey) setSolanaPubkey(pk);
    }, [wallet]);

    useEffect(() => {
        if (solanaPubkey) { fetchSolanaBalances() }
    }, [solanaPubkey]);

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

    const hasSignMessage = (a: Adapter): a is MessageSignerWalletAdapter =>
        typeof (a as any)?.signMessage === 'function';

    const getSolanaMessageSigner = async () => {
        // ensure a wallet is selected & connected
        const pk = solanaPubkey ?? (await connectSolana());
        if (!pk || !walletRef.current) throw new Error('No Solana wallet selected');
        const adapter = walletRef.current.adapter;
        if (!adapter.connected) await adapter.connect();

        // wait until wallet exposes signMessage (e.g., after unlock)
        const start = Date.now();
        while (!hasSignMessage(adapter)) {
            await new Promise(r => setTimeout(r, 50));
            if (Date.now() - start > 60000) throw new Error('Selected wallet cannot sign messages');
        }
        return adapter.signMessage!.bind(adapter);
    };

    const waitForWalletPick = () =>
        new Promise<void>((resolve, reject) => {
            const start = Date.now();
            const id = setInterval(() => {
                if (walletRef.current) { clearInterval(id); resolve(); }
                if (Date.now() - start > 60000) { clearInterval(id); reject(new Error('Wallet selection cancelled')); }
            }, 100);
        });

    const connectSolana = async (): Promise<string> => {
        // if no wallet selected, open modal and wait for user choice
        if (!walletRef.current) {
            setVisible(true);
            await waitForWalletPick();
        }
        const adapter = walletRef.current!.adapter;

        try { await adapter.connect(); } catch (e) {
            throw e;
        }

        const start = Date.now();
        let pk = adapter.publicKey?.toBase58?.();
        while (!pk) {
            await new Promise(r => setTimeout(r, 50));
            pk = adapter.publicKey?.toBase58?.();
            if (Date.now() - start > 60000) throw new Error('Wallet connect timed out');
        }

        setSolanaPubkey(pk);
        return pk;
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

                            const canisterId = getBackendCanisterId();
                            const actor = createActor(canisterId, { agent });
                            setBackendActor(actor)

                            console.log("[loginII] Backend Actor = ", actor);
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
        if ('Solana' in login && (!authData || !authData.signature || !authData.pubkey))
            throw new Error("Solana Signature and Public Key are required");
        if ('Bitcoin' in login && (!authData || !authData.signature || !authData.pubkey)) throw new Error("Bitcoin Signature and Public Key are required");

        console.log("[authenticateUser] authData = ", authData);
        try {
            let tmpActor = backend;
            if (actor) {
                setBackendActor(actor);
                tmpActor = actor;
            } else if (backendActor) {
                tmpActor = backendActor;
            }
            const result = await tmpActor.authenticate_user(login, authData ? [authData] : []);
            console.log("[authenticateUser] result = ", result);

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

            try { await disconnectSol(); } catch { }
        } catch (error) {
            console.error("Error logging out", error);
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
            setSolanaBalance(null);
            setSolanaPubkey(null);
        }
    };

    const fetchBalances = async () => {
        await fetchIcpBalances();
        await fetchEvmBalances();
        await fetchBitcoinBalance();
        await fetchSolanaBalances();
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
                balance: {
                    raw: BigInt(res.total),
                    formatted: formatCryptoUnits(res.total / 10 ** 8),
                    logo: bitcoinLogo,
                },
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

    const fetchSolanaBalances = async () => {
        if (!solanaPubkey) return;

        try {
            const conn = new Connection(SOLANA_RPC_URL, 'confirmed');
            const owner = new PublicKey(solanaPubkey);

            const [registry, lamports, tokAccsV1, tokAccs22] = await Promise.all([
                getRegisteredSolanaTokens(),
                conn.getBalance(owner, 'confirmed'),
                conn.getParsedTokenAccountsByOwner(owner, { programId: TOKEN_PROGRAM_ID }, 'confirmed'),
                conn.getParsedTokenAccountsByOwner(owner, { programId: TOKEN_2022_PROGRAM_ID }, 'confirmed'),
            ]);

            const SOL_DECIMALS = 9;
            const solBalance: Balance = {
                raw: lamports,
                formatted: (lamports / 10 ** SOL_DECIMALS).toLocaleString(undefined, {
                    maximumFractionDigits: 6,
                }),
                logo: solanaLogo,
            };

            const approvedMints = new Set(Object.keys(registry));
            console.log("approvedMints = ", approvedMints);
            const all = tokAccsV1.value.concat(tokAccs22.value);
            if (all.length === 0) {
                setSolanaBalance({ balance: solBalance, splTokens: {} });
                return;
            }

            const splBalances: Record<string, Balance & { symbol?: string; decimals?: number }> = Object.create(null);

            for (const acc of all) {
                const info = (acc.account.data as any)?.parsed?.info;
                if (!info) continue;

                const mint: string = info.mint;
                if (!approvedMints.has(mint)) continue;

                const amountStr: string = info.tokenAmount?.amount ?? '0';
                const meta = registry[mint];
                const decimals: number =
                    Number.isInteger(meta?.decimals) ? meta!.decimals : (info.tokenAmount?.decimals ?? 0);
                const uiAmount = Number(amountStr) / 10 ** decimals;

                splBalances[mint] = {
                    raw: BigInt(amountStr),
                    formatted: uiAmount.toLocaleString(undefined, {
                        maximumFractionDigits: Math.min(6, decimals),
                    }),
                    logo: SPL_TOKEN_LOGOS[meta?.symbol ?? ''] || splGenericIcon,
                    symbol: meta?.symbol,
                    decimals,
                };
            }

            setSolanaBalance({ balance: solBalance, splTokens: splBalances });
        } catch (e) {
            console.error('Failed to fetch Solana balances:', e);
            setSolanaBalance(null);
        }
    };

    return (
        <UserContext.Provider value={{
            user,
            userType,
            refetchUser,
            setUser,
            loginMethod,
            setLoginMethod: (login: LoginAddress | null, pwd?: string) => {
                setLoginMethod(login);
                setPassword(pwd || null);
            },

            currency,
            setCurrency: (currency: string) => {
                savePreferredCurrency(currency);
                setCurrency(currency);
            },

            sessionToken,
            password,
            authenticateUser,
            logout,

            icpAgent,
            backendActor,
            principal,
            loginInternetIdentity,

            icpBalances,
            evmBalances,
            solanaBalance,
            bitcoinBalance,
            fetchBalances,

            bitcoinAddress,
            connectUnisat,

            solanaPubkey,
            connectSolana,
            getSolanaMessageSigner,
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
