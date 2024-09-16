use candid::CandidType;

use crate::evm::transaction::TransactionStatus;

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
