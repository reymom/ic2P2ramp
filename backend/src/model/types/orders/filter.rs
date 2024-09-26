use candid::{CandidType, Deserialize};

use crate::types::{Blockchain, TransactionAddress};

#[derive(CandidType, Clone, Deserialize)]
pub enum OrderFilter {
    ByOfframperId(u64),
    ByOnramperId(u64),
    ByOfframperAddress(TransactionAddress),
    LockedByOnramper(TransactionAddress),
    ByState(OrderStateFilter),
    ByBlockchain(Blockchain),
}

#[derive(CandidType, Clone, Deserialize)]
pub enum OrderStateFilter {
    Created,
    Locked,
    Completed,
    Cancelled,
}
