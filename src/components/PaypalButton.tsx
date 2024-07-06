import React, { useEffect } from 'react';

interface PayPalButtonProps {
    amount: bigint;
    clientId: string;
    paypalId: string;
    onSuccess: (transactionId: string) => void;
    currency: string;
}

const PayPalButton: React.FC<PayPalButtonProps> = ({ amount, clientId, paypalId, onSuccess, currency }) => {
    useEffect(() => {
        if (!paypalId) return;

        // Load PayPal script
        const script = document.createElement('script');
        script.src = `https://www.paypal.com/sdk/js?client-id=${clientId}&currency=${currency}`;
        script.addEventListener('load', () => {
            // @ts-ignore
            window.paypal.Buttons({
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

                    // const transactionId = details.id;
                    // onSuccess(transactionId);

                    const captureId = details.purchase_units[0].payments.captures[0].id;
                    console.log("captureId = ", captureId);
                    onSuccess(captureId)
                },
                onError: (err: any) => {
                    console.error('PayPal Checkout onError', err);
                }
            }).render('#paypal-button-container');
        });
        document.body.appendChild(script);
    }, [amount, onSuccess, currency]);

    return (
        <div className="mt-4">
            {paypalId ? (
                <div id="paypal-button-container" className="flex justify-center"></div>
            ) : (
                <div className="text-red-500 text-center">Please enter your PayPal ID to proceed with payment</div>
            )}
        </div>
    );
}

export default PayPalButton;
