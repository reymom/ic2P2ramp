// Export the necessary types for inter-canister calls and integration tests

pub use crate::model::types::{
    errors,
    inscription::Inscription,
    runes::Etching,
    runes::{RuneID, RuneMetadata},
    transfer::TransactionType,
    utxo::RuneUTXOEntry,
    vault::VaultEntry,
    Address,
};
