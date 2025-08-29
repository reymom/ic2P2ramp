import React, { useEffect, useRef, useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { ConnectButton } from '@rainbow-me/rainbowkit';
import { useAccount } from 'wagmi';
import clsx from 'clsx';

import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import {
    faUserCircle,
    faSignOutAlt,
    faFileAlt,
    faPlusCircle,
    faRightToBracket,
    faBars,
    faTimes,
    faSun,
    faMoon,
    faCog,
    faTimeline,
    IconDefinition
} from '@fortawesome/free-solid-svg-icons';
import icpLogo from "@/assets/blockchains/icp-logo.svg";
import ethereumLogo from "@/assets/blockchains/ethereum-logo.png";
import logo from '@/assets/icR-logo.png';
import bitcoinLogo from '@/assets/blockchains/bitcoin-logo.svg';
import solanaLogo from "@/assets/blockchains/solana-logo.png";

import { useUser } from './user/UserContext';
import { truncate, formatTimeLeft } from '@/utils/formatters';
import { sessionMarginMilisec } from '@/model/session';
import { getExplorerUrls } from '@/utils/explorers';

const Menu: React.FC = () => {
    const [isMenuOpen, setIsMenuOpen] = useState(false);
    const [isMobile, setIsMobile] = useState(window.innerWidth < 1024);
    const [isProfileDropdownOpen, setIsProfileDropdownOpen] = useState(false);
    const [timeLeft, setTimeLeft] = useState<number | null>(null);
    const [isDarkMode, setIsDarkMode] = useState(
        document.documentElement.classList.contains("dark")
    );

    const { isConnected, chainId } = useAccount();
    const {
        user,
        icpBalances,
        bitcoinAddress,
        bitcoinBalance,
        solanaPubkey,
        solanaBalance,
        connectUnisat,
        connectSolana,
        loginInternetIdentity,
        logout
    } = useUser();
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

    const toggleDarkMode = () => {
        document.documentElement.classList.toggle("dark");
        setIsDarkMode((prev) => !prev);
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

    const menuGroups: Record<string, { to: string; label: string; icon: IconDefinition }[]> = {
        "ONRAMPING": [
            { to: "/view", label: "Orders", icon: faFileAlt },
            { to: "/create", label: "Create Order", icon: faPlusCircle }
        ],
        "NFTs": [
            { to: "#", label: "Coming Soon", icon: faTimeline }
        ],
        "STAKING": [
            { to: "#", label: "Coming Soon", icon: faTimeline }
        ]
    };

    const renderLinkGroup = (links: { to: string; label: string; icon: IconDefinition }[], isMobile: boolean) => {
        return (
            <div className={clsx(isMobile ? "p-3" : "flex flex-col", "text-gray-800 dark:text-gray-300")}>
                {links.map(({ to, label, icon }) => {
                    const active = location.pathname === to;
                    return (
                        <Link
                            key={to}
                            to={to}
                            onClick={() => closeMenu()}
                            className={clsx(
                                isMobile ? "block px-4 py-3" : "flex items-center space-x-2 px-4 py-2",
                                "bg-gray-200 dark:bg-gray-800 hover:bg-gray-300 hover:dark:bg-gray-700 rounded-md transition-all ease-in-out",
                                { 'bg-gray-300 dark:bg-gray-700': active }
                            )}
                        >
                            <FontAwesomeIcon icon={icon} className="w-6" />
                            <span>{label}</span>
                        </Link>
                    )
                })}
            </div>
        );
    };

    const renderMenuGroups = (isMobile: boolean) => (
        Object.entries(menuGroups).map(([title, links]) => {
            const isTitleActive = links.some(({ to }) => location.pathname === to);

            return (
                isMobile ? (
                    <div key={title}>
                        <h3 className="text-lg font-semibold text-gray-800 dark:text-gray-300 px-4 py-2">{title}</h3>
                        {renderLinkGroup(links, true)}
                    </div>
                ) : (
                    <div key={title} className="relative group flex items-center justify-center">
                        <button
                            onClick={() => closeMenu()}
                            className={clsx(
                                "px-4 py-2 text-lg font-semibold rounded-md bg-transparent",
                                "hover:text-gray-500 hover:dark:text-white transition-all ease-in-out duration-300",
                                isTitleActive ? 'text-gray-500 dark:text-white' : 'text-gray-800 dark:text-gray-300'
                            )}
                        >
                            {title}
                        </button>
                        <div className={clsx(
                            "z-50 absolute left-0 top-full mt-2 w-48 bg-gray-300 dark:bg-gray-800",
                            "border border-gray-300 dark:border-gray-700 shadow-lg rounded-md opacity-0",
                            "group-hover:opacity-100 group-hover:visible transition-all duration-300"
                        )}>
                            {renderLinkGroup(links, false)}
                        </div>
                    </div>
                )
            )
        })
    );

    const icRampLogo = (isMobile: boolean, closeMenu?: () => void) => (
        <Link to="/" className={clsx("flex items-center space-x-1")} onClick={closeMenu}>
            <img
                src={logo}
                className={clsx(
                    "rounded-full mr-2",
                    isMobile ? "h-14 w-14" : "h-20 w-20"
                )}
                alt="icRamp logo"
            />
            <h1 className={clsx(
                "tracking-wider transition-colors duration-300 app-title",
                isMobile ? "text-2xl" : "text-4xl"
            )}>
                icRamp
            </h1>
        </Link>
    );

    const handleInternetIdentityLogin = async () => {
        await loginInternetIdentity();
    };

    const handleConnectUnisat = async () => {
        await connectUnisat();
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
        <nav className="py-6 px-16 flex justify-between items-center rounded-lg bg-transparent relative">
            {isMobile &&
                <>
                    <div className="flex items-center justify-between w-full">
                        <button onClick={toggleMenu} className="p-4 text-gray-700">
                            <FontAwesomeIcon icon={faBars} size="2x" />
                        </button>
                    </div>
                    <div
                        className={clsx(
                            "fixed inset-0 bg-gray-900 bg-opacity-75 z-50 lg:hidden",
                            isMenuOpen ? 'block' : 'hidden'
                        )}
                    >
                        <div
                            className={clsx(
                                "absolute top-0 left-0 w-64 h-full shadow-md transition-transform duration-300",
                                isMenuOpen ? 'translate-x-0' : '-translate-x-full',
                                "bg-gray-100 dark:bg-gray-900 border-r border-gray-300 dark:border-gray-700"
                            )}
                            ref={menuRef}
                        >
                            <div className="p-4 flex items-center justify-between">
                                {icRampLogo(true, closeMenu)}
                                <button onClick={toggleMenu} className="p-2 text-gray-600 dark:text-gray-300">
                                    <FontAwesomeIcon icon={faTimes} size="lg" />
                                </button>
                            </div>

                            <div className="p-4 space-y-4">
                                {renderMenuGroups(true)}
                            </div>

                            {icpBalances && icpBalances['ICP'] && (
                                <div className="p-4">
                                    <div
                                        className={clsx(
                                            "border border-gray-300 dark:border-gray-700 rounded",
                                            "px-4 py-2 text-green-500 text-center font-medium bg-white dark:bg-gray-800"
                                        )}>
                                        {icpBalances['ICP'].formatted} ICP
                                    </div>
                                </div>
                            )}
                        </div>
                    </div>
                </>
            }

            {!isMobile &&
                <div className="flex items-center w-full space-x-10">
                    {icRampLogo(false)}
                    <div className="flex items-center space-x-6">
                        {renderMenuGroups(false)}
                    </div>
                </div>
            }

            <div className="w-72 flex justify-end items-center relative space-x-2">
                <div className="flex space-x-2">
                    {/* Theme Toggle */}
                    <button
                        onClick={toggleDarkMode}
                        className={clsx("px-3 py-2 rounded-lg transition-colors duration-200",
                            "bg-gray-100 hover:bg-gray-200",
                            "dark:bg-gray-800 dark:hover:bg-gray-700",
                            "text-gray-600 dark:text-gray-200"
                        )}>
                        <FontAwesomeIcon icon={isDarkMode ? faSun : faMoon} size="lg" />
                    </button>
                    {/* Settings */}
                    <button
                        className={clsx(
                            "px-3 py-2 rounded-lg transition-colors duration-200",
                            "bg-gray-100 hover:bg-gray-200",
                            "dark:bg-gray-800 dark:hover:bg-gray-700",
                            "text-gray-600 dark:text-gray-200"
                        )}>
                        <FontAwesomeIcon icon={faCog} size="lg" />
                    </button>
                </div>
                {!user ? (
                    <div className="relative">
                        <button
                            onClick={() => navigate('/')}
                            className={clsx(
                                "flex items-center gap-2 px-6 py-2 rounded-lg font-medium text-gray-200 dark:text-white text-xl",
                                "bg-gradient-to-r from-indigo-700 to-blue-600",
                                "hover:from-blue-700 hover:to-indigo-600",
                                "transition-all, duration-200 shadow-lg hover:shadow-xl"
                            )}
                        >
                            <FontAwesomeIcon icon={faRightToBracket} size="lg" className="mr-2" />
                            <span>Login</span>
                        </button>
                    </div>
                ) : (
                    <div className="relative" ref={profileDropdownRef}>
                        {/* Dropdown */}
                        <button onClick={toggleProfileDropdown} className="flex items-center space-x-2 p-2 border border-gray-400 rounded-lg transition-all">
                            <FontAwesomeIcon icon={faUserCircle} size="lg" className="text-violet-800 dark:text-violet-700" />
                            <svg className={`w-4 h-4 ml-1 transform ${isProfileDropdownOpen ? 'rotate-180' : ''}`} fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M19 9l-7 7-7-7"></path>
                            </svg>
                        </button>
                        {isProfileDropdownOpen && (
                            <div className="absolute right-0 mt-2 w-72 bg-white dark:bg-gray-800 shadow-lg rounded-lg z-50">
                                <div className="p-4 text-gray-700 dark:text-gray-300 border-b border-gray-200 dark:border-gray-700">

                                    <div className="flex items-center text-center">
                                        {(() => {
                                            const { address, explorerUrl } = (() => {
                                                if ('EVM' in user.login) {
                                                    if (!chainId) return { address: '', explorerUrl: null };
                                                    const urls = getExplorerUrls('EVM', user.login.EVM.address, BigInt(chainId));
                                                    return { address: user.login.EVM.address, explorerUrl: urls?.address };
                                                }
                                                if ('ICP' in user.login) {
                                                    const urls = getExplorerUrls('ICP', user.login.ICP.principal_id);
                                                    return { address: user.login.ICP.principal_id, explorerUrl: urls?.address };
                                                }
                                                if ('Solana' in user.login) {
                                                    const urls = getExplorerUrls('Solana', user.login.Solana.address);
                                                    return { address: user.login.Solana.address, explorerUrl: urls?.address };
                                                }
                                                if ('Email' in user.login) {
                                                    return { address: user.login.Email.email, explorerUrl: null };
                                                }
                                                if ('Bitcoin' in user.login) {
                                                    const urls = getExplorerUrls('ICP', user.login.Bitcoin.address);
                                                    return { address: user.login.Bitcoin.address, explorerUrl: urls?.address };
                                                }
                                                return { address: '', explorerUrl: null };
                                            })();

                                            return (
                                                <span className="flex-grow text-sm font-semibold text-blue-600 dark:text-blue-400 truncate">
                                                    {explorerUrl ? (
                                                        <a href={explorerUrl} target="_blank" rel="noopener noreferrer">
                                                            {truncate(address, 12, 12)}
                                                        </a>
                                                    ) : (
                                                        truncate(address, 12, 12)
                                                    )}
                                                </span>
                                            );
                                        })()}
                                    </div>

                                    <hr className="border-t border-gray-300 dark:border-gray-600 w-full my-2" />

                                    {icpBalances && icpBalances['ICP'] ? (
                                        <div className="relative flex justify-center items-center border border-gray-300 dark:border-gray-600 rounded-md px-3 py-2 text-green-800 dark:text-green-400 text-center font-medium">
                                            <img src={icpLogo} alt="ICP Logo" className="h-6 w-6 absolute left-3" />
                                            <span className="text-lg">{icpBalances['ICP'].formatted} ICP</span>
                                        </div>
                                    ) : (
                                        <div
                                            className="relative flex justify-center items-center px-3 py-2 bg-amber-400 dark:bg-amber-700 rounded-md hover:bg-amber-300 dark:hover:bg-amber-800 cursor-pointer"
                                            onClick={handleInternetIdentityLogin}
                                        >
                                            <img src={icpLogo} alt="ICP Logo" className="h-6 w-6 absolute left-3" />
                                            <span className="text-black dark:text-white text-lg">Connect ICP</span>
                                        </div>
                                    )}

                                    <hr className="border-t border-gray-300 dark:border-gray-600 w-full my-2" />

                                    {!isConnected ? (
                                        <div className="relative flex justify-center items-center px-3 py-2 bg-amber-400 dark:bg-amber-700 rounded-md hover:bg-amber-300 dark:hover:bg-amber-800 cursor-pointer">
                                            <img src={ethereumLogo} alt="Ethereum Logo" className="h-6 w-6 absolute left-3" />
                                            <div className="w-full text-left">
                                                <ConnectButton.Custom>
                                                    {({ openConnectModal }) => (
                                                        <button
                                                            className="text-black dark:text-white w-full text-lg"
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

                                    <hr className="border-t border-gray-300 dark:border-gray-600 w-full my-2" />

                                    {bitcoinAddress && bitcoinBalance ? (
                                        <div className="relative flex justify-center items-center border border-gray-300 dark:border-gray-600 rounded-md px-3 py-2 text-green-800 dark:text-green-400 text-center font-medium">
                                            <img src={bitcoinLogo} alt="Bitocoin Logo" className="h-6 w-6 absolute left-3" />
                                            <span className="text-lg">{bitcoinBalance.balance.formatted} BTC</span>
                                        </div>
                                    ) : (
                                        <div
                                            className="relative flex justify-center items-center px-3 py-2 bg-amber-400 dark:bg-amber-700 rounded-md hover:bg-amber-300 dark:hover:bg-amber-800 cursor-pointer"
                                            onClick={handleConnectUnisat}>
                                            <img src={bitcoinLogo} alt="Bitocoin Logo" className="h-6 w-6 absolute left-3" />
                                            <span className="text-black dark:text-white text-lg">Connect Unisat</span>
                                        </div>
                                    )}

                                    <hr className="border-t border-gray-300 dark:border-gray-600 w-full my-2" />

                                    {solanaPubkey && solanaBalance?.balance ? (
                                        <div className="relative flex justify-center items-center border border-gray-300 dark:border-gray-600 rounded-md px-3 py-2 text-green-800 dark:text-green-400 text-center font-medium">
                                            <img src={solanaLogo} alt="Solana Logo" className="h-6 w-6 absolute left-3" />
                                            <span className="text-lg">{solanaBalance.balance.formatted} SOL</span>
                                        </div>
                                    ) : (
                                        <div
                                            className="relative flex justify-center items-center px-3 py-2 bg-amber-400 dark:bg-amber-700 rounded-md hover:bg-amber-300 dark:hover:bg-amber-800 cursor-pointer"
                                            onClick={connectSolana}
                                        >
                                            <img src={solanaLogo} alt="Solana Logo" className="h-6 w-6 absolute left-3" />
                                            <span className="text-black dark:text-white text-lg">Connect Solana</span>
                                        </div>
                                    )}

                                </div>

                                <Link to="/profile" onClick={() => setIsProfileDropdownOpen(false)} className="flex items-center px-4 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700">
                                    <FontAwesomeIcon icon={faUserCircle} size="lg" className='mr-2' />
                                    <span>Profile</span>
                                </Link>
                                <button onClick={logout} className="flex items-center w-full px-4 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700">
                                    <FontAwesomeIcon icon={faSignOutAlt} size="lg" className='mr-2' />
                                    <span>Logout</span>
                                    {timeLeft !== null && (
                                        <span className="ml-auto text-sm text-gray-500 dark:text-gray-400">({formatTimeLeft(timeLeft)})</span>
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
