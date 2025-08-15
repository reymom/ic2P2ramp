use candid::{CandidType, Deserialize, Principal};
use icrc_ledger_types::icrc1::transfer::NumTokens;

use crate::model::{
    errors::{BlockchainError, Result},
    memory::heap::read_state,
};

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct IcpToken {
    pub symbol: String,
    pub decimals: u8,
    pub fee: NumTokens,
}

impl IcpToken {
    pub fn new(symbol: &str, decimals: u8, fee: NumTokens) -> Self {
        Self {
            symbol: symbol.to_string(),
            decimals,
            fee,
        }
    }
}

pub fn get_icp_token(ledger_principal: &Principal) -> Result<IcpToken> {
    read_state(|state| {
        state
            .icp_tokens
            .get(ledger_principal)
            .cloned()
            .ok_or_else(|| {
                BlockchainError::LedgerPrincipalNotSupported(ledger_principal.to_string()).into()
            })
    })
}

pub fn is_icp_token_supported(ledger_principal: &Principal) -> Result<()> {
    read_state(|state| {
        if state.icp_tokens.contains_key(ledger_principal) {
            Ok(())
        } else {
            Err(BlockchainError::LedgerPrincipalNotSupported(ledger_principal.to_string()).into())
        }
    })
}
