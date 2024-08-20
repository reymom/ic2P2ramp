import React, { useEffect, useState } from 'react';
import { Address, OrderFilter } from '../../declarations/backend/backend.did';
import { stringToOrderFilter, stringToOrderStateFilter } from '../../model/utils';
import { OrderFilterTypes, OrderStateFilterTypes, UserTypes } from '../../model/types';
import { useAccount } from 'wagmi';
import { useUser } from '../../UserContext';

interface OrderFiltersProps {
    setFilter: (filter: OrderFilter | null) => void;
}

const OrderFilters: React.FC<OrderFiltersProps> = ({ setFilter }) => {
    const [filterType, setFilterType] = useState<OrderFilterTypes | null>(null);

    const { address, chainId } = useAccount();
    const { userType, loginMethod } = useUser();

    useEffect(() => {
        constructFilter();
    }, [filterType, address, chainId])

    const constructFilter = () => {
        if (!filterType) {
            setFilter(null)
            return;
        }

        const [filterCategory, filterValue] = filterType.split(':');
        switch (filterCategory) {
            case "ByState":
                setFilter(stringToOrderFilter(filterCategory, stringToOrderStateFilter(filterValue as OrderStateFilterTypes)));
                break;
            case "ByOfframperAddress": case "LockedByOnramper":
                setFilter(stringToOrderFilter(filterCategory, loginMethod));
                break;
            case "ByChainId":
                // setFilter(stringToOrderFilter(filterCategory, chainId));
                break;
            default:
                setFilter(null);
        }
    }

    const handleFilterTypeChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
        const value = e.target.value as OrderFilterTypes | "all";
        if (value === "all") {
            setFilterType(null);
        } else {
            setFilterType(value);
        }
    };

    return (
        <div className="mb-4">
            <label className="block text-gray-700">Filter:</label>
            <select
                value={filterType || 'all'}
                onChange={handleFilterTypeChange}
                className="px-3 py-2 border rounded"
            >
                <option value='all'>All</option>
                <option value='ByState:Locked'>Locked</option>
                <option value='ByState:Cancelled'>Cancelled</option>
                <option value='ByState:Created'>Created</option>
                <option value='ByState:Completed'>Completed</option>

                {chainId && (
                    <option value="ByChainId">Chain ID</option>
                )}
                {address && (
                    userType == "Offramper" ? (
                        <option value="ByOfframperAddress">By Offramper Address</option>
                    ) : (
                        <option value="LockedByOnramper">Locked by Onramper</option>
                    )
                )}
            </select>
        </div>
    );
}

export default OrderFilters;
