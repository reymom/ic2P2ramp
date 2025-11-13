import React, { useEffect, useState } from 'react';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import clsx from 'clsx';

import { OrderState, PaymentProvider } from '@/declarations/icramp_backend/icramp_backend.did';
import { CURRENCY_ICON_MAP } from '@/constants/currencyIconsMap';
import { blockchainAssetToBlockchainType, paymentProviderTypeToString, providerToProviderType } from '@/model/helpers/types';
import { formatCryptoUnits, formatPrice, formatTimeLeft, truncate } from '@/utils/formatters';
import { Balance, useUser } from '@/components/user/UserContext';
import PayPalButton from '@/components/ui/PaypalButton';
import DynamicDots from '@/components/ui/DynamicDots';
import { useOrderLogic } from './hooks/useOrderLogic';
import { ProviderIcon } from '../ui/ProviderIcon';
import { sameCryptoAsset } from '@/utils/cryptoProviders';
import { NetworkIds, NetworkProps } from '@/constants/networks';
import { getEvmTokens } from '@/constants/evm_tokens';
import { fetchSolanaTokenOptions } from '@/model/blockchain/solana';
import { TokenOption } from '@/model/types';
import { ICP_TOKENS } from '@/constants/icp_tokens';

interface OrderProps {
    order: OrderState;
    refetchOrders: () => Promise<void>;
}

