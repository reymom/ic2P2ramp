import React, { useState, useEffect, useRef } from 'react';
import { LedgerCanister, AccountIdentifier } from '@dfinity/ledger-icp';
import { useUser } from '../../UserContext';
import { Principal } from '@dfinity/principal';
import { tokenCanisters } from '../../constants/addresses';

interface GetBalanceComponentProps {
    principal: string;
}

const GetBalanceComponent: React.FC<GetBalanceComponentProps> = ({ principal }) => {
    const [balance, setBalance] = useState<string | null>(null);
    const [error, setError] = useState(null);

    const { icpAgent } = useUser();

    const balanceCache = useRef<{ principal: string, balance: string } | null>(null);

    useEffect(() => {
        const fetchBalance = async () => {
            if (!icpAgent || !principal) return;

            if (balanceCache.current?.principal === principal) {
                setBalance(balanceCache.current.balance);
                return;
            }

            try {
                const ledgerTokenCanister = Principal.fromText(tokenCanisters.ICP);
                const ledger = LedgerCanister.create({ agent: icpAgent!, canisterId: ledgerTokenCanister });

                const accountIdentifier = AccountIdentifier.fromPrincipal({
                    principal: Principal.fromText(principal)
                });

                const balanceResult = await ledger.accountBalance({
                    accountIdentifier: accountIdentifier,
                    certified: true
                });

                const balanceFloat = Number(balanceResult) / 100_000_000;
                const balanceString = balanceFloat.toFixed(4);

                balanceCache.current = { principal, balance: balanceString };
                setBalance(balanceString);
                setError(null)
            } catch (err: any) {
                console.error('Failed to fetch balance:', err);
                setError(err.message);
                setBalance(null)
            }
        };

        setBalance(null);
        setError(null);
        fetchBalance();
    }, [principal, icpAgent]);

    return (
        <div>
            {error && <div className="text-red-500">Error: {error}</div>}
            {balance !== null ? (
                <div className="text-green-500">{balance} tICP</div>
            ) : (
                <div>Loading...</div>
            )}
        </div>
    );
};

export default GetBalanceComponent;
