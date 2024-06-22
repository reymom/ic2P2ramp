import React from 'react';
import { ConnectButton } from '@rainbow-me/rainbowkit';
import { useAccount } from 'wagmi';
import { pageTypes } from '../model/types';

interface ConnectAddressProps {
    setCurrentTab: (tab: pageTypes) => void;
}

const ConnectAddress: React.FC<ConnectAddressProps> = ({ setCurrentTab }) => {
    const { isConnected } = useAccount();

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