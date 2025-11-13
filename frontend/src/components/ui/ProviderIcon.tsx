import { PaymentProviderTypes } from '@/model/types';
import payPalLogo from '@/assets/logos/paypal-logo.png';
import stripeLogo from '@/assets/logos/stripe-logo.png';
import revolutLogo from '@/assets/logos/revolut-logo.jpg';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faCreditCard } from '@fortawesome/free-solid-svg-icons';
import ethereumLogo from "@/assets/blockchains/ethereum-logo.png";
import solanaLogo from "@/assets/blockchains/solana-logo.png";
import icpLogo from "@/assets/blockchains/icp-logo.svg";
import { NetworkIds } from '@/constants/networks';

export const ProviderIcon = ({
    type,
    className = "h-5 rounded-md w-auto",
    crypto,
    evmChain,
}: { type: PaymentProviderTypes; className?: string; crypto?: 'EVM' | 'Solana' | 'ICP', evmChain?: number }) => {
    if (type === "PayPal") return <img src={payPalLogo} alt="PayPal" className={className} />;
    if (type === "Stripe") return <img src={stripeLogo} alt="Stripe" className={className} />;
    if (type === "Revolut") return <img src={revolutLogo} alt="Revolut" className={className} />;
    if (type === "Email") return <FontAwesomeIcon icon={faCreditCard} size="lg" />
    if (type === "Crypto") {
        if (crypto === 'EVM') {
            let logo = ethereumLogo; let alt = 'EVM';
            if (evmChain !== undefined) {
                const chain = Object.values(NetworkIds).find((n) => n.id === evmChain);
                if (chain) { logo = chain.logo; alt = chain.name };
            }
            return <img src={logo} alt={alt} className={className} />
        } else {
            return crypto === 'Solana' ? <img src={solanaLogo} alt="Solana" className={className} />
                : crypto === 'ICP' ? <img src={icpLogo} alt="ICP" className={className} /> : null
        }
    }
    return null;
};
