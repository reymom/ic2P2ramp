use candid::{CandidType, Deserialize, Principal};
use icramp_types::bitcoin::runes::{RuneID, RuneUTXOEntry};

use crate::{
    errors::{Result, SystemError},
    inter_canister::{
        bitcoin::{self, bitcoin_backend_validate_rune},
        solana::{self, solana_backend_get_token_info},
    },
    model::types::{exchange_rate::RateAsset, icp::is_icp_token_supported},
};

use super::{
    evm::{chains, token},
    icp,
};

#[derive(CandidType, Deserialize, PartialEq, Clone)]
pub enum BlockchainType {
    EVM,
    ICP,
    Bitcoin,
    Solana,
}

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
    Solana {
        spl_token: Option<String>,
    },
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Crypto {
    pub asset: BlockchainAsset,
    pub amount: u128,
    pub fee: u128,
    pub rune_utxos: Option<Vec<RuneUTXOEntry>>,
}

impl Crypto {
    pub fn new(
        asset: BlockchainAsset,
        amount: u128,
        fee: u128,
        rune_utxos: Option<Vec<RuneUTXOEntry>>,
    ) -> Result<Self> {
        if let BlockchainAsset::Bitcoin { ref rune_id } = asset {
            if rune_id.is_some() {
                let utxos = rune_utxos.as_ref().ok_or_else(|| {
                    SystemError::InvalidInput("Missing rune UTXOs for Bitcoin order".to_string())
                })?;
                let total_runes: u64 = utxos.iter().map(|u| u.rune_amount).sum();
                if total_runes != amount as u64 {
                    return Err(SystemError::InvalidInput(format!(
                        "Rune UTXOs total {} does not equal expected amount {}",
                        total_runes, amount
                    ))
                    .into());
                }
            }
        }

        Ok(Self {
            asset,
            amount,
            fee,
            rune_utxos,
        })
    }
}

impl BlockchainAsset {
    pub fn blockchain_type(&self) -> BlockchainType {
        match &self {
            Self::EVM { .. } => BlockchainType::EVM,
            Self::Bitcoin { .. } => BlockchainType::Bitcoin,
            Self::ICP { .. } => BlockchainType::ICP,
            Self::Solana { .. } => BlockchainType::Solana,
        }
    }

    pub async fn get_symbol_and_decimals(&self) -> Result<(String, u8)> {
        match &self {
            Self::EVM {
                chain_id,
                token_address,
            } => match token_address {
                Some(token_address) => {
                    let token = token::get_evm_token(*chain_id, token_address)?;
                    Ok((token.rate_symbol, token.decimals))
                }
                None => Ok((chains::get_native_currency_symbol(*chain_id)?, 18)),
            },
            Self::ICP { ledger_principal } => {
                let token = icp::get_icp_token(ledger_principal)?;
                Ok((token.symbol, token.decimals))
            }
            Self::Bitcoin { rune_id } => match rune_id {
                Some(rune_id) => {
                    let token =
                        bitcoin::bitcoin_backend_get_rune_metadata(rune_id.to_string()).await?;
                    Ok((token.name, token.divisibility))
                }
                None => Ok(("BTC".to_string(), 8)),
            },
            Self::Solana { spl_token } => match spl_token {
                Some(mint) => {
                    let token = solana::solana_backend_get_token_info(mint.to_string()).await?;
                    Ok((token.rate_symbol, token.decimals))
                }
                None => Ok(("SOL".to_string(), 9)),
            },
        }
    }

    pub fn to_rate_asset(&self, symbol: &str) -> RateAsset {
        match &self {
            Self::Bitcoin { .. } => RateAsset::Rune {
                name: symbol.to_string(),
            },
            Self::Solana { spl_token } => RateAsset::Solana {
                symbol: symbol.to_string(),
                mint: spl_token.clone(),
            },
            _ => RateAsset::Crypto {
                symbol: symbol.to_string(),
            },
        }
    }

    pub async fn validate(&self) -> Result<()> {
        match self {
            BlockchainAsset::EVM {
                chain_id,
                token_address,
            } => {
                chains::chain_is_supported(*chain_id)?;
                if let Some(token) = token_address.clone() {
                    token::evm_token_is_approved(*chain_id, &token)?;
                };
            }
            BlockchainAsset::ICP { ledger_principal } => is_icp_token_supported(ledger_principal)?,
            BlockchainAsset::Bitcoin { rune_id } => {
                if let Some(rune_id) = rune_id.clone() {
                    bitcoin_backend_validate_rune(rune_id).await?;
                };
            }
            BlockchainAsset::Solana { spl_token } => {
                if let Some(mint) = spl_token {
                    let _ = solana_backend_get_token_info(mint.clone()).await?;
                };
            }
        };
        Ok(())
    }
}
