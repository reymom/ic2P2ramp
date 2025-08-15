use candid::{CandidType, Deserialize};
use ic_btc_interface::Network;

#[derive(CandidType, Deserialize, Default, Clone, Debug)]
pub struct UnisatConfig {
    pub api_url: String,
    pub api_key: String,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct InitArg {
    pub network: Network,
    pub proxy_url: String,
    pub unisat: UnisatConfig,
}
