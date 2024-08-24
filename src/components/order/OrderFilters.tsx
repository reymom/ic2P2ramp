import React, { useEffect, useState } from 'react';
import { TransactionAddress, OrderFilter, Blockchain } from '../../declarations/backend/backend.did';
import { stringToOrderFilter, stringToOrderStateFilter } from '../../model/utils';
import { BlockchainTypes, OrderFilterTypes, OrderStateFilterTypes } from '../../model/types';
import { useUser } from '../user/UserContext';
import { getIcpTokenOptions } from '../../constants/tokens';
import { Principal } from '@dfinity/principal';
import { NetworkIds } from '../../constants/networks';
import { truncate } from '../../model/helper';

interface OrderFiltersProps {
    setFilter: (filter: OrderFilter | null) => void;
}

const OrderFilters: React.FC<OrderFiltersProps> = ({ setFilter }) => {
    const [filterType, setFilterType] = useState<OrderFilterTypes | null>(null);

    const [selectedAddress, setSelectedAddress] = useState<TransactionAddress | null>(null);
    const [selectedBlockchain, setSelectedBlockchain] = useState<Blockchain | null>(null);
    const [blockchainType, setBlockchainType] = useState<BlockchainTypes | null>(null);

    const { user, userType } = useUser();

    useEffect(() => {
        constructFilter();
    }, [filterType, selectedBlockchain, selectedAddress])

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
                if (selectedAddress) {
                    setFilter(stringToOrderFilter(filterCategory, selectedAddress));
                }
                break;
            case "ByBlockchain":
                if (selectedBlockchain) {
                    setFilter(stringToOrderFilter(filterCategory, selectedBlockchain));
                }
                break;
            case "ByOfframperId": case "ByOnramperId":
                if (user) {
                    setFilter(stringToOrderFilter(filterCategory, user.id))
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

        if (value === "all") {
            setFilterType(null);
            setFilter(null);
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
        <div className="mb-4">
            <select
                value={filterType || 'all'}
                onChange={handleFilterTypeChange}
                className="block w-full px-3 py-2 border rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
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
                    className="block w-full px-3 py-2 border rounded focus:outline-none focus:ring-2 focus:ring-blue-500 mt-2"
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
                    className="block w-full px-3 py-2 border rounded focus:outline-none focus:ring-2 focus:ring-blue-500 mt-2"
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
                    className="block w-full px-3 py-2 border rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                >
                    <option value="">Select Chain ID</option>
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
                    className="block w-full px-3 py-2 border rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                >
                    <option value="">Select ICP Token</option>
                    {getIcpTokenOptions().map(token => (
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
