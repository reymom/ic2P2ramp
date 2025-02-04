import { NetworkIds, NetworkProps } from "@/constants/networks";
import icpLogo from "@/assets/blockchains/icp-logo.svg";
import bitcoinLogo from "@/assets/blockchains/bitcoin-logo.svg";

export const getNetwork = (orderBlockchain: any): NetworkProps | undefined => {
    if (!orderBlockchain) return undefined;
    return 'EVM' in orderBlockchain
        ? Object.values(NetworkIds).find(network => network.id === Number(orderBlockchain.EVM.chain_id))
        : undefined;
};

export const getNetworkLogo = (orderBlockchain: any): string | undefined => {
    if (!orderBlockchain) return undefined;
    if ('EVM' in orderBlockchain) return getNetwork(orderBlockchain)?.logo;
    if ('ICP' in orderBlockchain) return icpLogo;
    if ('Bitcoin' in orderBlockchain) return bitcoinLogo;
    return undefined;
};

export const getNetworkName = (orderBlockchain: any): string | undefined => {
    if (!orderBlockchain) return undefined;
    if ('EVM' in orderBlockchain) return getNetwork(orderBlockchain)?.name;
    if ('ICP' in orderBlockchain) return "ICP";
    if ('Bitcoin' in orderBlockchain) return "Bitcoin";
    return undefined;
};
