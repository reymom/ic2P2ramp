import React, { useEffect, useRef } from 'react';

interface PayPalButtonProps {
    orderId: string
    amount: Number;
    currency: string;
    paypalId: string;
    onSuccess: (transactionId: string) => void;
}

declare global {
    interface Window {
        paypal: any;
    }
}

const PayPalButton: React.FC<PayPalButtonProps> = ({ orderId, amount, currency, paypalId, onSuccess }) => {
    const paypalRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        if (!paypalRef.current) return;

        if (window.paypal) {
            renderPayPalButtons();
        } else {
            const checkPaypalLoaded = setInterval(() => {
                if (window.paypal) {
                    clearInterval(checkPaypalLoaded);
                    renderPayPalButtons();
                }
            }, 1000);
        }

        // Cleanup to remove PayPal button if component is unmounted
        return () => {
            if (paypalRef.current) {
                paypalRef.current.innerHTML = "";
            }
        };
    }, [orderId, amount, currency, paypalId]);

    const renderPayPalButtons = () => {
        try {
            if (!paypalRef.current) {
                console.error('PayPal button container not found');
                return;
            }

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
            {paypalId !== "" ? (
                <div ref={paypalRef} id={`paypal-button-container-${orderId}`} className="flex justify-center"></div>
            ) : (
                <div className="text-red-500 text-center">Please enter a PayPal ID to proceed with the payment</div>
            )
            }
        </div >
    );
}

export default PayPalButton;
