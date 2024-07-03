import React, { useEffect } from 'react';
import { ConnectButton } from '@rainbow-me/rainbowkit';
import { useAccount } from 'wagmi';
import { pageTypes } from '../model/types';
import { backend } from '../declarations/backend';
import { UserTypes } from '../model/types';

interface ConnectAddressProps {
    setCurrentTab: (tab: pageTypes) => void;
    setUserType: (type: UserTypes) => void;
}

const ConnectAddress: React.FC<ConnectAddressProps> = ({ setCurrentTab, setUserType }) => {
    const { isConnected, address } = useAccount();

    useEffect(() => {
        if (isConnected) {
            checkUserRegistration();
        }
    }, [isConnected, address]);

    const checkUserRegistration = async () => {
        try {
            const result = await backend.get_user(address as string);
            if ('Ok' in result) {
                const user = result.Ok;
                if ('Onramper' in user.user_type) {
                    setUserType('Onramper');
                    setCurrentTab(pageTypes.view);
                } else if ('Offramper' in user.user_type) {
                    setUserType('Offramper');
                    setCurrentTab(pageTypes.create);
                }
            } else {
                setCurrentTab(pageTypes.login);
            }
        } catch (error) {
            console.error('Failed to check user registration: ', error);
        }
    };

    return (
        <div className="flex flex-col items-center justify-center text-center h-full py-16 rounded">
            <div className="mb-6">
                <ConnectButton />
            </div>
            {isConnected && (
                <button onClick={() => setCurrentTab(pageTypes.login)} className="my-4 px-4 py-2 bg-blue-500 text-white rounded">
                    Add Payment Preferences
                </button>
            )}
            <button onClick={() => setCurrentTab(pageTypes.view)} className="mt-6 px-4 py-2 bg-gray-400 text-white rounded">
                Enter as Guest
            </button>
        </div>
    );
};

export default ConnectAddress;