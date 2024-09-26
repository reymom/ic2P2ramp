use candid::CandidType;

use crate::evm::rpc::TransactionReceipt;

use super::request::FailedSignRequest;

#[derive(CandidType, Debug, Clone)]
pub enum TransactionAction {
    Commit,
    Uncommit,
    Cancel,
    Release,
}

#[derive(CandidType, Debug, Clone)]
pub struct EvmTransactionLog {
    pub order_id: u64,
    pub action: TransactionAction,
    pub status: TransactionStatus,
}

#[derive(Debug, Clone, CandidType)]
pub enum TransactionStatus {
    Confirmed(TransactionReceipt),
    Failed(String),
    Pending,
    Unresolved(String, FailedSignRequest),
}
