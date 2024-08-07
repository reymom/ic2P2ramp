import React, { useEffect, useState } from 'react';
import { useUser } from '../UserContext';
import { Link } from 'react-router-dom';
import { ConnectButton } from '@rainbow-me/rainbowkit';
import { userTypeToString } from '../model/utils';
import logo from '../assets/p2ploan.webp';
import GetBalanceComponent from './icp/Balance';

const Menu: React.FC = () => {
    const [isMenuOpen, setIsMenuOpen] = useState(false);
    const [isMobile, setIsMobile] = useState(window.innerWidth < 1024);
    const [principal, setPrincipal] = useState<string>();


    const { user, icpAgent } = useUser();

    useEffect(() => {
        const getPrincipal = async () => {
            if (icpAgent) {
                let p = (await icpAgent?.getPrincipal()).toString()!
                setPrincipal(p)
            }
        }

        getPrincipal()
    }, [icpAgent])

    useEffect(() => {
        const handleResize = () => {
            setIsMobile(window.innerWidth < 1024);
        };
        window.addEventListener('resize', handleResize);
        return () => window.removeEventListener('resize', handleResize);
    }, []);

    const toggleMenu = () => {
        setIsMenuOpen(!isMenuOpen);
    };

    const closeMenu = () => {
        setIsMenuOpen(false);
    };

    const renderLinks = () => {
        if (!user) {
            return (
                <>
                    <Link to="/" onClick={closeMenu} className="block py-2 px-4 lg:inline-block lg:py-0">Register</Link>
                    <Link to="/view" onClick={closeMenu} className="block py-2 px-4 lg:inline-block lg:py-0">View Orders</Link>
                </>
            );
        }

        switch (userTypeToString(user.user_type)) {
            case "Onramper":
                return (
                    <>
                        <Link to="/profile" onClick={closeMenu} className="block py-2 px-4 lg:inline-block lg:py-0">View Profile</Link>
                        <Link to="/view" onClick={closeMenu} className="block py-2 px-4 lg:inline-block lg:py-0">View Orders</Link>
                    </>
                );
            case "Offramper":
                return (
                    <>
                        <Link to="/profile" onClick={closeMenu} className="block py-2 px-4 lg:inline-block lg:py-0">View Profile</Link>
                        <Link to="/view" onClick={closeMenu} className="block py-2 px-4 lg:inline-block lg:py-0">My Orders</Link>
                        <Link to="/create" onClick={closeMenu} className="block py-2 px-4 lg:inline-block lg:py-0">Create Order</Link>
                    </>
                );
            default:
                return null;
        }
    };

    return (
        <nav className="bg-white p-4 shadow-md flex justify-between items-center">
            {isMobile &&
                <div className="flex items-center justify-between w-full">
                    <button onClick={toggleMenu} className="p-4">
                        ☰
                    </button>
                    <ConnectButton accountStatus='full' chainStatus="icon" showBalance={false} />
                </div>
            }
            {isMobile && isMenuOpen && (
                <div className="fixed inset-0 bg-gray-800 bg-opacity-75 z-50 lg:hidden">
                    <div className="absolute top-0 left-0 w-64 bg-white h-full shadow-md">
                        <div className="p-4 flex items-center justify-between">
                            <h2 className="text-xl font-bold">Menu</h2>
                            <button onClick={toggleMenu} className="p-2">
                                ✖
                            </button>
                        </div>
                        <div className="p-4">
                            <div>
                                <Link to="/" className="flex items-center">
                                    <img src={logo} className="rounded-full h-12 w-12 mr-2" alt="ic2P2ramp logo" />
                                </Link>
                            </div>
                            <div>
                                Canister Balance: <GetBalanceComponent principal={"be2us-64aaa-aaaaa-qaabq-cai"} />
                            </div>
                            <div>
                                ICP Balance: {principal && <GetBalanceComponent principal={principal} />}
                            </div>
                            <div className="mt-4">
                                {renderLinks()}
                            </div>
                        </div>
                    </div>
                </div>
            )}
            {!isMobile &&
                <div className="flex justify-between w-full">
                    <Link to="/" className="flex items-center w-72">
                        <img src={logo} className="rounded-full h-12 w-12 mr-2" alt="ic2P2ramp logo" />
                        <h1 className="text-xl font-bold">ic2P2ramp</h1>
                    </Link>
                    <div className="flex-grow flex justify-center">
                        <div className="flex items-center space-x-4">
                            {renderLinks()}
                        </div>
                    </div>
                    <div className="w-72 justify-end items-end justify-items-end mr-0">
                        <ConnectButton accountStatus='full' chainStatus="icon" showBalance={false} />
                    </div>
                </div>
            }
        </nav>
    );
};

export default Menu;
