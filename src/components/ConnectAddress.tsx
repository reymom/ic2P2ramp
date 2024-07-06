import React, { useEffect } from 'react';
import { ConnectButton } from '@rainbow-me/rainbowkit';
import { useAccount } from 'wagmi';
import { useUser } from '../UserContext';
import { useNavigate } from 'react-router-dom';

const ConnectAddress: React.FC = () => {
    const { isConnected } = useAccount();
    const { userType } = useUser();
    const navigate = useNavigate();

    useEffect(() => {
        if (userType == "Onramper") {
            navigate("/view");
        } else if (userType == "Offramper") {
            navigate("/create");
        }
    }, [userType]);

    return (
        <div className="flex flex-col items-center justify-center text-center h-full py-16 rounded">
            <div className="mb-6">
                <ConnectButton />
            </div>
            {isConnected && (
                <button onClick={() => navigate('/login')} className="my-4 px-4 py-2 bg-blue-500 text-white rounded">
                    Register
                </button>
            )}
            <button onClick={() => navigate('/view')} className="mt-6 px-4 py-2 bg-gray-400 text-white rounded">
                Enter as Guest
            </button>
        </div>
    );
};

export default ConnectAddress;