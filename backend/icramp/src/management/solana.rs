use ic_cdk_timers::set_timer;
use std::time::Duration;

use crate::{
    inter_canister::solana::{solana_backend_cancel_deposit, solana_backend_get_tx},
    model::memory::stable::orders::{cancel_order, unset_processing_order},
};

const MAX_ATTEMPTS: u32 = 100;
const BASE_INTERVAL_SECS: u64 = 30;
const MAX_INTERVAL_SECS: u64 = 600;
const MIN_SOL_CONF: u64 = 2;

#[derive(Clone)]
pub enum SolanaTransactionAction {
    CancelOrder {
        order_id: u64,
        amount: u64,
        offramper: String,
        token: Option<String>,
    },
    // (add DepositFunds / CompleteOrder as you wire them to L1 transfers)
}

pub fn spawn_solana_tx_listener(signature: String, action: SolanaTransactionAction, attempt: u32) {
    if attempt >= MAX_ATTEMPTS {
        ic_cdk::println!("[solana] Max attempts reached for {signature}");
        return;
    }

    let next_attempt = attempt + 1;
    let retry = (BASE_INTERVAL_SECS * 2_u64.pow(attempt.min(10))).min(MAX_INTERVAL_SECS);

    set_timer(Duration::from_secs(retry), move || {
        ic_cdk::spawn(async move {
            match solana_backend_get_tx(signature.clone()).await {
                Ok(info) => {
                    let confs: u64 = info.confirmations.unwrap_or(u64::MAX);

                    if confs >= MIN_SOL_CONF {
                        match action.clone() {
                            SolanaTransactionAction::CancelOrder {
                                order_id,
                                offramper,
                                amount,
                                token,
                            } => {
                                match solana_backend_cancel_deposit(offramper, amount, token).await
                                {
                                    Ok(()) => match cancel_order(order_id) {
                                        Ok(()) => {
                                            let _ = unset_processing_order(&order_id);
                                            ic_cdk::println!("[solana] order canceled: {order_id}");
                                        }
                                        Err(e) => {
                                            let _ = unset_processing_order(&order_id);
                                            ic_cdk::println!(
                                                "[solana] cancel_order storage error: {:?}",
                                                e
                                            );
                                        }
                                    },
                                    Err(e) => {
                                        let _ = unset_processing_order(&order_id);
                                        ic_cdk::println!(
                                            "[solana] Error cancelling solana deposit: {:?}",
                                            e
                                        )
                                    }
                                }
                            }
                        }
                    } else {
                        // not yet confirmed → requeue
                        ic_cdk::println!(
                            "[solana] transaction not confirmed yet with signature: {:?}, status: {:?}",
                            signature,
                            info.confirmation_status.as_deref()
                        );
                        spawn_solana_tx_listener(signature, action, next_attempt);
                    }
                }
                Err(e) => {
                    ic_cdk::println!("[solana] get_tx error: {:?}", e);
                    spawn_solana_tx_listener(signature, action, next_attempt);
                }
            }
        });
    });
}
