// Export the necessary types for inter-canister calls and integration tests

pub use crate::model::types::{
    errors,
    runes::Etching,
    runes::{RuneID, RuneMetadata},
    transfer::TransactionType,
    utxo::RuneUTXOEntry,
    vault::VaultEntry,
    wallet::WalletConfig,
    Address,
};
pub use crate::ordinals::inscription::Inscription;
