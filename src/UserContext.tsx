import { createContext, useState, useContext, ReactNode, useEffect } from 'react';
import { Address, User } from './declarations/backend/backend.did';
import { backend } from './declarations/backend';
import { UserTypes } from './model/types';
import { userTypeToString } from './model/utils';
import { HttpAgent } from '@dfinity/agent';

interface UserContextProps {
    user: User | null;
    userType: UserTypes;
    loginMethod: Address | null;
    icpAgent: HttpAgent | null;
    setUser: (user: User | null) => void;
    setLoginMethod: (loginMethod: Address | null) => void;
    setIcpAgent: (agent: HttpAgent | null) => void;
}

const UserContext = createContext<UserContextProps | undefined>(undefined);

export const UserProvider = ({ children }: { children: ReactNode }) => {
    const [user, setUser] = useState<User | null>(null);
    const [userType, setUserType] = useState<UserTypes>("Visitor");
    const [loginMethod, setLoginMethod] = useState<Address | null>(null);
    const [icpAgent, setIcpAgent] = useState<HttpAgent | null>(null);

    useEffect(() => {
        if (loginMethod) {
            checkUserRegistration(loginMethod);
        } else {
            setUser(null);
            setIcpAgent(null);
        }
    }, [loginMethod]);

    useEffect(() => {
        if (!user) {
            setUserType("Visitor")
            return;
        }
        setUserType(userTypeToString(user!.user_type));
    }, [user])

    const checkUserRegistration = async (loginAddress: Address) => {
        try {
            const result = await backend.get_user(loginAddress);
            if ('Ok' in result) {
                setUser(result.Ok);
            } else {
                setUser(null);
            }
        } catch (error) {
            console.error('Failed to check user registration: ', error);
        }
    };

    return (
        <UserContext.Provider value={{ user, userType, loginMethod, icpAgent, setUser, setLoginMethod, setIcpAgent }}>
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
