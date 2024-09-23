import React, { useState, useRef, useEffect } from 'react';

import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faChevronDown } from '@fortawesome/free-solid-svg-icons';

import { CURRENCY_ICON_MAP } from '../../constants/currencyIconsMap';

interface CurrencySelectProps {
    selected: string;
    onChange: (symbol: string) => void;
}

const CurrencySelect: React.FC<CurrencySelectProps> = ({ selected, onChange }) => {
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
        <div className="relative w-1/6" ref={dropdownRef}>
            <button
                type="button"
                className="w-full pl-3 pr-2 py-2 border border-gray-500 bg-gray-600 text-white rounded-r-lg focus:outline-none flex items-center justify-between"
                onClick={() => setDropdownOpen(!dropdownOpen)}
            >
                {/* Display selected currency symbol with icon */}
                {selected ? (
                    <div>
                        <FontAwesomeIcon icon={CURRENCY_ICON_MAP[selected]} className="mr-2" />
                    </div>
                ) : (
                    <span>Select Currency</span>
                )}

                <FontAwesomeIcon icon={faChevronDown} className={`w-3 h-3 transition-transform ${dropdownOpen ? 'rotate-180' : ''}`} />
            </button>

            {/* Dropdown options */}
            {dropdownOpen && (
                <div className="absolute bg-gray-600 text-white rounded-md mt-2 shadow-lg z-10 left-1/2 transform -translate-x-1/2 min-w-max">
                    {Object.keys(CURRENCY_ICON_MAP).map((currency) => (
                        <div
                            key={currency}
                            className="flex items-center px-3 py-2 hover:bg-gray-500 cursor-pointer"
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