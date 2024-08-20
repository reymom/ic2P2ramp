import React, { useEffect, useRef, useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faUserCircle, faSignOutAlt, faFileAlt, faPlusCircle, faRightToBracket, faBars, faTimes } from '@fortawesome/free-solid-svg-icons';

import logo from '../assets/p2ploan.webp';
import { useUser } from '../UserContext';
import { userTypeToString } from '../model/utils';
import GetBalanceComponent from './icp/Balance';
import { truncate } from '../model/helper';
import { useAccount } from 'wagmi';
import { ConnectButton } from '@rainbow-me/rainbowkit';

const Menu: React.FC = () => {
    const [isMenuOpen, setIsMenuOpen] = useState(false);
    const [isMobile, setIsMobile] = useState(window.innerWidth < 1024);
    const [isProfileDropdownOpen, setIsProfileDropdownOpen] = useState(false);

    const { isConnected } = useAccount();
    const { user, principal, logout } = useUser();
    const navigate = useNavigate();

    const profileDropdownRef = useRef<HTMLDivElement>(null);
    const menuRef = useRef<HTMLDivElement>(null);


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
        if (!user) {
            return (
                <>
                    <Link to="/view" onClick={closeMenu} className="flex items-center space-x-2 py-2 px-4 lg:inline-block lg:py-0">
                        <FontAwesomeIcon icon={faFileAlt} />
                        <span>View Orders</span>
                    </Link>
                </>
            );
        }

        switch (userTypeToString(user.user_type)) {
            case "Onramper":
                return (
                    <Link to="/view" onClick={closeMenu} className="flex items-center space-x-2 py-2 px-4 lg:inline-block lg:py-0">
                        <FontAwesomeIcon icon={faFileAlt} />
                        <span>View Orders</span>
                    </Link>
                );
            case "Offramper":
                return (
                    <>
                        <Link to="/view" onClick={closeMenu} className="flex items-center space-x-2 py-2 px-4 lg:inline-block lg:py-0">
                            <FontAwesomeIcon icon={faFileAlt} />
                            <span>My Orders</span>
                        </Link>
                        <Link to="/create" onClick={closeMenu} className="flex items-center space-x-2 py-2 px-4 lg:inline-block lg:py-0">
                            <FontAwesomeIcon icon={faPlusCircle} />
                            <span>Create Order</span>
                        </Link>
                    </>
                );
            default:
                return null;
        }
    };

    return (
        <nav className="p-6 flex justify-between items-center rounded-lg" style={{ backgroundColor: 'transparent' }}>
            {isMobile &&
                <>
                    <div className="flex items-center justify-between w-full">
                        <button onClick={toggleMenu} className="p-4">
                            <FontAwesomeIcon icon={faBars} size="2x" />
                        </button>
                    </div>
                    <div className={`fixed inset-0 bg-gray-800 bg-opacity-75 z-50 lg:hidden ${isMenuOpen ? 'block' : 'hidden'}`}>
                        <div className="absolute top-0 left-0 w-64 bg-white h-full shadow-md" ref={menuRef}>
                            <div className="p-4 flex items-center justify-between">
                                <div>
                                    <Link to="/" className="flex items-center" onClick={closeMenu}>
                                        <img src={logo} className="rounded-full h-12 w-12 mr-2" alt="ic2P2ramp logo" />
                                    </Link>
                                </div>
                                <button onClick={toggleMenu} className="p-2">
                                    <FontAwesomeIcon icon={faTimes} size="lg" className="text-gray-600" />
                                </button>
                            </div>
                            <div className="p-4 flex-grow">
                                {renderLinks()}
                            </div>
                            <div className="p-4">
                                {principal && <GetBalanceComponent principal={principal} />}
                            </div>
                        </div>
                    </div>
                </>
            }

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
                </div>
            }

            <div className="w-72 flex justify-end items-center relative">
                {!user ? (
                    <div className="relative">
                        <button onClick={() => navigate('/')} className="bg-blue-500 flex items-center p-2 text-white rounded-full">
                            <FontAwesomeIcon icon={faRightToBracket} size="2x" className="text-gray-800 mr-2" />
                            <span>Login</span>
                        </button>
                    </div>
                ) : (
                    <div className="relative" ref={profileDropdownRef}>
                        <button onClick={toggleProfileDropdown} className="flex items-center space-x-2 p-2 bg-gray-600 text-white rounded-full">
                            <FontAwesomeIcon icon={faUserCircle} size="3x" className="text-white" />
                            <svg className={`w-4 h-4 ml-1 transform ${isProfileDropdownOpen ? 'rotate-180' : ''}`} fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M19 9l-7 7-7-7"></path>
                            </svg>
                        </button>
                        {isProfileDropdownOpen && (
                            <div className="absolute right-0 mt-2 w-64 bg-white shadow-lg rounded-lg z-10">
                                <div className="p-4 text-gray-700 border-b border-gray-200">

                                    <div className="flex items-center text-center">
                                        <span className="flex-grow text-sm font-semibold text-blue-600 truncate">{truncate(user.login_method.address, 10, 10)}</span>
                                        <span className="ml-2 text-gray-500">({Object.keys(user.login_method.address_type)[0]})</span>
                                    </div>

                                    {principal && (
                                        <div className="items-center text-center">
                                            <>
                                                <hr className="border-t border-gray-300 w-full my-2" />
                                                <GetBalanceComponent principal={principal} />
                                            </>
                                        </div>
                                    )}

                                    {isConnected && (
                                        <>
                                            <hr className="border-t border-gray-300 w-full my-2" />

                                            <div className="w-full flex justify-center">
                                                <div className="inline-block">
                                                    <ConnectButton chainStatus="icon" accountStatus="avatar" />
                                                </div>
                                            </div>
                                        </>
                                    )}

                                </div>
                                <Link to="/profile" onClick={() => setIsProfileDropdownOpen(false)} className="flex items-center px-4 py-2 text-gray-700 hover:bg-gray-100">
                                    <FontAwesomeIcon icon={faUserCircle} size="lg" className='mr-2' />
                                    <span>Profile</span>
                                </Link>
                                <button onClick={logout} className="flex items-center w-full px-4 py-2 text-gray-700 hover:bg-gray-100">
                                    <FontAwesomeIcon icon={faSignOutAlt} size="lg" className='mr-2' />
                                    <span>Logout</span>
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
