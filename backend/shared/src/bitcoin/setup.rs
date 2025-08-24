use candid::{CandidType, Deserialize, Principal};
use ic_btc_interface::Network;

#[derive(CandidType, candid::Deserialize, Debug)]
pub enum InstallArg {
    Reinstall(InitArg),
    Upgrade(Option<UpdateArg>),
}

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

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct UpdateArg {
    pub network: Option<Network>,
    pub btc_principal: Option<Principal>,
    pub unisat: Option<UnisatConfig>,
    pub proxy_url: Option<String>,
}
