import { createContext, useState, useContext, ReactNode, useEffect } from 'react';
import { ActorSubclass, HttpAgent } from '@dfinity/agent';
import { AccountIdentifier, LedgerCanister } from '@dfinity/ledger-icp';
import { Principal } from '@dfinity/principal';
import { AuthClient } from '@dfinity/auth-client';

import { backend } from '../../declarations/backend';
import { AuthenticationData, LoginAddress, Result_1, User, _SERVICE } from '../../declarations/backend/backend.did';
import { tokenCanisters } from '../../constants/addresses';
import { UserTypes } from '../../model/types';
import { saveUserSession, getUserSession, clearUserSession, isSessionExpired, getSessionToken, getUserType } from '../../model/session';
import { icpHost, iiUrl } from '../../model/icp';

interface UserContextProps {
    user: User | null;
    userType: UserTypes;
    loginMethod: LoginAddress | null;
    sessionToken: string | null;
    password: string | null;
    logout: () => Promise<void>;
    icpAgent: HttpAgent | null;
    backendActor: ActorSubclass<_SERVICE>,
    principal: Principal | null;
    icpBalance: string | null;
    fetchIcpBalance: () => void;
    setUser: (user: User | null) => void;
    setLoginMethod: (login: LoginAddress | null, pwd?: string) => void;
    setBackendActor: (actor: ActorSubclass<_SERVICE>) => void;
    loginInternetIdentity: () => Promise<[Principal, HttpAgent]>;
    authenticateUser: (
        login: LoginAddress | null,
        authData?: AuthenticationData,
    ) => Promise<Result_1>;
}

const UserContext = createContext<UserContextProps | undefined>(undefined);

export const UserProvider = ({ children }: { children: ReactNode }) => {
    const userSession = getUserSession();

    const [user, setUser] = useState<User | null>(userSession);
    const [loginMethod, setLoginMethod] = useState<LoginAddress | null>(null);
    const [password, setPassword] = useState<string | null>(null);
    const [icpAgent, setIcpAgent] = useState<HttpAgent | null>(null);
    const [backendActor, setBackendActor] = useState<ActorSubclass<_SERVICE>>(backend);
    const [principal, setPrincipal] = useState<Principal | null>(null);
    const [icpBalance, setIcpBalance] = useState<string | null>(null);

    const sessionToken = getSessionToken(user);
    const userType = getUserType(user);

    useEffect(() => {
        if (!user || (user && isSessionExpired(user))) {
            logout();
        }
    }, [user]);

    useEffect(() => {
        fetchIcpBalance();
    }, [principal, icpAgent]);

    const checkInternetIdentity = async () => {
        const authClient = await AuthClient.create();
        if (await authClient.isAuthenticated()) {
            const identity = authClient.getIdentity();
            const principal = identity.getPrincipal();
            setPrincipal(principal);
            console.log("ICP Principal = ", principal);

            const agent = new HttpAgent({ identity, host: icpHost });
            if (process.env.FRONTEND_ICP_ENV === 'test') {
                agent.fetchRootKey();
            }
            setIcpAgent(agent);
        }
    };

    useEffect(() => {
        checkInternetIdentity();
    }, []);

    const loginInternetIdentity = async (): Promise<[Principal, HttpAgent]> => {
        const authClient = await AuthClient.create();
        return new Promise((resolve, reject) => {
            authClient.login({
                identityProvider: iiUrl,
                onSuccess: async () => {
                    const identity = authClient.getIdentity();
                    const principal = identity.getPrincipal();
                    setPrincipal(principal);
                    console.log("ICP Principal = ", principal);

                    const agent = new HttpAgent({ identity, host: icpHost });
                    if (process.env.FRONTEND_ICP_ENV === 'test') {
                        agent.fetchRootKey();
                    }
                    setIcpAgent(agent);
                    resolve([principal, agent]);
                },
                onError: (error) => {
                    console.error("Internet Identity login failed:", error);
                    reject(error);
                },
            });
        });
    }

    const authenticateUser = async (login: LoginAddress | null, authData?: AuthenticationData): Promise<Result_1> => {
        if (!login) throw new Error("Login method is not defined");
        if ('Email' in login && (!authData || !authData.password)) throw new Error("Password is required");
        if ('EVM' in login && (!authData || !authData.signature)) throw new Error("EVM Signature is required");

        try {
            let result = await backendActor.authenticate_user(login, authData ? [authData] : []);

            if ('Ok' in result) {
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

    const fetchIcpBalance = async () => {
        if (!icpAgent || !principal) return;

        try {
            const ledgerTokenCanister = Principal.fromText(tokenCanisters.ICP);
            const ledger = LedgerCanister.create({ agent: icpAgent, canisterId: ledgerTokenCanister });

            const accountIdentifier = AccountIdentifier.fromPrincipal({ principal });
            const balanceResult = await ledger.accountBalance({
                accountIdentifier: accountIdentifier,
                certified: true
            });

            const balanceFloat = Number(balanceResult) / 100_000_000;
            const balanceString = balanceFloat.toFixed(2);

            setIcpBalance(balanceString);
        } catch (err: any) {
            console.error('Failed to fetch ICP balance:', err);
            setIcpBalance(null);
        }
    };

    return (
        <UserContext.Provider value={{
            user,
            userType,
            loginMethod,
            sessionToken,
            password,
            logout,
            icpAgent,
            backendActor,
            principal,
            icpBalance,
            fetchIcpBalance,
            setUser,
            setLoginMethod: (login: LoginAddress | null, pwd?: string) => {
                setLoginMethod(login);
                setPassword(pwd || null);
            },
            setBackendActor,
            loginInternetIdentity,
            authenticateUser
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
