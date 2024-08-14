import React, { useEffect } from 'react';
import { ConnectButton } from '@rainbow-me/rainbowkit';
import { useAccount } from 'wagmi';
import { useUser } from '../UserContext';
import { useNavigate } from 'react-router-dom';
import { AuthClient } from '@dfinity/auth-client';
import { HttpAgent } from '@dfinity/agent';

const host = process.env.FRONTEND_ENV === 'test' ? 'http://127.0.0.1:8080' : 'https://ic0.app';

const iiUrl = process.env.FRONTEND_ENV === 'production'
    ? `https://identity.ic0.app`
    : `http://${process.env.CANISTER_ID_INTERNET_IDENTITY}.localhost:8080`;

const ConnectAddress: React.FC = () => {
    const { isConnected, address } = useAccount();
    const { userType, setLoginMethod, setIcpAgent } = useUser();
    const navigate = useNavigate();

    useEffect(() => {
        if (userType == "Onramper") {
            navigate("/view");
        } else if (userType == "Offramper") {
            navigate("/create");
        }
    }, [userType]);

    const handleEvmLogin = () => {
        if (address) {
            setLoginMethod({ address_type: { EVM: null }, address: address })
        } else {
            console.error("EVM Address is undefined")
        }
    }

    const handleInternetIdentityLogin = async () => {
        const authClient = await AuthClient.create();
        await authClient.login({
            identityProvider: iiUrl,
            onSuccess: async () => {
                const identity = authClient.getIdentity();
                const principal = identity.getPrincipal();
                console.log("Principal connected = ", principal.toString());
                setLoginMethod({ address_type: { ICP: null }, address: principal.toText() });
                const agent = new HttpAgent({ identity, host });
                if (process.env.FRONTEND_ENV === 'test') {
                    agent.fetchRootKey();
                }
                setIcpAgent(agent);
                navigate("/login");
            },
            onError: (error) => {
                console.error("Internet Identity login failed:", error);
            },
        });
    };

    return (
        <div className="flex flex-col items-center justify-center text-center h-full py-16 rounded">
            <div className="mb-6">
                <ConnectButton />
            </div>
            {isConnected && (
                <button onClick={handleEvmLogin} className="my-4 px-4 py-2 bg-blue-500 text-white rounded">
                    Register
                </button>
            )}
            <button onClick={handleInternetIdentityLogin} className="my-4 px-4 py-2 bg-green-500 text-white rounded">
                Log in with Internet Identity
            </button>
            <button onClick={() => navigate('/view')} className="mt-6 px-4 py-2 bg-gray-400 text-white rounded">
                Enter as Guest
            </button>
        </div>
    );
};

export default ConnectAddress;