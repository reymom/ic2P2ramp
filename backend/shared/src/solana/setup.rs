use candid::{CandidType, Deserialize, Principal};
use sol_rpc_types::SolanaCluster;

use crate::solana::ed25519::Ed25519KeyName;

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct InitArg {
    pub sol_rpc_canister_id: Option<Principal>,
    pub network: SolanaCluster,
    pub ed25519_key_name: Ed25519KeyName,
    pub proxy_url: String,
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct UpdateArg {
    pub network: Option<SolanaCluster>,
    pub sol_rpc_canister_id: Option<Principal>,
    pub proxy_url: Option<String>,
}

#[derive(candid::CandidType, candid::Deserialize, Debug)]
pub enum InstallArg {
    Reinstall(InitArg),
    Upgrade(Option<UpdateArg>),
}
