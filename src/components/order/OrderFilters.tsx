import React, { useEffect, useState } from 'react';
import { Principal } from '@dfinity/principal';

import { TransactionAddress, OrderFilter, Blockchain, OrderStateFilter } from '../../declarations/backend/backend.did';
import { NetworkIds } from '../../constants/networks';
import { ICP_TOKENS } from '../../constants/icp_tokens';
import { BlockchainTypes, OrderFilterTypes } from '../../model/types';
import { useUser } from '../user/UserContext';
import { truncate } from '../../model/helper';

interface OrderFiltersProps {
    setFilter: (filter: OrderFilter | null) => void;
    currentFilter: OrderFilter | null;
}

const OrderFilters: React.FC<OrderFiltersProps> = ({ setFilter, currentFilter }) => {
    const [filterType, setFilterType] = useState<OrderFilterTypes | null>(null);

    const [selectedState, setSelectedState] = useState<OrderStateFilter | null>(null);
    const [selectedAddress, setSelectedAddress] = useState<TransactionAddress | null>(null);
    const [selectedBlockchain, setSelectedBlockchain] = useState<Blockchain | null>(null);
    const [blockchainType, setBlockchainType] = useState<BlockchainTypes | null>(null);

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
            setFilterType('ByBlockchain');
            setSelectedBlockchain(filter.ByBlockchain);
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

    useEffect(() => {
        constructFilter();
    }, [filterType, selectedState, selectedBlockchain, selectedAddress])

    const constructFilter = () => {
        if (!filterType) {
            setFilter(null)
            return;
        }

        switch (filterType) {
            case "ByState":
                if (selectedState) {
                    setFilter({ ByState: selectedState });
                }
                break;
            case "ByOfframperAddress": case "LockedByOnramper":
                if (selectedAddress) {
                    setFilter({ [filterType]: selectedAddress } as OrderFilter)
                }
                break;
            case "ByBlockchain":
                if (selectedBlockchain) {
                    setFilter({ [filterType]: selectedBlockchain } as OrderFilter);
                }
                break;
            case "ByOfframperId": case "ByOnramperId":
                if (user) {
                    setFilter({ [filterType]: user.id } as OrderFilter)
                }
                break;
            default:
                setFilter(null);
        }
    }

    const handleFilterTypeChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
        const value = e.target.value as OrderFilterTypes | "all";

        if (value !== "ByBlockchain") {
            setBlockchainType(null);
            setSelectedBlockchain(null);
        }
        if (!(value in Array(["ByOfframperAddress", "LockedByOnramper"]))) {
            setSelectedAddress(null);
        }
        if (!value.startsWith('ByState')) setSelectedState(null);

        if (value === "all") {
            setFilterType(null);
            setFilter(null);
        } else if (value.startsWith('ByState')) {
            const stateValue = value.split(':')[1];
            setFilterType('ByState');
            setSelectedState({ [stateValue]: null } as OrderStateFilter);
        } else {
            setFilterType(value);
        }
    };

    const handleAddressChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
        const address = user?.addresses.find(addr => addr.address === e.target.value);
        setSelectedAddress(address || null);
    };

    const handleChainIdChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
        if (e.target.value == "") {
            setSelectedBlockchain(null);
            return;
        };
        const chainId = parseInt(e.target.value, 10);
        setSelectedBlockchain({ EVM: { chain_id: BigInt(chainId) } });
    }

    const handleCanisterChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
        if (e.target.value == "") {
            setSelectedBlockchain(null);
            return;
        }
        const ledgerCanister = Principal.fromText(e.target.value);
        setSelectedBlockchain({ ICP: { ledger_principal: ledgerCanister } })
    }

    return (
        <div className="flex gap-4 items-center flex-grow">
            <select
                value={filterType ? `${filterType}${selectedState ? `:${Object.keys(selectedState)[0]}` : ''}` : 'all'}
                onChange={handleFilterTypeChange}
                className="w-full px-3 py-2 border-gray-500 bg-gray-600 border rounded focus:outline-none focus:ring-2 focus:ring-blue-900"
            >
                <option value='all'>All</option>
                <option value='ByState:Created'>Created</option>
                <option value='ByState:Locked'>Locked</option>
                <option value='ByState:Cancelled'>Cancelled</option>
                <option value='ByState:Completed'>Completed</option>

                {userType == "Offramper" ? (
                    <option value="ByOfframperId">By Offramper (me)</option>
                ) : userType == "Onramper" ? (
                    <option value="ByOnramperId">By Onramper (me)</option>
                ) : null}

                <option value="ByBlockchain">By Blockchain</option>
                {userType == "Offramper" ? (
                    <option value="ByOfframperAddress">By Offramper Address</option>
                ) : userType == "Onramper" ? (
                    <option value="LockedByOnramper">Locked by Onramper</option>
                ) : null}

            </select>

            {(filterType === 'ByOfframperAddress' || filterType === 'LockedByOnramper') && (
                <select
                    value={selectedAddress?.address || ''}
                    onChange={handleAddressChange}
                    className="w-full px-3 py-2 border-gray-500 bg-gray-600 border rounded focus:outline-none focus:ring-2 focus:ring-blue-900"
                >
                    <option value=''>Select Address</option>
                    {user?.addresses.map((addr, index) => (
                        <option key={index} value={addr.address}>
                            {truncate(addr.address, 10, 10)} ({Object.keys(addr.address_type)[0]})
                        </option>
                    ))}
                </select>
            )}

            {filterType === 'ByBlockchain' && (
                <select
                    value={blockchainType || ""}
                    onChange={(e) => setBlockchainType(e.target.value !== "" ? e.target.value as BlockchainTypes : null)}
                    className="w-full px-3 py-2 border-gray-500 bg-gray-600 border rounded focus:outline-none focus:ring-2 focus:ring-blue-900"
                >
                    <option value="">Select Blockchain</option>
                    <option value="EVM">EVM</option>
                    <option value="ICP">ICP</option>
                    <option value="Solana">Solana</option>
                </select>
            )}

            {blockchainType === 'EVM' && (
                <select
                    value={(selectedBlockchain && 'EVM' in selectedBlockchain) ? Number(selectedBlockchain.EVM.chain_id) : ''}
                    onChange={handleChainIdChange}
                    className="w-full px-3 py-2 border-gray-500 bg-gray-600 border rounded focus:outline-none focus:ring-2 focus:ring-blue-900"
                >
                    <option value="">Select Chain</option>
                    {Object.keys(NetworkIds).map(networkId => {
                        const network = NetworkIds[networkId as keyof typeof NetworkIds];
                        return (
                            <option key={networkId} value={network!.id}>
                                {network!.name}
                            </option>
                        )
                    })}
                </select>
            )}
            {blockchainType === 'ICP' && (
                <select
                    value={(selectedBlockchain && 'ICP' in selectedBlockchain) ? selectedBlockchain.ICP.ledger_principal.toString() : ''}
                    onChange={handleCanisterChange}
                    className="w-full px-3 py-2 border-gray-500 bg-gray-600 border rounded focus:outline-none focus:ring-2 focus:ring-blue-900"
                >
                    <option value="">Select ICP Token</option>
                    {ICP_TOKENS.map(token => (
                        <option key={token.address} value={token.address}>
                            {token.name}
                        </option>
                    ))}
                </select>
            )}
        </div>
    );
}

export default OrderFilters;
