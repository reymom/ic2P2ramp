import { FC, ReactNode, useMemo } from "react";
import { ConnectionProvider, WalletProvider } from "@solana/wallet-adapter-react";
import { WalletModalProvider } from "@solana/wallet-adapter-react-ui";
import { WalletAdapterNetwork } from "@solana/wallet-adapter-base";
import {
    PhantomWalletAdapter,
    SolflareWalletAdapter,
    LedgerWalletAdapter,
    WalletConnectWalletAdapter,
    UnsafeBurnerWalletAdapter,
} from "@solana/wallet-adapter-wallets";
import "@solana/wallet-adapter-react-ui/styles.css";
import { clusterApiUrl } from "@solana/web3.js";

const SOL_ENV = process.env.FRONTEND_SOL_ENV === "mainnet" ? "mainnet" : "devnet";
export const NETWORK: WalletAdapterNetwork = SOL_ENV === "mainnet" ? WalletAdapterNetwork.Mainnet : WalletAdapterNetwork.Devnet;

export const SolanaProvider: FC<{ children: ReactNode; rpcUrl?: string }> = ({ children, rpcUrl }) => {
    const endpoint = useMemo(() => rpcUrl || clusterApiUrl(NETWORK), [rpcUrl]);
    const wallets = useMemo(
        () => [
            new PhantomWalletAdapter(),
            new SolflareWalletAdapter({ network: NETWORK }),
            new LedgerWalletAdapter(),
            ...(process.env.FRONTEND_WALLETCONNECT_PROJECT_ID
                ? [new WalletConnectWalletAdapter({
                    network: NETWORK,
                    options: { projectId: process.env.FRONTEND_WALLETCONNECT_PROJECT_ID }
                })]
                : []),
            ...(process.env.NODE_ENV !== "production" ? [new UnsafeBurnerWalletAdapter()] : []),
        ],
        [NETWORK]
    );

    return (
        <ConnectionProvider endpoint={endpoint}>
            <WalletProvider wallets={wallets} autoConnect={false}>
                <WalletModalProvider>{children}</WalletModalProvider>
            </WalletProvider>
        </ConnectionProvider>
    );
};
