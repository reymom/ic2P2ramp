use std::collections::HashMap;
use std::{fmt, str::FromStr};

use candid::{CandidType, Deserialize};
use evm_rpc_canister_types::RpcServices;
use ic_cdk::api::management_canister::ecdsa::EcdsaKeyId;

use super::state::{InvalidStateError, State};
use crate::model::types::{
    evm::chains::ChainState,
    payment::{paypal::PayPalState, revolut::RevolutState, truelayer::TrueLayerState},
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

#[derive(CandidType, Deserialize, Clone)]
pub struct TrueLayerConfig {
    pub client_id: String,
    pub client_secret: String,
    pub private_key: Vec<u8>,
    pub kid: String,
    pub host_url: String,
    pub proxy_url: String,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct InitArg {
    pub chains: Vec<ChainConfig>,
    pub ecdsa_key_id: EcdsaKeyId,
    pub paypal: PaypalConfig,
    pub revolut: RevolutConfig,
    pub truelayer: TrueLayerConfig,
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
            truelayer,
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
            paypal: PayPalState::new(paypal.client_id, paypal.client_secret, paypal.api_url),
            revolut: RevolutState::new(
                revolut.client_id,
                revolut.api_url,
                revolut.proxy_url,
                revolut.private_key_der,
                revolut.kid,
                revolut.tan,
            ),
            truelayer: TrueLayerState::new(
                truelayer.client_id,
                truelayer.client_secret,
                truelayer.kid,
                truelayer.private_key,
                truelayer.host_url,
                truelayer.proxy_url,
            ),
            proxy_url,
            icp_tokens: HashMap::new(),
        };
        Ok(state)
    }
}

impl fmt::Debug for TrueLayerConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TrueLayerConfig")
            .field("client_id", &self.client_id)
            .field("private_key_der", &"[SECRET]")
            .field("kid", &self.kid)
            .field("host_url", &self.host_url)
            .field("proxy_url", &self.proxy_url)
            .finish()
    }
}

impl fmt::Debug for RevolutConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RevolutConfig")
            .field("client_id", &self.client_id)
            .field("api_url", &self.api_url)
            .field("proxy_url", &self.proxy_url)
            .field("private_key_der", &"[SECRET]")
            .field("kid", &self.kid)
            .field("tan", &self.tan)
            .finish()
    }
}
