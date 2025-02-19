use candid::{CandidType, Deserialize};

use crate::{ordinals::inscription::Inscription, RuneID, WalletConfig};

use super::runes::Etching;

#[derive(Deserialize, CandidType, Clone)]
pub enum TransactionType {
    LegacyBitcoin,                   // Use P2PKH
    TaprootBitcoin,                  // Use P2TR (Raw Key Spend)
    RuneTransfer(RuneID),            // Transfer Runes using P2TR Raw Key Spend
    RuneEtching(Etching),            // Etch a new Rune using P2TR Script Spend
    OrdinalTransfer,                 // Transfer Ordinals (NFTs in Bitcoin)
    OrdinalInscription(Inscription), // Create an Ordinal inscription (minting NFTs)
}

impl TransactionType {
    pub fn get_wallet_config(self) -> WalletConfig {
        match self {
            Self::LegacyBitcoin => WalletConfig::for_p2pkh(),
            Self::RuneTransfer(_) | Self::TaprootBitcoin | Self::OrdinalTransfer => {
                WalletConfig::for_p2tr_raw_key()
            }
            Self::RuneEtching(_) => WalletConfig::for_p2tr_script(),
            Self::OrdinalInscription(_) => WalletConfig::for_p2tr_script(),
        }
    }

    pub fn to_taproot_use_case(&self) -> Option<TaprootUseCase> {
        match self {
            TransactionType::RuneEtching(etching) => {
                Some(TaprootUseCase::RuneEtching(etching.clone()))
            }
            TransactionType::OrdinalInscription(inscription) => {
                Some(TaprootUseCase::Inscription(inscription.clone()))
            }
            _ => None,
        }
    }
}

pub enum TaprootUseCase {
    Standard,
    RuneEtching(Etching),
    Inscription(Inscription),
}
