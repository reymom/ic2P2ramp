use candid::{CandidType, Deserialize, Principal};
use icramp_types::bitcoin::runes::{RuneID, RuneUTXOEntry};

use crate::{
    errors::{BlockchainError, Result, SystemError},
    inter_canister::{bitcoin, solana},
    model::types::exchange_rate::RateAsset,
};

use super::{
    evm::{chains, token},
    icp,
};

#[derive(CandidType, Deserialize, PartialEq, Clone)]
pub enum BlockchainType {
    EVM,
    ICP,
    Bitcion,
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

    pub async fn to_whole_units(&self) -> Result<f64> {
        let decimals = self.asset.get_decimals().await?;
        let divisor = 10u128.pow(decimals as u32);
        Ok((self.amount as f64) / (divisor as f64))
    }
}

impl BlockchainAsset {
    pub fn blockchain_type(&self) -> BlockchainType {
        match &self {
            Self::EVM { .. } => BlockchainType::EVM,
            Self::Bitcoin { .. } => BlockchainType::Bitcion,
            Self::ICP { .. } => BlockchainType::ICP,
            Self::Solana { .. } => BlockchainType::Solana,
        }
    }

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
                Some(rune_id) => Ok(bitcoin::bitcoin_backend_get_rune_metadata(
                    rune_id.to_string(),
                )
                .await?
                .name),
                None => Ok("BTC".to_string()),
            },
            Self::Solana { spl_token } => match spl_token {
                Some(mint) => Ok(solana::solana_backend_get_token_info(mint.to_string())
                    .await?
                    .rate_symbol),
                None => Ok("SOL".to_string()),
            },
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
}