const OrderCard: React.FC<OrderProps> = ({ order, refetchOrders }) => {
    const [fillIndex, setFillIndex] = useState(0);
    const [fillsOpen, setFillsOpen] = useState(false);

    const {
        orderState,
        baseOrder,
        orderBlockchainAsset,
        token,
        cryptoAmount,
        currentPrice,
        isLoading,
        loadingMessage,
        message,
        txHash,
        remainingTime,
        isPayable,
        loadingPayable,
        loadingPrice,
        committedProvider,
        lockAmount,
        topUpAmount,
        topUpLoading,
        topUpTxHash,
        getStatusColors,
        getNetworkLogo,
        getNetworkName,
        getExplorerLinks,
        handleProviderSelection,
        commitToOrder,
        removeOrder,
        topUp,
        setLockAmount,
        setTopUpAmount,
        handlePayPalSuccess,
        handleRevolutRedirect,
        handleStripePay,
        handleCryptoPay,
    } = useOrderLogic(order, refetchOrders);

    const { backgroundColor, borderColor, textColor } = getStatusColors();
    const { user, userType, evmBalances, bitcoinBalance, solanaBalance, icpBalances } = useUser();

    useEffect(() => { setFillIndex(0); }, [orderState]);

    const [solOptions, setSolOptions] = useState<TokenOption[]>();
    useEffect(() => {
        fetchSolanaTokenOptions().then(setSolOptions);
    }, []);

    const tokenUnit = token && (
        <img
            src={token.logo}
            alt={token.name}
            title={token.name}
            className="h-5 w-5 inline-block border border-white bg-gray-100 rounded-full"
        />
    );
    const tokenOverlay = token && (
        <img
            src={token.logo}
            alt={token.name}
            title={token.name}
            className="h-4 w-4 rounded-full border border-white bg-gray-100 absolute -bottom-1 -right-1"
        />
    );

    const getAvailableBalance = (): Balance | null => {
        const blockchainType = blockchainAssetToBlockchainType(orderBlockchainAsset!);
        if (blockchainType === 'ICP' && token && icpBalances) {
            return icpBalances[token.name] ?? null;
        } else if (blockchainType === 'EVM' && token && evmBalances) {
            if (token.isNative) return evmBalances[token.name] ?? null;
            return evmBalances[token.address] ?? null;
        } else if (blockchainType === 'Bitcoin') {
            if (token?.runeMetadata && bitcoinBalance)
                return bitcoinBalance.runes[token.runeMetadata.id] ?? null;
            return bitcoinBalance?.balance ?? null;
        } else if (blockchainType === 'Solana') {
            if (token?.isNative) return solanaBalance?.balance ?? null;
            return token?.address
                ? (solanaBalance?.splTokens[token.address] ?? null)
                : null;
        }
        return null
    };

    const describeCryptoProvider = (provider: PaymentProvider): {
        chain: 'EVM' | 'Solana' | 'ICP' | undefined;
        evmNetwork: NetworkProps | undefined;
        token: TokenOption | undefined;
    } => {
        if (!('Crypto' in provider)) return { chain: undefined, evmNetwork: undefined, token: undefined };
        const asset = provider.Crypto.asset;

        let chain: 'EVM' | 'Solana' | 'ICP' | undefined;
        let evmNetwork: NetworkProps | undefined;
        let token: TokenOption | undefined;
        if ('EVM' in asset) {
            chain = 'EVM';
            const cid = Number(asset.EVM.chain_id);
            evmNetwork = Object.values(NetworkIds).find(n => n.id === cid);
            const tokenAddr = asset.EVM.token_address?.[0];
            if (tokenAddr) {
                token = getEvmTokens(cid).find(
                    (t) => t.address.toLowerCase() === tokenAddr.toLowerCase(),
                );

            }
        } else if ('Solana' in asset) {
            chain = 'Solana';
            const mint = asset.Solana.spl_token?.[0];
            if (mint && solOptions) {
                token = solOptions.find((t) => t.address === mint);
            }
        } else if ('ICP' in asset) {
            chain = 'ICP';
            const principalStr =
                asset.ICP.ledger_principal.toText?.() ??
                String(asset.ICP.ledger_principal);
            token = ICP_TOKENS.find((t) => t.address === principalStr);
        }

        return { chain, evmNetwork, token };
    };

    const commonOrderDiv = baseOrder && orderBlockchainAsset && (
        <div className="space-y-3">

            {/* Fiat and Crypto Amount */}
            <div className="text-lg flex justify-between">
                <span className="opacity-90">Price:</span>
                <span className="font-medium flex items-center space-x-2">
                    <span>
                        {loadingPrice ? (
                            <DynamicDots isLoading={loadingPrice} />
                        ) : currentPrice ? formatPrice(Number(currentPrice)) : ""}
                    </span>
                    <span className="border border-white bg-amber-600 rounded-full h-5 w-5 flex items-center justify-center text-sm leading-none">
                        <FontAwesomeIcon icon={CURRENCY_ICON_MAP[baseOrder!.currency]} className="text-gray-300" />
                    </span>
                </span>
            </div>
            <div className="text-lg flex justify-between">
                <span className="opacity-90">Amount:</span>
                <span className="font-medium flex items-center space-x-2" title={cryptoAmount?.fullAmount}>
                    <span>{cryptoAmount?.shortAmount}</span>
                    {tokenUnit}
                </span>
            </div>

            {/* Offramper Address */}
            <div className="text-lg flex justify-between">
                <span className="opacity-90">Address:</span>
                <span className="font-medium">
                    <a
                        href={`${getExplorerLinks(baseOrder.offramper_address.address)?.address}`}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="text-blue-800 dark:text-white hover:text-blue-700 dark:hover:text-gray-400 transition-colors duration-200"
                        title="View on Explorer"
                    >
                        {truncate(baseOrder.offramper_address.address, 4, 4)}
                    </a>
                </span>
            </div>
        </div>
    );

    return (
        <li className={`px-14 pt-10 pb-8 border rounded-xl shadow-md ${backgroundColor} ${borderColor} ${textColor} relative`}>
            {isLoading && (
                <div className="absolute inset-0 rounded-xl bg-black bg-opacity-60 flex flex-col items-center justify-center z-40">
                    <div className="w-10 h-10 border-t-4 border-b-4 border-indigo-400 rounded-full animate-spin mb-4"></div>
                    {loadingMessage && (
                        <div className="text-black dark:text-white text-2xl font-bold mt-2">
                            {loadingMessage}<DynamicDots isLoading />
                        </div>
                    )}
                </div>
            )}

            {getNetworkLogo() && (
                <div className="absolute top-2.5 left-2.5 h-8 w-8">
                    <div className="relative inline-block">
                        <img
                            src={getNetworkLogo()}
                            alt="Blockchain Logo"
                            title={getNetworkName()}
                            className="h-8 w-8"
                        />
                        {tokenOverlay}
                    </div>
                </div>
            )}
            {'Created' in orderState && (
                <div className="flex flex-col">
                    {commonOrderDiv}

                    <hr className="border-t border-gray-500 w-full my-3" />

                    {(() => {
                        const fills = orderState.Created.fills;
                        const filledSum = fills.reduce((acc, f) => acc + Number(f.crypto_amount), 0);
                        const remaining = Number(orderState.Created.crypto.amount);
                        const totalInit = filledSum + remaining;
                        const pct = totalInit > 0 ? Math.round((filledSum / totalInit) * 100) : 0;
                        const clickable = fills.length > 0;
                        return (
                            <div className="text-lg flex justify-between items-center">
                                <span className="opacity-90">Filled:</span>
                                <button
                                    type="button"
                                    onClick={() => clickable && setFillsOpen(true)}
                                    disabled={!clickable}
                                    className={clsx(
                                        "font-medium underline decoration-dotted",
                                        clickable ? "text-indigo-300 hover:text-indigo-200" : "text-gray-400 cursor-not-allowed"
                                    )}
                                    title={clickable ? "View fills" : "No fills yet"}
                                >
                                    {pct}% <span className="text-xs opacity-70">({fills.length} fill{fills.length === 1 ? "" : "s"})</span>
                                </button>
                            </div>
                        );
                    })()}

                    <hr className="border-t border-gray-500 w-full my-3" />

                    <div className="text-lg">
                        <span className="opacity-90">Payment Methods:</span>
                        <div className="mt-2 flex flex-wrap gap-2">
                            {orderState.Created.offramper_providers.map((provider, index) => {
                                const providerType = paymentProviderTypeToString(
                                    providerToProviderType(provider),
                                );

                                // chain for Crypto → icon
                                let chain: 'EVM' | 'Solana' | 'ICP' | undefined;
                                let evmNetwork: NetworkProps | undefined;
                                let token: TokenOption | undefined;
                                if ('Crypto' in provider) {
                                    const desc = describeCryptoProvider(provider);
                                    chain = desc.chain; evmNetwork = desc.evmNetwork; token = desc.token;
                                }

                                if (userType === 'Onramper') {
                                    let checked = false;
                                    if (committedProvider) {
                                        const commitedProv = committedProvider[1];
                                        const committedProvType = paymentProviderTypeToString(committedProvider[0]);
                                        if (
                                            providerType === "Crypto" &&
                                            "Crypto" in provider &&
                                            "Crypto" in commitedProv
                                        ) {
                                            checked = sameCryptoAsset(
                                                provider.Crypto.asset,
                                                commitedProv.Crypto.asset,
                                            );
                                        } else {
                                            checked =
                                                providerType === committedProvType ||
                                                (providerType === "Stripe" &&
                                                    committedProvType === "Email");
                                        }
                                    }

                                    return (
                                        <button
                                            key={index}
                                            type="button"
                                            onClick={() => handleProviderSelection(providerType, provider)}
                                            className={clsx(
                                                'inline-flex items-center gap-1 px-3 py-1 rounded-full border text-sm transition',
                                                checked
                                                    ? 'bg-indigo-600/25 border-indigo-400 text-indigo-50'
                                                    : 'bg-gray-700/40 border-gray-500/60 text-gray-200 hover:bg-indigo-500/10 hover:border-indigo-400/60',
                                            )}
                                        >
                                            <ProviderIcon
                                                type={providerType}
                                                className="h-4 w-auto rounded-md"
                                                crypto={chain ?? undefined}
                                                evmChain={evmNetwork?.id ?? undefined}
                                            />
                                            <span>{providerType}</span>
                                            {token?.logo && (
                                                <img
                                                    src={token.logo}
                                                    alt={token.logo}
                                                    className="h-5 w-auto rounded-md"
                                                />
                                            )}
                                        </button>
                                    );
                                }

                                return (
                                    <span
                                        key={index}
                                        className="inline-flex items-center gap-1 px-3 py-1 rounded-full border text-sm bg-gray-700/40 border-gray-500/60 text-gray-200 cursor-default"
                                    >
                                        <ProviderIcon
                                            type={providerType}
                                            className="h-4 w-auto rounded-md"
                                            crypto={chain ?? undefined}
                                            evmChain={evmNetwork?.id ?? undefined}
                                        />
                                        <span>{providerType}</span>
                                        {token?.logo && (
                                            <img
                                                src={token.logo}
                                                alt={token.logo}
                                                className="h-5 w-auto rounded-md"
                                            />
                                        )}
                                    </span>
                                );
                            })}
                        </div>
                    </div>

                    {/* 🔹 Lock Amount input */}
                    {user && userType === 'Onramper' && (() => {
                        const dec = token?.decimals ?? 8;
                        const maxLockHuman = Number(orderState.Created.crypto.amount) / Math.pow(10, dec);
                        const num = Number(lockAmount);
                        const lockValid = Number.isFinite(num) && num > 0 && num <= maxLockHuman;

                        const addressTypeOk = user.addresses.some(
                            (addr) =>
                                Object.keys(addr.address_type)[0] ===
                                Object.keys(orderState.Created.offramper_address.address_type)[0]
                        );
                        const disabledBase = !committedProvider || !addressTypeOk;

                        const setPct = (p: number) =>
                            setLockAmount((maxLockHuman * p).toFixed(Math.min(6, dec)));

                        return (
                            <>
                                <hr className="border-t border-gray-500 w-full my-3" />
                                <div className="relative mb-2">
                                    <input
                                        name="lock_amount"
                                        type="number"
                                        min="0"
                                        step="any"
                                        value={lockAmount}
                                        onChange={(e) => setLockAmount(e.target.value)}
                                        placeholder={"0.00"}
                                        className={clsx(
                                            "w-full h-11 rounded-2xl bg-white/5 border pl-4 pr-24",
                                            "text-base placeholder-white/40 outline-none",
                                            "focus:ring-2 focus:ring-indigo-500/70 focus:border-transparent transition",
                                            !lockAmount || lockValid ? "border-white/10" : "border-red-500 focus:ring-red-500"
                                        )}
                                    />
                                    {/* 🔹 Quick % buttons */}
                                    <div className="absolute right-2 top-1 flex gap-2">
                                        {[0.25, 0.5, 0.75, 1].map((p) => (
                                            <button
                                                key={p}
                                                type="button"
                                                onClick={() => setPct(p)}
                                                className="px-2 py-1 rounded-md text-[10px] bg-white/10 hover:bg-white/20"
                                                title={`Set ${p * 100}%`}
                                            >
                                                {Math.round(p * 100)}%
                                            </button>
                                        ))}
                                    </div>
                                </div>

                                {!lockValid && lockAmount && (
                                    <div className="text-xs text-red-400 mb-2">Enter an amount &gt; 0 and ≤ max.</div>
                                )}

                                {/* Commit Button for Onramper */}
                                <button
                                    onClick={() => lockValid && commitToOrder(committedProvider![1])}
                                    className={clsx(
                                        "mt-1 px-4 py-2 rounded-md w-full font-medium flex items-center justify-center",
                                        disabledBase || !lockValid || isLoading
                                            ? "bg-gray-500 cursor-not-allowed"
                                            : "bg-green-700 hover:bg-green-800"
                                    )}
                                    disabled={disabledBase || !lockValid || isLoading}
                                >
                                    {isLoading ? (
                                        <>
                                            <div className="mr-2 w-4 h-4 border-t-2 border-b-2 border-white rounded-full animate-spin"></div>
                                            <span>Locking<DynamicDots isLoading /></span>
                                        </>
                                    ) : (
                                        <span>Lock Order (30m)</span>
                                    )}
                                </button>
                            </>
                        );
                    })()}

                    {/* Top up (only Offramper & owner) */}
                    {user && userType === 'Offramper' && orderState.Created.offramper_user_id === user.id &&
                        <div className="mt-4">
                            <div className="relative">
                                <div className="relative flex-grow justify-between items-center mb-4">
                                    <input
                                        type="number"
                                        min="0"
                                        step="any"
                                        value={topUpAmount}
                                        onChange={(e) => setTopUpAmount(Number(e.target.value))}
                                        placeholder="0.00"
                                        className={clsx(
                                            "w-full h-11 rounded-2xl bg-white/5 border pl-4 pr-28",
                                            "text-base placeholder-white/40 outline-none",
                                            "focus:ring-2 focus:ring-indigo-500/70 focus:border-transparent transition",
                                            topUpAmount && getAvailableBalance() && topUpAmount > Number(getAvailableBalance()!.formatted) ? "border-red-500" : "border-white/10",
                                            topUpAmount && getAvailableBalance() && topUpAmount > Number(getAvailableBalance()!.formatted) ? 'focus:ring-red-500' : "focus:border-blue-200",
                                        )}
                                        disabled={topUpLoading}
                                    />
                                    <span className="absolute right-1/2 top-1/2 -translate-y-1/2 translate-x-1/2 z-6">
                                        <button
                                            type="button"
                                            onClick={() => setTopUpAmount(Number(getAvailableBalance()!.formatted))}
                                            className="px-3 rounded-lg text-[10px] font-semibold text-gray-200 bg-gray-500 hover:bg-gray-400"
                                            title="Lock full amount"
                                        >
                                            max: {getAvailableBalance() ? getAvailableBalance()!.formatted : "0.00"} {token?.name}
                                        </button>
                                    </span>
                                </div>

                                <button
                                    onClick={topUp}
                                    disabled={topUpLoading || topUpAmount <= 0}
                                    className={clsx("absolute right-1 top-1 bottom-1 px-4 rounded-xl font-semibold",
                                        "bg-indigo-600 hover:bg-indigo-500 active:bg-indigo-600",
                                        "disabled:bg-slate-600 disabled:cursor-not-allowed",
                                        "shadow-lg shadow-indigo-900/30 transition")}
                                >
                                    {topUpLoading ? (
                                        <span className="inline-flex items-center gap-2">
                                            <span className="w-4 h-4 border-2 border-white/40 border-t-transparent rounded-full animate-spin" />
                                            Processing
                                        </span>
                                    ) : (
                                        'Top up'
                                    )}
                                </button>
                            </div>

                            {topUpTxHash && (
                                <a
                                    href={`${getExplorerLinks('', topUpTxHash)?.transaction}`}
                                    target="_blank"
                                    rel="noreferrer"
                                    className="mt-2 inline-block text-xs text-indigo-300 hover:text-indigo-200 underline decoration-dotted"
                                >
                                    View transaction {truncate(topUpTxHash, 5, 5)}
                                </a>
                            )}
                        </div>
                    }

                    {/* Remove Button for Offramper */}
                    {user && userType === 'Offramper' && orderState.Created.offramper_user_id === user.id && (
                        <button
                            onClick={removeOrder}
                            disabled={isLoading}
                            className="mt-3 px-4 py-2 bg-red-700 rounded-md w-full font-medium hover:bg-red-800 flex justify-center items-center"
                        >
                            {isLoading ? (
                                <>
                                    <div className="mr-2 w-4 h-4 border-t-2 border-b-2 border-white rounded-full animate-spin"></div>
                                    Removing<DynamicDots isLoading />
                                </>
                            ) : (
                                "Remove"
                            )}
                        </button>
                    )}

                    {fillsOpen && (
                        <div className="fixed inset-0 z-50 flex items-center justify-center">
                            {/* backdrop */}
                            <div className="absolute inset-0 bg-black/60" onClick={() => setFillsOpen(false)} />
                            {/* dialog */}
                            <div className="relative z-10 w-full max-w-lg rounded-2xl border border-white/10 bg-gray-900 p-4 shadow-2xl">
                                <div className="flex items-center justify-between mb-2">
                                    <h3 className="text-base font-semibold">Fills ({orderState.Created.fills.length})</h3>
                                    <button
                                        onClick={() => setFillsOpen(false)}
                                        className="text-sm px-2 py-1 rounded-md bg-white/10 hover:bg-white/20"
                                        aria-label="Close"
                                    >
                                        Close
                                    </button>
                                </div>

                                {orderState.Created.fills.length === 0 ? (
                                    <div className="text-sm opacity-80">No fills yet.</div>
                                ) : (() => {
                                    const fills = orderState.Created.fills;
                                    const total = fills.length;
                                    const idx = Math.min(fillIndex, Math.max(0, total - 1));
                                    const f = fills[idx];
                                    const dec = token?.decimals ?? 8;
                                    const amt = Number(f.crypto_amount) / Math.pow(10, dec);
                                    const fee = Number(f.crypto_fee) / Math.pow(10, dec);
                                    const txLink = f.tx_id.length ? getExplorerLinks('', f.tx_id[0])?.transaction : undefined;
                                    const provider =
                                        'PayPal' in f.provider ? 'PayPal' :
                                            'Revolut' in f.provider ? 'Revolut' :
                                                'Email' in f.provider ? 'Stripe' : 'Other';
                                    return (
                                        <>
                                            <div className="rounded-xl border border-white/10 p-3 bg-white/5">
                                                <div className="mt-1 grid grid-cols-2 gap-2 text-sm">
                                                    <div>
                                                        <span className="opacity-70">Payer:</span>
                                                        <span className="font-medium"> <a
                                                            href={getExplorerLinks(f.payer.address)?.address}
                                                            target="_blank"
                                                            rel="noreferrer"
                                                            className="underline decoration-dotted"
                                                        >
                                                            {truncate(f.payer.address, 4, 4)}
                                                        </a></span>
                                                    </div>
                                                    <div><span className="opacity-70">Provider:</span> <span className="font-medium">{provider}</span></div>
                                                    <div><span className="opacity-70">Fiat:</span> <span className="font-medium">{formatPrice(Number(f.fiat))}</span></div>
                                                    <div><span className="opacity-70">Offramper Fee:</span> <span className="font-medium">{formatPrice(Number(f.offramper_fee))}</span></div>
                                                    <div><span className="opacity-70">Crypto:</span> <span className="font-medium">{amt.toFixed(Math.min(6, dec))} {tokenUnit}</span></div>
                                                    <div><span className="opacity-70">Crypto Fee:</span> <span className="font-medium">{fee.toFixed(Math.min(6, dec))} {tokenUnit}</span></div>
                                                </div>
                                                {txLink && f.tx_id.length && (
                                                    <a
                                                        href={txLink}
                                                        target="_blank"
                                                        rel="noreferrer"
                                                        className="mt-1 inline-block text-xs text-indigo-300 hover:text-indigo-200 underline decoration-dotted"
                                                    >
                                                        View tx {truncate(f.tx_id[0], 5, 5)}
                                                    </a>
                                                )}
                                            </div>

                                            {/* Pager */}
                                            {total > 1 && (
                                                <div className="mt-3 flex items-center justify-between text-xs">
                                                    <button
                                                        onClick={() => setFillIndex((i) => Math.max(0, i - 1))}
                                                        className="px-2 py-1 rounded-md bg-white/10 hover:bg-white/20 disabled:opacity-40"
                                                        disabled={idx === 0}
                                                    >
                                                        Prev
                                                    </button>
                                                    <span className="opacity-70">Fill {idx + 1} of {total}</span>
                                                    <button
                                                        onClick={() => setFillIndex((i) => Math.min(total - 1, i + 1))}
                                                        className="px-2 py-1 rounded-md bg-white/10 hover:bg-white/20 disabled:opacity-40"
                                                        disabled={idx >= total - 1}
                                                    >
                                                        Next
                                                    </button>
                                                </div>
                                            )}
                                        </>
                                    );
                                })()}
                            </div>
                        </div>
                    )}
                </div>
            )}
            {'Locked' in orderState && (
                <div className="flex flex-col">
                    {commonOrderDiv}

                    {/* 🔹 Locked amount row */}
                    <div className="text-lg flex justify-between mt-3">
                        <span className="opacity-90">Locked:</span>
                        <span className="font-medium flex items-center space-x-2" title={orderState.Locked.lock_amount.toString()}>
                            <span>
                                {(() => {
                                    const dec = token?.decimals ?? 8;
                                    const v = Number(orderState.Locked.lock_amount) / Math.pow(10, dec);
                                    return `${v.toFixed(Math.min(6, dec))}`;
                                })()}
                            </span>
                            {tokenUnit}
                        </span>
                    </div>

                    {user && userType === 'Onramper' && orderState.Locked.onramper.user_id === user.id && !orderState.Locked.uncommited && (
                        <>
                            <div>
                                {orderState.Locked.onramper.provider.hasOwnProperty('PayPal') ? (
                                    <PayPalButton
                                        orderId={orderState.Locked.base.id.toString()}
                                        amount={Number(orderState.Locked.price + orderState.Locked.offramper_fee) / 100.}
                                        currency={orderState.Locked.base.currency}
                                        paypalId={(() => {
                                            const provider = orderState.Locked.base.offramper_providers.find(
                                                provider => 'PayPal' in provider
                                            );
                                            if (provider && 'PayPal' in provider) {
                                                return provider.PayPal.id;
                                            }
                                            return '';
                                        })()}
                                        onSuccess={(transactionId) => handlePayPalSuccess(transactionId)}
                                        disabled={!isPayable || isLoading || orderState.Locked.payment_done}
                                    />
                                ) : orderState.Locked.onramper.provider.hasOwnProperty('Revolut') ? (
                                    <div>
                                        <button
                                            className={`mt-4 px-4 py-2 bg-blue-600 rounded-md hover:bg-blue-700 ${isPayable ? "cursor-not-allowed" : ""}`}
                                            onClick={handleRevolutRedirect}
                                            disabled={!isPayable || isLoading}
                                        >
                                            Confirm Revolut Consent
                                        </button>
                                    </div>
                                ) : orderState.Locked.onramper.provider.hasOwnProperty('Email') ? (
                                    <div>
                                        {(() => {
                                            const stripeUrl =
                                                orderState.Locked?.payment_url?.[0] ??
                                                orderState.Locked?.payment_url;

                                            const disabled = !isPayable || isLoading || !stripeUrl;
                                            return (
                                                <button
                                                    className={`w-full mt-4 px-4 py-2 bg-purple-700 rounded-md hover:bg-purple-800 ${disabled ? "cursor-not-allowed opacity-70" : ""}`}
                                                    onClick={handleStripePay}
                                                    disabled={disabled}
                                                    title={!stripeUrl ? "Payment link not available yet" : "Pay securely by card"}
                                                >
                                                    Pay by Card (Stripe)
                                                </button>
                                            );
                                        })()}
                                    </div>
                                ) : 'Crypto' in orderState.Locked.onramper.provider ? (
                                    (() => {
                                        const provider = orderState.Locked.onramper.provider;

                                        const providerType = paymentProviderTypeToString(
                                            providerToProviderType(provider),
                                        );

                                        const { chain, evmNetwork, token } = describeCryptoProvider(provider);

                                        const chainLabel =
                                            chain === 'EVM'
                                                ? evmNetwork?.name ?? 'EVM'
                                                : chain ?? undefined;

                                        return (
                                            <div>
                                                <button
                                                    className={`w-full mt-4 px-4 py-2 bg-emerald-600 rounded-md hover:bg-emerald-700 ${!isPayable || isLoading || orderState.Locked.payment_done
                                                        ? 'cursor-not-allowed opacity-70'
                                                        : ''
                                                        }`}
                                                    onClick={handleCryptoPay}
                                                    disabled={!isPayable || isLoading || orderState.Locked.payment_done}
                                                    title="Pay with your wallet using the matching crypto provider"
                                                >
                                                    <span className="flex items-center justify-center gap-2">
                                                        <span className="inline-flex items-center gap-1">
                                                            <ProviderIcon
                                                                type={providerType}
                                                                className="h-4 w-auto rounded-md"
                                                                crypto={chain ?? undefined}
                                                                evmChain={evmNetwork?.id ?? undefined}
                                                            />
                                                            <span>
                                                                Pay with Crypto
                                                                {chain === 'EVM' && evmNetwork
                                                                    ? ` (${evmNetwork.name})`
                                                                    : ''}
                                                            </span>
                                                            <span>
                                                                {token?.logo && (
                                                                    <img
                                                                        src={token.logo}
                                                                        alt={token.logo}
                                                                        className="h-5 w-auto rounded-md"
                                                                    />
                                                                )}
                                                            </span>
                                                        </span>
                                                    </span>
                                                </button>

                                                {/* optional: show destination (offramper) address for visual verification */}
                                                <p className="mt-1 text-xs opacity-80 break-words">
                                                    Destination:{' '}
                                                    {
                                                        (
                                                            orderState.Locked.base.offramper_providers.find(
                                                                (p) =>
                                                                    'Crypto' in p &&
                                                                    sameCryptoAsset(
                                                                        provider.Crypto.asset,
                                                                        p.Crypto.asset,
                                                                    ),
                                                            ) as any
                                                        )?.Crypto?.address?.address
                                                    }
                                                </p>
                                            </div>
                                        );
                                    })()
                                ) : null}
                            </div>
                            <div className="text-red-500 mt-2">
                                {!orderState.Locked.payment_done && !loadingPayable && !isPayable && (
                                    "This order cannot be paid at the moment. Please contact support or try again later."
                                )}
                                {orderState.Locked.payment_done && !isLoading && (
                                    "Payment is validated but couldn't release your funds. Please contact support to solve this issue."
                                )}
                            </div>
                        </>
                    )}

                    {remainingTime !== null && (
                        <div className="text-sm text-gray-600 dark:text-gray-200 mt-2">
                            (Locked for {formatTimeLeft(remainingTime)})
                        </div>
                    )}
                </div>
            )}
            {'Completed' in orderState && (
                <div className="flex flex-col space-y-3">
                    {/* Totals */}
                    <div className="text-lg flex justify-between">
                        <span className="opacity-90">Fiat:</span>
                        <span className="font-medium">{formatPrice(Number(orderState.Completed.total_fiat))}</span>
                    </div>
                    <div className="text-lg flex justify-between">
                        <span className="opacity-90">Offramper Fee:</span>
                        <span className="font-medium">{formatPrice(Number(orderState.Completed.total_offramper_fee))}</span>
                    </div>
                    <div className="text-lg flex justify-between">
                        <span className="opacity-90">Crypto:</span>
                        {(() => {
                            const dec = token?.decimals ?? 8;
                            const v = Number(orderState.Completed.total_crypto) / Math.pow(10, dec);
                            return <span className="font-medium flex items-center space-x-2" title={v.toString()}>
                                <span>{formatCryptoUnits(v)}</span>
                                {tokenUnit}
                            </span>
                        })()}
                    </div>
                    <div className="text-lg flex justify-between">
                        <span className="opacity-90">Crypto Fee:</span>
                        {(() => {
                            const dec = token?.decimals ?? 8;
                            const v = Number(orderState.Completed.total_crypto_fee) / Math.pow(10, dec);
                            return <span className="font-medium flex items-center space-x-2" title={v.toString()}>
                                <span>{formatCryptoUnits(v)}</span>
                                {tokenUnit}
                            </span>
                        })()}
                    </div>

                    {/* Offramper */}
                    <div className="text-lg flex justify-between">
                        <span className="opacity-90">Offramper:</span>
                        <span className="font-medium">
                            <a
                                href={`${getExplorerLinks(orderState.Completed.offramper.address)?.address}`}
                                target="_blank"
                                rel="noopener noreferrer"
                                className="text-black dark:text-white hover:text-gray-400 transition-colors duration-200"
                                title="View on Block Explorer"
                            >
                                {truncate(orderState.Completed.offramper.address, 4, 4)}
                            </a>
                        </span>
                    </div>

                    {/* Network (EVM only, keep your logo helpers) */}
                    {'EVM' in orderState.Completed.asset && (
                        <div className="text-lg flex justify-between">
                            <span className="opacity-80">Network:</span>
                            <img src={getNetworkLogo()} alt={getNetworkName()} title={getNetworkName()} className="h-5 w-5" />
                        </div>
                    )}

                    {/* Fills list */}
                    {(() => {
                        const fills = orderState.Completed.fills;
                        const total = fills.length;
                        const idx = Math.min(fillIndex, Math.max(0, total - 1));
                        const f = fills[idx];
                        const dec = token?.decimals ?? 8;
                        const amt = Number(f.crypto_amount) / Math.pow(10, dec);
                        const fee = Number(f.crypto_fee) / Math.pow(10, dec);
                        const txLink = f.tx_id.length ? getExplorerLinks('', f.tx_id[0])?.transaction : undefined;
                        const provider =
                            'PayPal' in f.provider ? 'PayPal' :
                                'Revolut' in f.provider ? 'Revolut' :
                                    'Email' in f.provider ? 'Stripe' :
                                        'Crypto' in f.provider ? 'Crypto' : 'Other';

                        return (
                            <div className="mt-3">
                                <div className="text-sm opacity-80 mb-1">Fills ({idx + 1} / {total})</div>
                                <div className="rounded-xl border border-white/10 p-3 bg-white/5">
                                    <div className="mt-1 grid grid-cols-2 gap-2 text-sm">
                                        <div>
                                            <span className="opacity-70">Payer:</span>
                                            <span className="font-medium"> <a
                                                href={getExplorerLinks(f.payer.address)?.address}
                                                target="_blank"
                                                rel="noreferrer"
                                                className="underline decoration-dotted"
                                            >
                                                {truncate(f.payer.address, 4, 4)}
                                            </a></span>
                                        </div>
                                        <div><span className="opacity-70">Provider:</span> <span className="font-medium">{provider}</span></div>
                                        <div><span className="opacity-70">Fiat:</span> <span className="font-medium">{formatPrice(Number(f.fiat))}</span></div>
                                        <div><span className="opacity-70">Offramper Fee:</span> <span className="font-medium">{formatPrice(Number(f.offramper_fee))}</span></div>
                                        <div><span className="opacity-70">Crypto:</span> <span className="font-medium">{amt.toFixed(Math.min(6, dec))} {tokenUnit}</span></div>
                                        <div><span className="opacity-70">Crypto Fee:</span> <span className="font-medium">{fee.toFixed(Math.min(6, dec))} {tokenUnit}</span></div>
                                    </div>
                                    {txLink && f.tx_id.length && (
                                        <a
                                            href={txLink}
                                            target="_blank"
                                            rel="noreferrer"
                                            className="mt-1 inline-block text-xs text-indigo-300 hover:text-indigo-200 underline decoration-dotted"
                                        >
                                            View tx {truncate(f.tx_id[0], 5, 5)}
                                        </a>
                                    )}
                                </div>

                                {/* Pager */}
                                {total > 1 && (
                                    <div className="mt-2 flex items-center justify-between text-xs">
                                        <button
                                            onClick={() => setFillIndex((i) => Math.max(0, i - 1))}
                                            className="px-2 py-1 rounded-md bg-white/10 hover:bg-white/20 disabled:opacity-40"
                                            disabled={idx === 0}
                                        >
                                            Prev
                                        </button>
                                        <span className="opacity-70">Fill {idx + 1} of {total}</span>
                                        <button
                                            onClick={() => setFillIndex((i) => Math.min(total - 1, i + 1))}
                                            className="px-2 py-1 rounded-md bg-white/10 hover:bg-white/20 disabled:opacity-40"
                                            disabled={idx >= total - 1}
                                        >
                                            Next
                                        </button>
                                    </div>
                                )}
                            </div>
                        );
                    })()}
                </div>
            )}
            {'Cancelled' in orderState && (
                <div className="flex flex-col space-y-3">
                    <div><strong>Order ID:</strong> {orderState.Cancelled.toString()}</div>
                    <div><strong>Status:</strong> Cancelled</div>
                </div>
            )}

            {!message && txHash && (
                <div className="relative mt-2 text-xs text-blue-400 z-50 flex items-center justify-center text-center">
                    <a href={`${getExplorerLinks("", txHash)?.transaction}`} target="_blank" className="hover:underline z-50">
                        View Transaction {truncate(txHash, 5, 5)}
                    </a>
                </div>
            )}

            {isLoading && ('Bitcoin' in orderBlockchainAsset!) && txHash && (
                <div className="relative mt-4 text-sm font-medium flex items-center justify-center text-center flex-col z-50">
                    <p className="mb-2">
                        Bitcoin transactions typically take 15-60+ minutes to confirm.
                    </p>
                    <p className="text-amber-500 dark:text-amber-400">
                        Your funds will be released automatically once confirmed.
                    </p>
                </div>
            )}

            {message && (
                <div className="relative mt-4 text-sm font-medium flex items-center justify-center text-center z-50">
                    <p className="text-red-600">{message}&nbsp;</p>
                    {txHash &&
                        <a href={`${getExplorerLinks("", txHash)?.transaction}`} target="_blank" className="text-red-500 hover:underline z-50">
                            View tx: {truncate(txHash, 6, 6)}
                        </a>
                    }
                </div>
            )}
        </li>
    );
}

export default OrderCard;
