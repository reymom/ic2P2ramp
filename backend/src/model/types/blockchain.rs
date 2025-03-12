use bitcoin_backend::types::RuneID;
use candid::{CandidType, Deserialize, Principal};

use crate::{
    errors::{BlockchainError, Result},
    inter_canister::bitcoin,
    model::helpers::normalize_rune_name,
};

use super::{
    evm::{chains, token},
    icp,
};

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum BlockchainAsset {
    EVM {
        chain_id: u64,
        token_address: Option<String>,
    },
    ICP {
        ledger_principal: Principal,
    },
    Bitcoin {
        rune_id: Option<RuneID>,
    },
    Solana,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Crypto {
    pub asset: BlockchainAsset,
    pub amount: u128,
    pub fee: u128,
}

impl Crypto {
    pub fn new(asset: BlockchainAsset, amount: u128, fee: u128) -> Self {
        Self { asset, amount, fee }
    }

    pub async fn to_whole_units(&self) -> Result<f64> {
        let decimals = self.asset.get_decimals().await?;
        let divisor = 10u128.pow(decimals as u32);
        Ok((self.amount as f64) / (divisor as f64))
    }
}

impl BlockchainAsset {
    pub async fn get_symbol(&self) -> Result<String> {
        match &self {
            Self::EVM {
                chain_id,
                token_address,
            } => match token_address {
                Some(token_address) => {
                    Ok(token::get_evm_token(*chain_id, token_address)?.rate_symbol)
                }
                None => Ok(chains::get_native_currency_symbol(*chain_id)?),
            },
            Self::ICP { ledger_principal } => Ok(icp::get_icp_token(ledger_principal)?.symbol),
            Self::Bitcoin { rune_id } => match rune_id {
                Some(rune_id) => Ok(normalize_rune_name(
                    &bitcoin::bitcoin_backend_get_rune_metadata(rune_id.to_string())
                        .await?
                        .name,
                )),
                None => Ok("BTC".to_string()),
            },
            _ => Err(BlockchainError::UnsupportedBlockchain.into()),
        }
    }

    async fn get_decimals(&self) -> Result<u8> {
        match &self {
            Self::EVM {
                chain_id,
                token_address,
            } => {
                if let Some(token_address) = token_address {
                    Ok(token::get_evm_token(*chain_id, token_address)?.decimals)
                } else {
                    Ok(18)
                }
            }
            Self::ICP { ledger_principal } => Ok(icp::get_icp_token(ledger_principal)?.decimals),
            Self::Bitcoin { rune_id } => {
                if let Some(rune_id) = rune_id {
                    Ok(
                        bitcoin::bitcoin_backend_get_rune_metadata(rune_id.to_string())
                            .await?
                            .divisibility,
                    )
                } else {
                    Ok(8)
                }
            }
            _ => Err(BlockchainError::UnsupportedBlockchain.into()),
        }
    }
}
