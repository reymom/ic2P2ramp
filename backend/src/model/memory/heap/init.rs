use std::collections::HashMap;
use std::{fmt, str::FromStr};

use candid::{CandidType, Deserialize};
use evm_rpc_canister_types::RpcServices;
use ic_cdk::api::management_canister::ecdsa::EcdsaKeyId;

use super::state::{InvalidStateError, State};
use crate::model::types::{
    evm::chains::ChainState,
    payment::{paypal::PayPalState, revolut::RevolutState},
};

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct ChainConfig {
    pub chain_id: u64,
    pub vault_manager_address: String,
    pub services: RpcServices,
    pub currency_symbol: String,
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct PaypalConfig {
    pub client_id: String,
    pub client_secret: String,
    pub api_url: String,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct RevolutConfig {
    pub client_id: String,
    pub api_url: String,
    pub proxy_url: String,
    pub private_key_der: Vec<u8>,
    pub kid: String,
    pub tan: String,
}

impl fmt::Debug for RevolutConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RevolutConfig")
            .field("client_id", &self.client_id)
            .field("api_url", &self.api_url)
            .field("proxy_url", &self.proxy_url)
            .field("private_key_der", &"[REDACTED]")
            .field("kid", &self.kid)
            .field("tan", &self.tan)
            .finish()
    }
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct InitArg {
    pub chains: Vec<ChainConfig>,
    pub ecdsa_key_id: EcdsaKeyId,
    pub paypal: PaypalConfig,
    pub revolut: RevolutConfig,
    pub proxy_url: String,
}

impl TryFrom<InitArg> for State {
    type Error = InvalidStateError;

    fn try_from(
        InitArg {
            chains,
            ecdsa_key_id,
            paypal,
            revolut,
            proxy_url,
        }: InitArg,
    ) -> Result<Self, Self::Error> {
        let mut chains_map = HashMap::new();
        for config in chains {
            ethers_core::types::Address::from_str(&config.vault_manager_address).map_err(|e| {
                InvalidStateError::InvalidEthereumContractAddress(format!("ERROR: {}", e))
            })?;

            chains_map.insert(
                config.chain_id,
                ChainState::new(
                    config.vault_manager_address,
                    config.services,
                    config.currency_symbol,
                ),
            );
        }

        let state = Self {
            chains: chains_map,
            ecdsa_pub_key: None,
            ecdsa_key_id,
            evm_address: None,
            paypal: PayPalState {
                access_token: None,
                token_expiration: None,
                client_id: paypal.client_id,
                client_secret: paypal.client_secret,
                api_url: paypal.api_url,
            },
            revolut: RevolutState {
                access_token: None,
                token_expiration: None,
                client_id: revolut.client_id,
                api_url: revolut.api_url,
                proxy_url: revolut.proxy_url,
                private_key_der: revolut.private_key_der,
                kid: revolut.kid,
                tan: revolut.tan,
            },
            proxy_url,
            icp_tokens: HashMap::new(),
        };
        Ok(state)
    }
}
