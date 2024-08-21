import { createContext, useState, useContext, ReactNode, useEffect } from 'react';
import { LoginAddress, Result_1, User } from './declarations/backend/backend.did';
import { backend } from './declarations/backend';
import { UserTypes } from './model/types';
import { userTypeToString } from './model/utils';
import { HttpAgent } from '@dfinity/agent';
import { Principal } from '@dfinity/principal';

interface UserContextProps {
    user: User | null;
    userType: UserTypes;
    loginMethod: LoginAddress | null;
    password: string | null;
    logout: () => void;
    icpAgent: HttpAgent | null;
    principal: Principal | null;
    setUser: (user: User | null) => void;
    setLoginMethod: (loginMethod: LoginAddress | null, password?: string) => void;
    setIcpAgent: (agent: HttpAgent | null) => void;
    setPrincipal: (principal: Principal | null) => void;
    authenticateUser: (loginAddress: LoginAddress, password?: string) => Promise<Result_1>;
}

const UserContext = createContext<UserContextProps | undefined>(undefined);

export const UserProvider = ({ children }: { children: ReactNode }) => {
    const [user, setUser] = useState<User | null>(null);
    const [userType, setUserType] = useState<UserTypes>("Visitor");
    const [loginMethod, setLoginMethod] = useState<LoginAddress | null>(null);
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

    const authenticateUser = async (loginAddress: LoginAddress): Promise<Result_1> => {
        try {
            return await backend.authenticate_user(loginAddress, password ? [password] : []);
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
            setLoginMethod: (loginMethod: LoginAddress | null, password?: string) => {
                setLoginMethod(loginMethod);
                setPassword(password || null);
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
