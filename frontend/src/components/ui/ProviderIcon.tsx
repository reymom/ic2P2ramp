import { PaymentProviderTypes } from '@/model/types';
import payPalLogo from '@/assets/logos/paypal-logo.png';
import stripeLogo from '@/assets/logos/stripe-logo.png';
import revolutLogo from '@/assets/logos/revolut-logo.jpg';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faCreditCard } from '@fortawesome/free-solid-svg-icons';

export const ProviderIcon = ({ type, className = "h-5 rounded-md w-auto" }: { type: PaymentProviderTypes; className?: string }) => {
    if (type === "PayPal") return <img src={payPalLogo} alt="PayPal" className={className} />;
    if (type === "Stripe") return <img src={stripeLogo} alt="Stripe" className={className} />;
    if (type === "Revolut") return <img src={revolutLogo} alt="Revolut" className={className} />;
    if (type === "Email") return <FontAwesomeIcon icon={faCreditCard} size="lg" />
    return null;
};
