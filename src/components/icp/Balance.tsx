import React, { useState, useEffect } from 'react';
import { LedgerCanister, AccountIdentifier } from '@dfinity/ledger-icp';
import { useUser } from '../../UserContext';
import { Principal } from '@dfinity/principal';

const ICP_LEDGER_CANISTER_ID = Principal.fromText('ryjl3-tyaaa-aaaaa-aaaba-cai');

interface GetBalanceComponentProps {
    principal: string;
}

const GetBalanceComponent: React.FC<GetBalanceComponentProps> = ({ principal }) => {
    const [balance, setBalance] = useState<bigint>();
    const [error, setError] = useState(null);

    const { icpAgent } = useUser();

    useEffect(() => {
        const fetchBalance = async () => {
            try {
                const ledger = LedgerCanister.create({ agent: icpAgent!, canisterId: ICP_LEDGER_CANISTER_ID });

                const accountIdentifier = AccountIdentifier.fromPrincipal({
                    principal: Principal.fromText(principal)
                });

                const balance = await ledger.accountBalance({
                    accountIdentifier: accountIdentifier,
                    certified: true
                });

                setBalance(balance);
            } catch (err: any) {
                console.error('Failed to fetch balance:', err);
                setError(err.message);
            }
        };

        if (icpAgent) {
            fetchBalance();
        }
    }, [principal, icpAgent]);

    return (
        <div>
            {error && <div className="text-red-500">Error: {error}</div>}
            {balance !== null ? (
                <div className="text-green-500">Balance: {balance ? balance!.toString() : 0} tICP</div>
            ) : (
                <div>Loading...</div>
            )}
        </div>
    );
};

export default GetBalanceComponent;
