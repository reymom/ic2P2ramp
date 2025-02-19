use bitcoin::Txid;
use ic_cdk::api::management_canister::bitcoin::Satoshi;

use crate::{
    errors::BitcoinError,
    model::types::{errors::Result, transfer::TransactionType, wallet::WalletConfig},
};

pub async fn send_btc_or_ordinal(
    dst_address: String,
    amount: Satoshi,
    tx_type: TransactionType,
) -> Result<Txid> {
    match tx_type {
        TransactionType::LegacyBitcoin => {
            crate::wallet::p2pkh::send(WalletConfig::for_p2pkh(), dst_address, amount).await
        }
        TransactionType::TaprootBitcoin
        | TransactionType::RuneTransfer(_)
        | TransactionType::OrdinalTransfer => {
            crate::wallet::p2tr_raw_key_spend::send_key_spend(
                WalletConfig::for_p2tr_raw_key(),
                dst_address,
                amount,
                tx_type,
            )
            .await
        }
        TransactionType::RuneEtching(_) | TransactionType::OrdinalInscription(_) => {
            Err(BitcoinError::InternalError("not available yet".to_string()))
        }
    }
}
