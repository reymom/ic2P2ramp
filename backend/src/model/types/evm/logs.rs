use candid::CandidType;
use evm_rpc_canister_types::TransactionReceipt;

use super::{request::SignRequestCandid, transaction::TransactionAction};
use crate::model::errors::RampError;

#[derive(CandidType, Debug, Clone)]
pub struct EvmTransactionLog {
    pub order_id: u64,
    pub action: TransactionAction,
    pub status: TransactionStatus,
}

#[derive(Debug, Clone, CandidType)]
pub enum TransactionStatus {
    Broadcasting,
    Broadcasted(String, SignRequestCandid),
    BroadcastError(RampError),
    Confirmed(TransactionReceipt),
    Failed(String),
    Pending,
    Unresolved(String, SignRequestCandid),
}
