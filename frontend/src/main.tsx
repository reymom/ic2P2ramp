import React from 'react';
import ReactDOM from 'react-dom/client';
import { BrowserRouter } from 'react-router-dom';
import { RainbowKitProvider } from '@rainbow-me/rainbowkit';
import { WagmiProvider } from 'wagmi';
import {
    QueryClientProvider,
    QueryClient,
} from "@tanstack/react-query";

import App from './App';
import PageTitleUpdater from './components/PageTitleUpdater';
import { UserProvider } from './components/user/UserContext';
import { SolanaProvider } from './components/SolanaProvider';
import { SOLANA_RPC_URL } from './model/blockchain/solana';

import './index.css';
import process from 'process';
import { Buffer } from 'buffer';
import { config } from './wagmi';

declare global {
    interface Window {
        Telegram?: any;
        unisat?: any;
    }
}

(window as any).global ||= window;
(window as any).process ||= process;
(window as any).Buffer ||= Buffer;


if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
    document.documentElement.classList.add('dark')
}


const queryClient = new QueryClient();

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
    <React.StrictMode>
        <WagmiProvider config={config}>
            <QueryClientProvider client={queryClient}>
                <RainbowKitProvider>
                    <SolanaProvider rpcUrl={SOLANA_RPC_URL}>
                        <UserProvider>
                            <BrowserRouter>
                                <PageTitleUpdater />
                                <App />
                            </BrowserRouter>
                        </UserProvider>
                    </SolanaProvider>
                </RainbowKitProvider>
            </QueryClientProvider>
        </WagmiProvider>
    </React.StrictMode>,
);
