use candid::{CandidType, Deserialize};
use ethers_core::types::U256;
use ic_cdk::api::management_canister::ecdsa::EcdsaKeyId;
use std::collections::HashMap;
use std::str::FromStr;

use crate::evm::rpc::RpcServices;

use super::chains::ChainState;
use super::paypal::PayPalState;
use super::state::InvalidStateError;
use super::State;

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct ChainConfig {
    pub chain_id: u64,
    pub vault_manager_address: String,
    pub services: RpcServices,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct InitArg {
    pub chains: Vec<ChainConfig>,
    pub ecdsa_key_id: EcdsaKeyId,
    pub client_id: String,
    pub client_secret: String,
}

impl TryFrom<InitArg> for State {
    type Error = InvalidStateError;

    fn try_from(
        InitArg {
            chains,
            ecdsa_key_id,
            client_id,
            client_secret,
        }: InitArg,
    ) -> Result<Self, Self::Error> {
        let mut chains_map = HashMap::new();
        for config in chains {
            ethers_core::types::Address::from_str(&config.vault_manager_address).map_err(|e| {
                InvalidStateError::InvalidEthereumContractAddress(format!("ERROR: {}", e))
            })?;

            chains_map.insert(
                config.chain_id,
                ChainState {
                    vault_manager_address: config.vault_manager_address,
                    rpc_services: config.services,
                    nonce: U256::zero(),
                    approved_tokens: HashMap::new(),
                },
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
                client_id,
                client_secret,
            },
        };
        Ok(state)
    }
}
