use candid::CandidType;

use crate::{evm::rpc::TransactionReceipt, model::errors::RampError};

use super::{request::SignRequestCandid, transaction::TransactionAction};

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
