use candid::CandidType;
use serde::{Deserialize, Serialize};
use sol_rpc_types::TransactionStatusMeta;

/// mostly for checking confirmation
#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct TxInfo {
    pub signature: String,
    pub found: bool,
    pub confirmed: bool, // true if confirmation_status is Confirmed/Finalized
    pub confirmations: Option<u64>,
    pub confirmation_status: Option<String>, // "processed" | "confirmed" | "finalized" (stringified)
    pub slot: Option<u64>,
    pub err: Option<String>,
}

/// to check transaction metadata
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct TxMetadata {
    pub signature: String,
    pub slot: u64,
    pub meta: TransactionStatusMeta,

    /// Message account keys used to index pre/post balances:
    /// static_account_keys() || loaded_addresses.writable || loaded_addresses.readonly
    pub account_keys: Vec<String>,
}
