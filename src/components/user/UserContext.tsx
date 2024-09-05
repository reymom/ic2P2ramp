import { createContext, useState, useContext, ReactNode, useEffect } from 'react';
import { AuthenticationData, LoginAddress, Result_1, User } from '../../declarations/backend/backend.did';
import { backend } from '../../declarations/backend';
import { UserTypes } from '../../model/types';
import { userTypeToString } from '../../model/utils';
import { HttpAgent } from '@dfinity/agent';
import { Principal } from '@dfinity/principal';
import { tokenCanisters } from '../../constants/addresses';
import { AccountIdentifier, LedgerCanister } from '@dfinity/ledger-icp';

interface UserContextProps {
    user: User | null;
    userType: UserTypes;
    loginMethod: LoginAddress | null;
    password: string | null;
    logout: () => void;
    icpAgent: HttpAgent | null;
    principal: Principal | null;
    icpBalance: string | null;
    fetchIcpBalance: () => void;
    setUser: (user: User | null) => void;
    setLoginMethod: (login: LoginAddress | null, pwd?: string) => void;
    setIcpAgent: (agent: HttpAgent | null) => void;
    setPrincipal: (principal: Principal | null) => void;
    authenticateUser: (login: LoginAddress | null, authData?: AuthenticationData) => Promise<Result_1>;
}

const UserContext = createContext<UserContextProps | undefined>(undefined);

export const UserProvider = ({ children }: { children: ReactNode }) => {
    const [user, setUser] = useState<User | null>(null);
    const [userType, setUserType] = useState<UserTypes>("Visitor");
    const [loginMethod, setLoginMethod] = useState<LoginAddress | null>(null);
    const [password, setPassword] = useState<string | null>(null);
    const [icpAgent, setIcpAgent] = useState<HttpAgent | null>(null);
    const [principal, setPrincipal] = useState<Principal | null>(null);
    const [icpBalance, setIcpBalance] = useState<string | null>(null);

    useEffect(() => {
        if (!user) {
            setUserType("Visitor")
            return;
        }
        setUserType(userTypeToString(user!.user_type));
    }, [user]);

    useEffect(() => {
        fetchIcpBalance();
    }, [principal, icpAgent]);

    const authenticateUser = async (login: LoginAddress | null, authData?: AuthenticationData): Promise<Result_1> => {
        if (!login) throw new Error("Login method is not defined");
        if ('Email' in login && (!authData || !authData.password)) throw new Error("Password is required");
        try {
            return await backend.authenticate_user(login, authData ? [authData] : []);
        } catch (error) {
            console.error('Failed to fetch user: ', error);
            throw error;
        }
    }

    const logout = () => {
        setUser(null);
        setLoginMethod(null);
        setIcpAgent(null);
        setPrincipal(null);
        setUserType("Visitor");
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
            password,
            logout,
            icpAgent,
            principal,
            icpBalance,
            fetchIcpBalance,
            setUser,
            setLoginMethod: (login: LoginAddress | null, pwd?: string) => {
                setLoginMethod(login);
                setPassword(pwd || null);
            },
            setIcpAgent,
            setPrincipal,
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
