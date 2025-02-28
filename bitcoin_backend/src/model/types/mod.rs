pub mod errors;
pub mod inscription;
pub mod runes;
pub mod schnorr;
pub mod transfer;
pub mod utxo;
pub mod vault;
#[cfg(feature = "canister")]
pub mod wallet;

pub type Address = String;
