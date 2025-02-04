import { Balance, useUser } from "./UserContext";
import icpLogo from "@/assets/blockchains/icp-logo.svg";
import ethereumLogo from "@/assets/blockchains/ethereum-logo.png";
import bitcoinLogo from '@/assets/blockchains/bitcoin-logo.svg';

const BalancesDashboard: React.FC = () => {
    const { user, bitcoinAddress, icpBalances, evmBalances, bitcoinBalance, refetchUser } = useUser();

    const renderBalances = (balances: { [key: string]: Balance | any }, logo: string, title: string) => (
        <>
            <h3 className="text-lg font-semibold mb-2 flex items-center">
                <img src={logo} alt={title} className="h-6 w-6 mr-2" /> {title}
            </h3>
            <table className="w-full border-collapse border border-gray-500">
                <thead>
                    <tr>
                        <th className="border border-gray-500 px-4 py-2">Token</th>
                        <th className="border border-gray-500 px-4 py-2">Balance</th>
                    </tr>
                </thead>
                <tbody>
                    {Object.entries(balances).map(([key, balance]) => (
                        <tr key={key}>
                            <td className="border border-gray-500 px-4 py-2 text-center">
                                <img src={balance.logo} alt={key} className="h-6 w-6 mx-auto" />
                            </td>
                            <td className="border border-gray-500 px-4 py-2">{balance.formatted}</td>
                        </tr>
                    ))}
                </tbody>
            </table>
        </>
    );

    return (
        <div className="p-4">
            <h2 className="text-2xl font-semibold mb-4">Balances</h2>
            <div className="space-y-6">
                {bitcoinBalance && renderBalances({ BTC: bitcoinBalance, ...bitcoinBalance.runes }, bitcoinLogo, 'Bitcoin')}
                {icpBalances && renderBalances(icpBalances, icpLogo, 'ICP')}
                {evmBalances && renderBalances(evmBalances, ethereumLogo, 'EVM')}
            </div>
        </div>
    );
};

export default BalancesDashboard;