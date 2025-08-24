use candid::{CandidType, Deserialize};

use super::{
    inscription::Inscription,
    runes::{Etching, RuneID},
};

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
