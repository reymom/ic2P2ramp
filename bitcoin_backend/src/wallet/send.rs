use bitcoin::Txid;
use ic_cdk::api::management_canister::bitcoin::Satoshi;

use crate::{
    memory::heap::config,
    model::types::{
        errors::Result,
        transfer::TransactionType,
        wallet::{TaprootUseCase, WalletConfig},
    },
};

pub async fn send_btc_or_rune(
    dst_address: String,
    amount: Satoshi,
    tx_type: TransactionType,
) -> Result<Txid> {
    match tx_type {
        TransactionType::LegacyBitcoin => {
            crate::wallet::p2pkh::send(WalletConfig::for_p2pkh(), dst_address, amount).await
        }
        TransactionType::SimpleTaprootBitcoin => {
            crate::wallet::p2tr_raw_key_spend::send_key_spend(
                WalletConfig::for_p2tr_raw_key(),
                dst_address,
                amount,
            )
            .await
        }
        TransactionType::ScriptedTaprootBitcoin => {
            crate::wallet::p2tr_script_spend::send_script_spend(
                WalletConfig::for_p2tr_script(),
                TaprootUseCase::Standard,
                dst_address,
                amount,
                tx_type,
            )
            .await
        }
        TransactionType::RuneTransfer(ref rune) => {
            let rune_symbol = config::get_rune_metadata(&rune)?.symbol;
            ic_cdk::println!("[send] rune_symbol: {}", rune_symbol);
            crate::wallet::p2tr_script_spend::send_script_spend(
                WalletConfig::for_p2tr_script(),
                TaprootUseCase::RuneTransfer(rune_symbol),
                dst_address,
                amount,
                tx_type,
            )
            .await
        }
    }
}
