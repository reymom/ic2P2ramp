use candid::Principal;
use ic_cdk::{api, spawn};
use solana_transaction_status_client_types::TransactionConfirmationStatus;
use std::str::FromStr;
use std::time::Duration;

use sol_rpc_types::MultiRpcResult;
use solana_signature::Signature;

use crate::solana::client::client;

/// Maximum number of polling attempts before giving up
const MAX_ATTEMPTS: u32 = 10;
/// Base interval in seconds
const BASE_INTERVAL: u64 = 5;

/// After sending a TX, poll until it’s finalized. Once confirmed, you can call your on‐chain logic.
pub fn spawn_sol_tx_listener(txid: String, caller: Principal, attempt: u32) {
    if attempt >= MAX_ATTEMPTS {
        api::print(&format!("[listener] Giving up on {}", txid));
        return;
    }
    let next_attempt = attempt + 1;
    let wait = BASE_INTERVAL.saturating_mul(2u64.pow(attempt.min(5)));

    api::print(&format!("[listener] Checking {} attempt {}", txid, attempt));
    ic_cdk_timers::set_timer(Duration::from_secs(wait), move || {
        spawn(async move {
            // Build RPC client from heap‐state
            let client = client();

            // Query signature status
            let signature = Signature::from_str(&txid).expect("TX id is not valid base58");
            let statuses = client
                .get_signature_statuses(&[signature]) // ① pass signatures here
                .expect("invalid signature list") // ② unwrap the Result
                .send()
                .await;

            match statuses {
                MultiRpcResult::Consistent(Ok(result_map)) => {
                    if let Some(Some(status)) = result_map.get(0) {
                        if status.confirmation_status
                            == Some(TransactionConfirmationStatus::Finalized)
                        {
                            api::print(&format!("[listener] TX finalized: {}", txid));
                        }
                        return;
                    };
                    api::print(&format!("[listener] {} not finalized yet", txid));
                    spawn_sol_tx_listener(txid.clone(), caller.clone(), next_attempt);
                }
                MultiRpcResult::Inconsistent(results) => {
                    api::print(&format!(
                        "[listener] Inconsistent results for {}: {:?}",
                        txid, results
                    ));
                }
                _ => {
                    api::print(&format!(
                        "[listener] Error fetching status for {}, retrying",
                        txid
                    ));
                    spawn_sol_tx_listener(txid.clone(), caller.clone(), next_attempt);
                }
            }
        });
    });
}
