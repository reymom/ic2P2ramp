import React, { useEffect, useRef, useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { ConnectButton } from '@rainbow-me/rainbowkit';
import { useAccount } from 'wagmi';

import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faUserCircle, faSignOutAlt, faFileAlt, faPlusCircle, faRightToBracket, faBars, faTimes } from '@fortawesome/free-solid-svg-icons';
import icpLogo from "../assets/blockchains/icp-logo.svg";
import ethereumLogo from "../assets/blockchains/ethereum-logo.png";
import logo from '../assets/icR-logo.png';

import { useUser } from './user/UserContext';
import { userTypeToString } from '../model/utils';
import { truncate, formatTimeLeft } from '../model/helper';
import { sessionMarginMilisec } from '../model/session';

const Menu: React.FC = () => {
    const [isMenuOpen, setIsMenuOpen] = useState(false);
    const [isMobile, setIsMobile] = useState(window.innerWidth < 1024);
    const [isProfileDropdownOpen, setIsProfileDropdownOpen] = useState(false);
    const [timeLeft, setTimeLeft] = useState<number | null>(null);

    const { isConnected } = useAccount();
    const { user, icpBalances, loginInternetIdentity, logout } = useUser();
    const navigate = useNavigate();

    const profileDropdownRef = useRef<HTMLDivElement>(null);
    const menuRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        setIsProfileDropdownOpen(false);
        const handleResize = () => {
            setIsMobile(window.innerWidth < 1024);
        };
        window.addEventListener('resize', handleResize);
        return () => window.removeEventListener('resize', handleResize);
    }, []);

    const toggleMenu = () => {
        setIsMenuOpen(!isMenuOpen);
    };

    const toggleProfileDropdown = () => {
        setIsProfileDropdownOpen(!isProfileDropdownOpen);
    };

    useEffect(() => {
        function handleClickOutside(event: MouseEvent) {
            if (profileDropdownRef.current && !profileDropdownRef.current.contains(event.target as Node)) {
                setIsProfileDropdownOpen(false);
            }
        }
        document.addEventListener('mousedown', handleClickOutside);
        return () => {
            document.removeEventListener('mousedown', handleClickOutside);
        };
    }, [profileDropdownRef]);

    useEffect(() => {
        function handleClickOutside(event: MouseEvent) {
            if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
                closeMenu();
            }
        }
        document.addEventListener('mousedown', handleClickOutside);
        return () => {
            document.removeEventListener('mousedown', handleClickOutside);
        };
    }, [menuRef]);

    const closeMenu = () => {
        setIsMenuOpen(false);
    };

    const renderLinks = () => {
        const viewLink =
            <Link to="/view" onClick={closeMenu} className="flex items-center space-x-2 text-gray-800 hover:text-gray-900 transition-colors duration-200 bg-gray-200 hover:bg-gray-300 py-2 px-3 border rounded-md">
                <FontAwesomeIcon icon={faFileAlt} className="text-teal-900 w-6" />
                <span>Orders</span>
            </Link>

        if (!user) return viewLink;
        switch (userTypeToString(user.user_type)) {
            case "Onramper": return viewLink;
            case "Offramper":
                return (
                    <>
                        <Link to="/view" onClick={closeMenu} className="flex items-center space-x-2 text-gray-800 hover:text-gray-900 transition-colors duration-200 bg-gray-200 hover:bg-gray-300 py-2 px-3 border rounded-md">
                            <FontAwesomeIcon icon={faFileAlt} className="text-yellow-900 w-6" />
                            <span>My Orders</span>
                        </Link>
                        <Link to="/create" onClick={closeMenu} className="flex items-center space-x-2 text-gray-800 hover:text-gray-900 transition-colors duration-200 bg-gray-200 hover:bg-gray-300 py-2 px-3 border rounded-md">
                            <FontAwesomeIcon icon={faPlusCircle} className="text-blue-900 w-6" />
                            <span>Create Order</span>
                        </Link>
                    </>
                );
            default: return null;
        }
    };

    const handleInternetIdentityLogin = async () => {
        await loginInternetIdentity();
    };

    useEffect(() => {
        if (user && user.session && user.session.length > 0 && user.session[0]) {
            const sessionExpiry = user.session[0].expires_at;
            const calculateTimeLeft = () => {
                const currentTime = BigInt((Date.now() + sessionMarginMilisec) * 1_000_000);
                const timeLeftNano = sessionExpiry - currentTime;
                const timeLeftSeconds = Number(timeLeftNano) / 1_000_000_000;

                setTimeLeft(timeLeftSeconds > 0 ? timeLeftSeconds : null);
            };

            calculateTimeLeft();

            const timer = setInterval(calculateTimeLeft, 1000);
            return () => clearInterval(timer);
        }
    }, [user]);

    return (
        <nav className="py-6 px-10 flex justify-between items-center rounded-lg bg-transparent relative">
            {isMobile &&
                <>
                    <div className="flex items-center justify-between w-full">
                        <button onClick={toggleMenu} className="p-4 text-gray-700">
                            <FontAwesomeIcon icon={faBars} size="2x" />
                        </button>
                    </div>
                    <div className={`fixed inset-0 bg-gray-900 bg-opacity-75 z-50 lg:hidden ${isMenuOpen ? 'block' : 'hidden'}`}>
                        <div className="absolute top-0 left-0 w-64 bg-gray-200 h-full shadow-md" ref={menuRef}>
                            <div className="p-4 flex items-center justify-between">
                                <div>
                                    <Link to="/" className="flex items-center" onClick={closeMenu}>
                                        <img src={logo} className="rounded-full h-20 w-20 mr-2" alt="icRamp logo" />
                                        <h1 className="text-2xl text-sky-700 tracking-wider -mt-2" style={{
                                            WebkitTextStroke: '1px #280d57',
                                            WebkitTextFillColor: '#280d57',
                                            letterSpacing: '0.08em',
                                        }}>
                                            icRamp
                                        </h1>
                                    </Link>
                                </div>
                                <button onClick={toggleMenu} className="p-2 text-gray-600">
                                    <FontAwesomeIcon icon={faTimes} size="lg" />
                                </button>
                            </div>
                            <div className="p-4 space-y-4">
                                {renderLinks()}
                            </div>
                            {icpBalances && icpBalances['ICP'] && (
                                <div className="p-4">
                                    <div className="border border-gray-300 rounded px-4 py-2 text-green-500 text-center font-medium">
                                        {icpBalances['ICP'].formatted} ICP
                                    </div>
                                </div>
                            )}
                        </div>
                    </div>
                </>
            }

            {!isMobile &&
                <div className="flex justify-between w-full">
                    <Link to="/" className="flex items-center w-72 text-center align-middle">
                        <img src={logo} className="rounded-full h-20 w-20 mr-2" alt="icRamp logo" />
                        <h1 className="text-4xl text-sky-700 tracking-wider -mt-2" style={{
                            // color: '#ffffff',
                            WebkitTextStroke: '1px #280d57',
                            WebkitTextFillColor: '#280d57',
                            letterSpacing: '0.08em',
                        }}>
                            icRamp
                        </h1>
                    </Link>
                    <div className="absolute inset-0 flex justify-center items-center text-gray-800">
                        <div className="flex items-center space-x-6">
                            {renderLinks()}
                        </div>
                    </div>
                </div>
            }

            <div className="w-72 flex justify-end items-center relative">
                {!user ? (
                    <div className="relative">
                        <button
                            onClick={() => navigate('/')}
                            className="flex items-center justify-center bg-indigo-800 hover:bg-indigo-900 text-lg font-bold text-white px-3 py-2 rounded-lg transition-all"
                        >
                            <FontAwesomeIcon icon={faRightToBracket} size="lg" className="mr-2" />
                            <span>Login</span>
                        </button>
                    </div>
                ) : (
                    <div className="relative" ref={profileDropdownRef}>
                        {/* Dropdown */}
                        <button onClick={toggleProfileDropdown} className="flex items-center space-x-2 p-2 border border-gray-400 rounded-lg transition-all">
                            <FontAwesomeIcon icon={faUserCircle} size="2x" className="text-violet-800" />
                            <svg className={`w-4 h-4 ml-1 transform ${isProfileDropdownOpen ? 'rotate-180' : ''}`} fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M19 9l-7 7-7-7"></path>
                            </svg>
                        </button>
                        {isProfileDropdownOpen && (
                            <div className="absolute right-0 mt-2 w-72 bg-white shadow-lg rounded-lg z-50">
                                <div className="p-4 text-gray-700 border-b border-gray-200">

                                    <div className="flex items-center text-center">
                                        <span className="flex-grow text-sm font-semibold text-blue-600 truncate">
                                            {(() => {
                                                if ('EVM' in user.login) {
                                                    return truncate(user.login.EVM.address, 12, 12);
                                                } else if ('ICP' in user.login) {
                                                    return truncate(user.login.ICP.principal_id, 12, 12);
                                                } else if ('Solana' in user.login) {
                                                    return truncate(user.login.Solana.address, 12, 12);
                                                } else if ('Email' in user.login) {
                                                    return truncate(user.login.Email.email, 12, 12);
                                                }
                                                return '';
                                            })()}
                                        </span>
                                    </div>

                                    <div className="items-center text-center">
                                        <hr className="border-t border-gray-300 w-full my-2" />
                                        {icpBalances && icpBalances['ICP'] ? (
                                            <div className="relative flex justify-center items-center border border-gray-300 rounded-md px-3 py-2 text-green-800 text-center font-medium">
                                                <img src={icpLogo} alt="ICP Logo" className="h-6 w-6 absolute left-3" />
                                                <span className="text-lg">{icpBalances['ICP'].formatted} ICP</span>
                                            </div>
                                        ) : (
                                            <div
                                                className="relative flex justify-center items-center px-3 py-2 bg-amber-800 rounded-md hover:bg-amber-900 cursor-pointer"
                                                onClick={handleInternetIdentityLogin}
                                            >
                                                <img src={icpLogo} alt="ICP Logo" className="h-6 w-6 absolute left-3" />
                                                <span className="text-white text-lg">Connect ICP</span>
                                            </div>
                                        )}
                                    </div>

                                    <hr className="border-t border-gray-300 w-full my-2" />
                                    {!isConnected ? (
                                        <div className="relative flex justify-center items-center px-3 py-2 bg-amber-800 rounded-md hover:bg-amber-900 cursor-pointer">
                                            <img src={ethereumLogo} alt="Ethereum Logo" className="h-6 w-6 absolute left-3" />
                                            <div className="w-full text-left">
                                                <ConnectButton.Custom>
                                                    {({ openConnectModal }) => (
                                                        <button
                                                            className="text-white w-full text-lg"
                                                            onClick={openConnectModal}
                                                        >
                                                            Connect wallet
                                                        </button>
                                                    )}
                                                </ConnectButton.Custom>
                                            </div>
                                        </div>
                                    ) : (
                                        <div className="w-full flex justify-center">
                                            <div className="inline-block">
                                                <ConnectButton chainStatus="icon" accountStatus="avatar" />
                                            </div>
                                        </div>
                                    )}
                                </div>

                                <Link to="/profile" onClick={() => setIsProfileDropdownOpen(false)} className="flex items-center px-4 py-2 text-gray-700 hover:bg-gray-100">
                                    <FontAwesomeIcon icon={faUserCircle} size="lg" className='mr-2' />
                                    <span>Profile</span>
                                </Link>
                                <button onClick={logout} className="flex items-center w-full px-4 py-2 text-gray-700 hover:bg-gray-100">
                                    <FontAwesomeIcon icon={faSignOutAlt} size="lg" className='mr-2' />
                                    <span>Logout</span>
                                    {timeLeft !== null && (
                                        <span className="ml-auto text-sm text-gray-500">({formatTimeLeft(timeLeft)})</span>
                                    )}
                                </button>
                            </div>
                        )}
                    </div>
                )}
            </div>
        </nav >
    );
};

export default Menu;
