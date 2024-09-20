use std::collections::HashMap;

use candid::{CandidType, Deserialize};

use crate::model::{
    errors::{RampError, Result},
    memory::heap::{mutate_state, read_state},
};

#[derive(CandidType, Clone, Debug, Deserialize)]
pub struct Token {
    pub address: String,
    pub decimals: u8,
    pub rate_symbol: String,
    pub rate: Option<f64>,           // Optional token rate
    pub description: Option<String>, // Optional metadata
}

impl Token {
    pub fn new(address: String, decimals: u8, rate_symbol: &str) -> Self {
        Self {
            address,
            decimals,
            rate_symbol: rate_symbol.to_string(),
            rate: None,
            description: None,
        }
    }

    // pub fn set_rate(&mut self, rate: f64) {
    //     self.rate = Some(rate);
    // }

    pub fn set_description(&mut self, desc: Option<String>) {
        self.description = desc;
    }
}

#[derive(Clone, Debug)]
pub struct TokenManager {
    pub tokens: HashMap<String, Token>,
}

impl TokenManager {
    pub fn new() -> Self {
        Self {
            tokens: HashMap::new(),
        }
    }

    pub fn add_token(&mut self, address: String, token: Token) {
        self.tokens.insert(address, token);
    }
}

pub fn approve_evm_tokens(chain_id: u64, tokens: HashMap<String, Token>) {
    mutate_state(|state| {
        if let Some(chain_state) = state.chains.get_mut(&chain_id) {
            for (address, token) in tokens {
                chain_state.approved_tokens.insert(address, token);
            }
        }
    })
}

pub fn get_evm_token(chain_id: u64, token_address: &str) -> Result<Token> {
    read_state(|state| {
        state
            .chains
            .get(&chain_id)
            .ok_or_else(|| RampError::ChainIdNotFound(chain_id))
            .and_then(|chain_state| {
                chain_state
                    .approved_tokens
                    .get(token_address)
                    .cloned()
                    .ok_or_else(|| RampError::UnregisteredEvmToken)
            })
    })
}

pub fn evm_token_is_approved(chain_id: u64, token_address: &str) -> Result<()> {
    get_evm_token(chain_id, token_address).map(|_| ())
}
