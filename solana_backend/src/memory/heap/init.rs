use candid::{CandidType, Deserialize, Principal};
use sol_rpc_types::SolanaCluster;

use crate::model::types::ed25519::Ed25519KeyName;

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct InitArg {
    pub sol_rpc_canister_id: Option<Principal>,
    pub network: SolanaCluster,
    pub ed25519_key_name: Ed25519KeyName,
    pub proxy_url: String,
}
