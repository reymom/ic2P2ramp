use candid::{CandidType, Deserialize, Principal};
use icrc_ledger_types::icrc1::transfer::NumTokens;

use crate::model::{
    errors::{RampError, Result},
    memory::heap::read_state,
};

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Blockchain {
    EVM { chain_id: u64 },
    ICP { ledger_principal: Principal },
    Solana,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Crypto {
    pub blockchain: Blockchain,
    pub token: Option<String>, // For EVM tokens, this will be the contract address
    pub amount: u128,
    pub fee: u128,
}

impl Crypto {
    pub fn new(blockchain: Blockchain, token: Option<String>, amount: u128, fee: u128) -> Self {
        Self {
            blockchain,
            token,
            amount,
            fee,
        }
    }
}

pub fn get_fee(ledger_principal: &Principal) -> Result<NumTokens> {
    read_state(|state| {
        state
            .icp_fees
            .get(ledger_principal)
            .cloned()
            .ok_or_else(|| RampError::LedgerPrincipalNotSupported(ledger_principal.to_string()))
    })
}

pub fn is_icp_token_supported(ledger_principal: &Principal) -> Result<()> {
    read_state(|state| {
        if state.icp_fees.contains_key(ledger_principal) {
            Ok(())
        } else {
            Err(RampError::LedgerPrincipalNotSupported(
                ledger_principal.to_string(),
            ))
        }
    })
}
