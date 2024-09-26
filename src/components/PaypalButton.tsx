import React, { useEffect, useRef, useState } from 'react';

interface PayPalButtonProps {
    orderId: string
    amount: Number;
    currency: string;
    paypalId: string;
    onSuccess: (transactionId: string) => void;
    disabled: boolean;
}

declare global {
    interface Window {
        paypal: any;
    }
}

const PayPalButton: React.FC<PayPalButtonProps> = ({ orderId, amount, currency, paypalId, onSuccess, disabled }) => {
    const paypalRef = useRef<HTMLDivElement>(null);
    const [paypalScriptLoaded, setPaypalScriptLoaded] = useState(false);

    useEffect(() => {
        const clientId = process.env.FRONTEND_PAYPAL_CLIENT_ID;
        if (!clientId || !paypalId) return;

        const scriptId = 'paypal-sdk';
        if (!document.getElementById(scriptId)) {
            const script = document.createElement('script');
            script.id = scriptId;
            script.src = `https://www.paypal.com/sdk/js?client-id=${clientId}&currency=${currency}`;
            script.async = true;
            script.onload = () => {
                console.log('PayPal SDK script loaded');
                setPaypalScriptLoaded(true);
            };
            script.onerror = () => {
                console.error('Failed to load PayPal SDK script');
            };
            document.head.appendChild(script);
        } else {
            setPaypalScriptLoaded(true);
        }
    }, [paypalId, currency]);

    useEffect(() => {
        if (!paypalRef.current || !paypalScriptLoaded) return;

        if (window.paypal) {
            renderPayPalButtons();
        } else {
            const checkPaypalLoaded = setInterval(() => {
                if (window.paypal) {
                    clearInterval(checkPaypalLoaded);
                    renderPayPalButtons();
                }
            }, 500);
        }

        // Cleanup to remove PayPal button if component is unmounted
        return () => {
            if (paypalRef.current) {
                paypalRef.current.innerHTML = "";
            }
        };
    }, [paypalScriptLoaded, orderId, amount, currency, paypalId]);

    const renderPayPalButtons = () => {
        try {
            if (!paypalRef.current) return;

            window.paypal.Buttons({
                fundingSource: window.paypal.FUNDING.PAYPAL,
                createOrder: (data: any, actions: any) => {
                    return actions.order.create({
                        purchase_units: [{
                            amount: {
                                value: amount.toString(),
                                currency_code: currency
                            },
                            payee: {
                                email_address: paypalId
                            },
                        }]
                    });
                },
                onApprove: async (data: any, actions: any) => {
                    const details = await actions.order.capture();
                    console.log("details = ", details);

                    const orderId = details.id;
                    onSuccess(orderId);
                },
                onError: (err: any) => {
                    console.error('PayPal Checkout onError', err);
                }
            }).render(paypalRef.current);
        } catch (err) {
            console.error("PayPal button render failed:", err);
        }
    };


    return (
        <div className="mt-4">
            {paypalId ? (
                <div
                    ref={paypalRef}
                    id={`paypal-button-container-${orderId}`}
                    className={`flex justify-center ${disabled ? 'pointer-events-none opacity-50' : ''}`}>
                </div>
            ) : (
                <div className="text-red-500 text-center">Please enter a PayPal ID to proceed with the payment</div>
            )}
        </div >
    );
}

export default PayPalButton;
