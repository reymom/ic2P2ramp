import React, { useState, useRef, useEffect } from 'react';

import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faChevronDown } from '@fortawesome/free-solid-svg-icons';

import { CURRENCY_ICON_MAP } from '../../constants/currencyIconsMap';

interface CurrencySelectProps {
    selected: string;
    onChange: (symbol: string) => void;
    className?: string;
    buttonClassName?: string;
    dropdownClassName?: string;
    compact?: boolean;
}

const CurrencySelect: React.FC<CurrencySelectProps> = ({ selected, onChange, className, buttonClassName, dropdownClassName, compact }) => {
    const [dropdownOpen, setDropdownOpen] = useState(false);
    const dropdownRef = useRef<HTMLDivElement>(null);

    const handleOptionSelect = (symbol: string) => {
        onChange(symbol);
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
                className={`w-full pl-3 pr-2 py-2 border focus:outline-none flex items-center justify-between ${buttonClassName}`}
                onClick={() => setDropdownOpen(!dropdownOpen)}
            >
                {/* Display selected currency symbol with icon */}
                {selected && (
                    <div>
                        <FontAwesomeIcon icon={CURRENCY_ICON_MAP[selected]} className="mr-2" />
                    </div>
                )}

                <FontAwesomeIcon icon={faChevronDown} className={`w-3 h-3 transition-transform ${dropdownOpen ? 'rotate-180' : ''}`} />
            </button>

            {/* Dropdown options */}
            {dropdownOpen && (
                <div className="absolute border border-inherit rounded-md mt-2 shadow-lg z-10 left-1/2 transform -translate-x-1/2 min-w-max">
                    {Object.keys(CURRENCY_ICON_MAP).map((currency, index) => (
                        <div
                            key={currency}
                            className={`flex items-center px-3 py-2 cursor-pointer transition-all ${dropdownClassName} 
                                ${index === 0 ? "rounded-t-md" : index === Object.keys(CURRENCY_ICON_MAP).length - 1 ? "rounded-b-md" : ""}`}
                            onClick={() => handleOptionSelect(currency)}
                        >
                            <FontAwesomeIcon icon={CURRENCY_ICON_MAP[currency]} className="mr-2" />
                            <span>{currency}</span>
                        </div>
                    ))}
                </div>
            )}
        </div>
    );
};

export default CurrencySelect;