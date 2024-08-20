import { createContext, useState, useContext, ReactNode, useEffect } from 'react';
import { Address, RampError, Result_4, User } from './declarations/backend/backend.did';
import { backend } from './declarations/backend';
import { UserTypes } from './model/types';
import { userTypeToString } from './model/utils';
import { HttpAgent } from '@dfinity/agent';
import { Principal } from '@dfinity/principal';

interface UserContextProps {
    user: User | null;
    userType: UserTypes;
    loginMethod: Address | null;
    password: string | null;
    logout: () => void;
    icpAgent: HttpAgent | null;
    principal: Principal | null;
    setUser: (user: User | null) => void;
    setLoginMethod: (loginMethod: Address | null, password?: string) => void;
    setIcpAgent: (agent: HttpAgent | null) => void;
    setPrincipal: (principal: Principal | null) => void;
    getUser: (loginAddress: Address, password?: string) => Promise<Result_4>;
}

const UserContext = createContext<UserContextProps | undefined>(undefined);

export const UserProvider = ({ children }: { children: ReactNode }) => {
    const [user, setUser] = useState<User | null>(null);
    const [userType, setUserType] = useState<UserTypes>("Visitor");
    const [loginMethod, setLoginMethod] = useState<Address | null>(null);
    const [password, setPassword] = useState<string | null>(null);
    const [icpAgent, setIcpAgent] = useState<HttpAgent | null>(null);
    const [principal, setPrincipal] = useState<Principal | null>(null);

    useEffect(() => {
        if (!user) {
            setUserType("Visitor")
            return;
        }
        setUserType(userTypeToString(user!.user_type));
    }, [user])

    const getUser = async (loginAddress: Address, password?: string): Promise<Result_4> => {
        try {
            return await backend.get_user(loginAddress, password ? [password] : []);
        } catch (error) {
            console.error('Failed to fetch user: ', error);
            throw error;
        }
    }

    const logout = () => {
        setUser(null);
        setLoginMethod(null);
        setPassword(null);
        setIcpAgent(null);
        setPrincipal(null);
        setUserType("Visitor");
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
            setUser,
            setLoginMethod: (loginMethod: Address | null, password?: string) => {
                setLoginMethod(loginMethod);
                setPassword(password || null);
            },
            setIcpAgent,
            setPrincipal,
            getUser
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
