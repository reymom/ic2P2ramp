import React from 'react';
import ReactDOM from 'react-dom/client';
import { BrowserRouter } from 'react-router-dom';
import { RainbowKitProvider } from '@rainbow-me/rainbowkit';
import { WagmiProvider } from 'wagmi';
import {
    QueryClientProvider,
    QueryClient,
} from "@tanstack/react-query";

import './index.css';
import { config } from './wagmi';

import App from './App';
import { UserProvider } from './components/user/UserContext';
import PageTitleUpdater from './components/PageTitleUpdater';


declare global {
    interface Window {
        Telegram?: any;
    }
}

const queryClient = new QueryClient();

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
    <React.StrictMode>
        <WagmiProvider config={config}>
            <QueryClientProvider client={queryClient}>
                <RainbowKitProvider>
                    <UserProvider>
                        <BrowserRouter>
                            <PageTitleUpdater />
                            <App />
                        </BrowserRouter>
                    </UserProvider>
                </RainbowKitProvider>
            </QueryClientProvider>
        </WagmiProvider>
    </React.StrictMode>,
);
