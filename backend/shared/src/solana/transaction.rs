use candid::CandidType;
use serde::{Deserialize, Serialize};

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
