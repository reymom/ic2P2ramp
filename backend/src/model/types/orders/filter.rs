use candid::{CandidType, Deserialize};

use crate::{
    model::types::blockchain::BlockchainType,
    types::{BlockchainAsset, TransactionAddress},
};

#[derive(CandidType, Clone, Deserialize)]
pub enum OrderFilter {
    ByOfframperId(u64),
    ByOnramperId(u64),
    ByOfframperAddress(TransactionAddress),
    LockedByOnramper(TransactionAddress),
    ByState(OrderStateFilter),
    ByBlockchain(BlockchainType),
    ByBlockchainAsset(BlockchainAsset),
}

#[derive(CandidType, Clone, Deserialize)]
pub enum OrderStateFilter {
    Created,
    Locked,
    Completed,
    Cancelled,
}
