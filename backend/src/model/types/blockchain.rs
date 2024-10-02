use candid::{CandidType, Deserialize, Principal};

use crate::errors::{BlockchainError, Result};

use super::{
    evm::{chains, token},
    icp,
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

    pub fn get_symbol(&self) -> Result<String> {
        match &self.blockchain {
            Blockchain::EVM { chain_id } => match &self.token {
                Some(token_address) => {
                    Ok(token::get_evm_token(*chain_id, token_address)?.rate_symbol)
                }
                None => Ok(chains::get_native_currency_symbol(*chain_id)?),
            },
            Blockchain::ICP { ledger_principal } => {
                Ok(icp::get_icp_token(ledger_principal)?.symbol)
            }
            _ => Err(BlockchainError::UnsupportedBlockchain.into()),
        }
    }

    fn get_decimals(&self) -> Result<u8> {
        match &self.blockchain {
            Blockchain::EVM { chain_id } => {
                if let Some(token_address) = &self.token {
                    Ok(token::get_evm_token(*chain_id, token_address)?.decimals)
                } else {
                    Ok(18)
                }
            }
            Blockchain::ICP { ledger_principal } => {
                Ok(icp::get_icp_token(ledger_principal)?.decimals)
            }
            _ => Err(BlockchainError::UnsupportedBlockchain.into()),
        }
    }

    pub fn to_whole_units(&self) -> Result<f64> {
        let decimals = self.get_decimals()?;
        let divisor = 10u128.pow(decimals as u32);
        Ok((self.amount as f64) / (divisor as f64))
    }
}
