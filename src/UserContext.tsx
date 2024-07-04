import { createContext, useState, useContext, ReactNode, useEffect } from 'react';
import { useAccount } from 'wagmi';
import { User } from './declarations/backend/backend.did';
import { backend } from './declarations/backend';
import { UserTypes } from './model/types';
import { userTypeToString } from './model/utils';

interface UserContextProps {
    user: User | null;
    userType: UserTypes;
    setUser: (user: User | null) => void;
}

const UserContext = createContext<UserContextProps | undefined>(undefined);

export const UserProvider = ({ children }: { children: ReactNode }) => {
    const { address, isConnected } = useAccount();
    const [user, setUser] = useState<User | null>(null);
    const [userType, setUserType] = useState<UserTypes>("Visitor");

    useEffect(() => {
        if (!isConnected) {
            setUser(null);
        }

        if (address) {
            checkUserRegistration(address);
        }
    }, [address, isConnected]);

    useEffect(() => {
        if (!user) {
            setUserType("Visitor")
            return;
        }
        setUserType(userTypeToString(user!.user_type));
    }, [user])

    const checkUserRegistration = async (evm_address: string) => {
        try {
            const result = await backend.get_user(evm_address);
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
        <UserContext.Provider value={{ user, userType, setUser }}>
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
