import React, { useState, useRef, useEffect } from 'react';

import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faChevronDown } from '@fortawesome/free-solid-svg-icons';

import { TokenOption } from '../../model/types';

interface TokenSelectProps {
    tokenOptions: TokenOption[];
    selectedToken: TokenOption | null;
    onChange: (token: string) => void;
    className?: string;
    buttonClassName?: string;
}

const TokenSelect: React.FC<TokenSelectProps> = ({ tokenOptions, selectedToken, onChange, className, buttonClassName }) => {
    const [dropdownOpen, setDropdownOpen] = useState(false);
    const dropdownRef = useRef<HTMLDivElement>(null);

    const handleOptionSelect = (tokenAddress: string) => {
        onChange(tokenAddress);
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
                className={`w-full pl-3 pr-2 py-1.5 border focus:outline-none flex items-center justify-between ${buttonClassName} 
                    ${tokenOptions.length == 0 ? "cursor-pointer bg-gray-500" : ""}`}
                onClick={() => setDropdownOpen(!dropdownOpen)}
                disabled={tokenOptions.length == 0}
            >
                {/* Display selected token with logo */}
                {selectedToken ? (
                    <div>
                        <img src={selectedToken.logo} alt="" className="h-6 w-6 inline-block mr-2" />
                        <span>{selectedToken.name}</span>
                    </div>
                ) : (
                    <span>Select Token</span>
                )}

                <FontAwesomeIcon icon={faChevronDown} className={`w-3 h-3 transition-transform ${dropdownOpen ? 'rotate-180' : ''}`} />
            </button>

            {/* Dropdown options */}
            {dropdownOpen && (
                <div className="absolute top-full border border-inherit rounded-md mt-2 shadow-lg z-10 w-full bg-gray-600 border-gray-500">
                    {tokenOptions.map((token, index) => (
                        <div
                            key={token.address}
                            className={`flex items-center px-3 py-2 cursor-pointer hover:bg-gray-700 transition-all 
                                ${index === 0 ? "rounded-t-md" : index === tokenOptions.length - 1 ? "rounded-b-md" : ""}`}
                            onClick={() => handleOptionSelect(token.address)}
                        >
                            <img src={token.logo} alt={token.name} className="h-6 w-6 inline-block mr-2" />
                            <span>{token.name}</span>
                        </div>
                    ))}
                </div>
            )}
        </div>
    );
};

export default TokenSelect;
