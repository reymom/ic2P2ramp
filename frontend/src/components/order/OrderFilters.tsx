import React, { useEffect, useState } from 'react';
import { FilterX } from "lucide-react";
import clsx from 'clsx';

import {
    TransactionAddress,
    OrderFilter,
    BlockchainAsset,
    OrderStateFilter,
    BlockchainType
} from '@/declarations/icramp_backend/icramp_backend.did';
import { OrderFilterTypes } from '@/model/types';
import { truncate } from '@/utils/formatters';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { useUser } from '@/components/user/UserContext';
import icpLogo from '@/assets/blockchains/icp-logo.svg';
import ethereumLogo from '@/assets/blockchains/ethereum-logo.png';
import bitcoinLogo from '@/assets/blockchains/bitcoin-logo.svg';
import solanaLogo from '@/assets/blockchains/solana-logo.png';

interface OrderFiltersProps {
    setFilter: (filter: OrderFilter | null) => void;
    currentFilter: OrderFilter | null;
}

const OrderFilters: React.FC<OrderFiltersProps> = ({ setFilter, currentFilter }) => {
    const [filterType, setFilterType] = useState<OrderFilterTypes | null>(null);
    const [selectedState, setSelectedState] = useState<OrderStateFilter | null>(null);
    const [selectedAddress, setSelectedAddress] = useState<TransactionAddress | null>(null);
    const [selectedBlockchainAsset, setSelectedBlockchainAsset] = useState<BlockchainAsset | null>(null);
    const [selectedBlockchainType, setSelectedBlockchainType] = useState<BlockchainType | null>(null);

    const { user, userType } = useUser();

    useEffect(() => {
        if (currentFilter) {
            setFilterTypeFromCurrentFilter(currentFilter);
        }
    }, [currentFilter]);

    const setFilterTypeFromCurrentFilter = (filter: OrderFilter) => {
        if ('ByState' in filter) {
            setFilterType('ByState');
            setSelectedState(filter.ByState);
        } else if ('ByOfframperId' in filter) {
            setFilterType('ByOfframperId');
        } else if ('ByOnramperId' in filter) {
            setFilterType('ByOnramperId');
        } else if ('ByBlockchain' in filter) {
            setFilterType('ByBlockchain')
        } else if ('ByBlockchainAsset' in filter) {
            setFilterType('ByBlockchainAsset');
            setSelectedBlockchainAsset(filter.ByBlockchainAsset);
        } else if ('ByOfframperAddress' in filter) {
            setFilterType('ByOfframperAddress');
            setSelectedAddress(filter.ByOfframperAddress);
        } else if ('LockedByOnramper' in filter) {
            setFilterType('LockedByOnramper');
            setSelectedAddress(filter.LockedByOnramper);
        } else {
            setFilterType(null);
        }
    };

    // useEffect(() => {
    //     constructFilter();
    // }, [filterType, selectedState, selectedBlockchainType, selectedBlockchainAsset, selectedAddress])

    // const constructFilter = () => {
    //     if (!filterType) {
    //         setFilter(null)
    //         return;
    //     }

    //     switch (filterType) {
    //         case "ByState":
    //             if (selectedState) {
    //                 setFilter({ ByState: selectedState });
    //             }
    //             break;
    //         case "ByOfframperAddress": case "LockedByOnramper":
    //             if (selectedAddress) {
    //                 setFilter({ [filterType]: selectedAddress } as OrderFilter)
    //             }
    //             break;
    //         case "ByBlockchain":
    //             if (selectedBlockchainType) {
    //                 console.log("By Blockchain = ", selectedBlockchainType);
    //                 setFilter({ ByBlockchain: selectedBlockchainType } as OrderFilter)
    //             }
    //             break;
    //         case "ByBlockchainAsset":
    //             if (selectedBlockchainAsset) {
    //                 setFilter({ [filterType]: selectedBlockchainAsset } as OrderFilter);
    //             }
    //             break;
    //         case "ByOfframperId": case "ByOnramperId":
    //             if (user) {
    //                 setFilter({ [filterType]: user.id } as OrderFilter)
    //             }
    //             break;
    //         default:
    //             setFilter(null);
    //     }
    // }

    const handleFilterTypeChange = (value: string) => {
        if (!value.startsWith("ByBlockchain:")) setSelectedBlockchainType(null);
        if (!value.startsWith("ByEVMChain")) setSelectedBlockchainAsset(null);
        if (!value.startsWith("ByState")) setSelectedState(null);
        if (value !== "ByOfframperAddress" && value !== "LockedByOnramper") setSelectedAddress(null);

        if (value === "all") { setFilterType(null); setFilter(null); return; }

        if (value.startsWith("ByState:")) {
            type StateKey = 'Created' | 'Locked' | 'Completed' | 'Cancelled';
            const k = value.split(':')[1] as StateKey;
            const nextState = { [k]: null } as unknown as OrderStateFilter;
            setFilterType('ByState');
            setSelectedState(nextState);
            setFilter({ ByState: nextState });
            return;
        }

        if (value.startsWith("ByBlockchain:")) {
            const kind = value.split(":")[1] as 'EVM' | 'ICP' | 'Bitcoin' | 'Solana';
            const chain = { [kind]: null } as BlockchainType;
            setFilterType("ByBlockchain"); setSelectedBlockchainType(chain);
            setFilter({ ByBlockchain: chain });
            return;
        }

        setFilterType(value as OrderFilterTypes);

        if ((value === "ByOfframperAddress" || value === "LockedByOnramper") && user?.addresses.length) {
            const addr = user.addresses[0];
            setSelectedAddress(addr);
            setFilter({ [value]: addr } as OrderFilter);
            return;
        }

        if (value === "ByOfframperId" && user) { setFilter({ ByOfframperId: user.id }); return; }
        if (value === "ByOnramperId" && user) { setFilter({ ByOnramperId: user.id }); return; }
    };

    const isSameState = (state1: OrderStateFilter | null, state2: OrderStateFilter | null) => {
        if (!state1 || !state2) return false;
        return JSON.stringify(state1) === JSON.stringify(state2);
    };

    return (
        <div className="flex items-center justify-between w-full">
            {/* Blockchain Filters */}
            <div className="flex items-center gap-2">
                <button
                    onClick={() => handleFilterTypeChange("ByBlockchain:EVM")}
                    className={`w-12 h-10 rounded-md flex items-center justify-center
                        ${selectedBlockchainType && 'EVM' in selectedBlockchainType ? "bg-blue-700" : "bg-gray-200 dark:bg-gray-700 hover:bg-gray-600"}`}
                >
                    <img src={ethereumLogo} alt="Ethereum" className="w-8 h-8" />
                </button>
                <button
                    onClick={() => handleFilterTypeChange("ByBlockchain:ICP")}
                    className={`w-12 h-10 rounded-md border flex items-center justify-center
                        ${selectedBlockchainType && 'ICP' in selectedBlockchainType ? "bg-blue-700" : "bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600"}`}
                >
                    <img src={icpLogo} alt="ICP" className="w-8 h-8" />
                </button>
                <button
                    onClick={() => handleFilterTypeChange("ByBlockchain:Bitcoin")}
                    className={`w-12 h-10 rounded-md border flex items-center justify-center
                        ${selectedBlockchainType && 'Bitcoin' in selectedBlockchainType ? "bg-blue-700" : "bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600"}`}
                >
                    <img src={bitcoinLogo} alt="Bitcoin" className="w-8 h-8" />
                </button>
                <button
                    onClick={() => handleFilterTypeChange("ByBlockchain:Solana")}
                    className={`w-12 h-10 rounded-md border flex items-center justify-center
                        ${selectedBlockchainType && 'Solana' in selectedBlockchainType ? "bg-blue-700" : "bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600"}`}
                >
                    <img src={solanaLogo} alt="Solana" className="w-8 h-8" />
                </button>
            </div>

            {/* Middle Filters */}
            <div className="flex gap-2 justify-center flex-grow">
                <button
                    onClick={() => handleFilterTypeChange("ByState:Created")}
                    className={`w-[120px] h-10 text-center rounded-md text-sm 
                        ${filterType === "ByState" && isSameState(selectedState, { "Created": null }) ?
                            "bg-blue-500 dark:bg-blue-600 text-black dark:text-white" : "bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600"}`}
                >
                    Created
                </button>
                <button
                    onClick={() => handleFilterTypeChange("ByState:Locked")}
                    className={`w-[120px] h-10 text-center rounded-md text-sm 
                        ${filterType === "ByState" && isSameState(selectedState, { "Locked": null }) ?
                            "bg-blue-500 dark:bg-blue-600 text-black dark:text-white" : "bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600"}`}
                >
                    Locked
                </button>
                {/* Offramper/Onramper Filters */}
                <button
                    onClick={() => handleFilterTypeChange(userType === "Offramper" ? "ByOfframperId" : "ByOnramperId")}
                    className={`w-[120px] h-10 text-center rounded-md text-sm 
                        ${filterType === (userType === "Offramper" ? "ByOfframperId" : "ByOnramperId") ?
                            "bg-blue-500 dark:bg-blue-600 text-black dark:text-white" : "bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600"}`}
                >
                    My Orders
                </button>

                <Select
                    value={
                        filterType === "ByState" && selectedState &&
                            (isSameState(selectedState, { Completed: null }) || isSameState(selectedState, { Cancelled: null }))
                            ? `ByState:${Object.keys(selectedState)[0]}`
                            : ""
                    }
                    onValueChange={handleFilterTypeChange}>
                    <SelectTrigger className={`w-[120px] h-10 rounded-md transition  
                        ${filterType === "ByState" && selectedState &&
                            (isSameState(selectedState, { Completed: null }) || isSameState(selectedState, { Cancelled: null }))
                            ? "bg-blue-500 dark:bg-blue-600 text-black dark:text-white"
                            : "bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark-hover-bg-gray-600"}
                    `}>
                        <SelectValue placeholder="More Filters" />
                    </SelectTrigger>
                    <SelectContent>
                        {user && <SelectItem value="ByOfframperAddress" className="cursor-pointer">By Offramper Address</SelectItem>}
                        {user && <SelectItem value="LockedByOnramper" className="cursor-pointer">Locked by Onramper</SelectItem>}
                        <SelectItem value="ByState:Completed" className="cursor-pointer">Completed</SelectItem>
                        <SelectItem value="ByState:Cancelled" className="cursor-pointer">Cancelled</SelectItem>
                    </SelectContent>
                </Select>

                {user && (filterType === 'ByOfframperAddress' || filterType === 'LockedByOnramper') && (
                    <Select
                        value={selectedAddress?.address ?? undefined}
                        onValueChange={(value) => {
                            const address = user?.addresses.find(addr => addr.address === value);
                            setSelectedAddress(address || null);
                            if (address && (filterType === 'ByOfframperAddress' || filterType === 'LockedByOnramper')) {
                                setFilter({ [filterType]: address } as OrderFilter);
                            }
                        }}
                    >
                        <SelectTrigger className="w-full md:w-[180px]">
                            <SelectValue placeholder="Select address" />
                        </SelectTrigger>
                        <SelectContent>
                            {user?.addresses.map((addr, index) => (
                                <SelectItem key={index} value={addr.address}>
                                    {truncate(addr.address, 10, 10)} ({Object.keys(addr.address_type)[0]})
                                </SelectItem>
                            ))}
                        </SelectContent>
                    </Select>
                )}

                <button
                    onClick={() => {
                        setFilterType(null);
                        setSelectedBlockchainAsset(null);
                        setSelectedBlockchainType(null);
                        setSelectedState(null);
                        setSelectedAddress(null);
                        setFilter(null);
                    }}
                    disabled={filterType === null}
                    className={clsx(
                        "inline-flex items-center gap-2 px-3 py-1 text-sm text-muted-foreground",
                        filterType !== null && "hover:text-foreground",
                    )}>
                    <FilterX className="h-4 w-4" />
                    Clear
                </button>

            </div>
        </div>
    );
}

export default OrderFilters;
