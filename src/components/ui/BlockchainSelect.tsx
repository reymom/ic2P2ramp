import React, { useState, useRef, useEffect } from 'react';

import icpLogo from '../../assets/blockchains/icp-logo.svg';
import ethereumLogo from '../../assets/blockchains/ethereum-logo.png';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faChevronDown } from '@fortawesome/free-solid-svg-icons';

interface BlockchainSelectProps {
    selectedBlockchain: string | undefined;
    onChange: (blockchain: string) => void;
    className?: string;
    buttonClassName?: string;
}

const BlockchainSelect: React.FC<BlockchainSelectProps> = ({ selectedBlockchain, onChange, className, buttonClassName }) => {
    const [dropdownOpen, setDropdownOpen] = useState(false);
    const dropdownRef = useRef<HTMLDivElement>(null);

    const blockchainOptions = [
        { name: 'EVM', logo: ethereumLogo },
        { name: 'ICP', logo: icpLogo }
    ];

    const handleOptionSelect = (blockchain: string) => {
        onChange(blockchain);
        setDropdownOpen(false);
    };

    useEffect(() => {
        const handleClickOutside = (event: MouseEvent) => {
            if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
                setDropdownOpen(false);
            }
        };

        document.addEventListener('mousedown', handleClickOutside);
        return () => {
            document.removeEventListener('mousedown', handleClickOutside);
        };
    }, []);

    return (
        <div className={`relative ${className}`} ref={dropdownRef}>
            <button
                type="button"
                className={`w-full pl-3 pr-2 py-1.5 border focus:outline-none flex items-center justify-between ${buttonClassName}`}
                onClick={() => setDropdownOpen(!dropdownOpen)}
            >
                {/* Display selected blockchain with logo */}
                {selectedBlockchain ? (
                    <div>
                        <img src={blockchainOptions.find(b => b.name === selectedBlockchain)?.logo} alt="" className="h-6 w-6 inline-block mr-2" />
                        <span>{blockchainOptions.find(b => b.name === selectedBlockchain)?.name}</span>
                    </div>
                ) : (
                    <span>Select Blockchain</span>
                )}

                <FontAwesomeIcon icon={faChevronDown} className={`w-3 h-3 transition-transform ${dropdownOpen ? 'rotate-180' : ''}`} />
            </button>

            {/* Dropdown options */}
            {dropdownOpen && (
                <div className="absolute top-full border border-inherit rounded-md mt-2 shadow-lg z-10 w-full bg-gray-600 border-gray-500">
                    {blockchainOptions.map((blockchain, index) => (
                        <div
                            key={blockchain.name}
                            className={`flex items-center px-3 py-2 cursor-pointer hover:bg-gray-700 transition-all
                                ${index === 0 ? "rounded-t-md" : index === blockchainOptions.length - 1 ? "rounded-b-md" : ""}`}
                            onClick={() => handleOptionSelect(blockchain.name)}
                        >
                            <img src={blockchain.logo} alt={blockchain.name} className="h-6 w-6 inline-block mr-2" />
                            <span>{blockchain.name}</span>
                        </div>
                    ))}
                </div>
            )}
        </div>
    );
};

export default BlockchainSelect;
