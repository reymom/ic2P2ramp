use candid::Principal;
use icramp_types::bitcoin::transfer::TransactionType;
use icrc_ledger_types::icrc1::{account::Account, transfer::NumTokens};

use crate::{
    errors::Result,
    evm::vault::Ic2P2ramp,
    icp::vault::Ic2P2ramp as ICPRamp,
    inter_canister::{
        bitcoin,
        solana::{solana_backend_send_sol, solana_backend_send_spl_token},
    },
    management::{
        self,
        solana::{SolanaTransactionAction, spawn_solana_tx_listener},
    },
    model::memory::stable::orders,
    types::{BlockchainAsset, icp::get_icp_token, orders::LockedOrder},
};

pub mod paypal;
pub mod revolut;
pub mod stripe;

pub async fn handle_payment_completion(order: &LockedOrder) -> Result<()> {
    let offramper = order.base.offramper_address.address.clone();
    let onramper = order.onramper.address.address.clone();
    match order.base.crypto.asset.clone() {
        BlockchainAsset::EVM {
            chain_id,
            token_address,
        } => {
            Ic2P2ramp::release_funds(
                order.base.id,
                offramper,
                onramper,
                order.base.crypto.clone(),
                chain_id,
                token_address,
            )
            .await
        }
        BlockchainAsset::ICP { ledger_principal } => {
            handle_icp_payment_completion(order, &ledger_principal).await
        }
        BlockchainAsset::Bitcoin { rune_id } => {
            let dst_address = order.onramper.address.address.clone();
            let tx_type = match rune_id.clone() {
                Some(rune_id) => TransactionType::RuneTransfer(rune_id),
                None => TransactionType::TaprootBitcoin,
            };
            let tx_id = bitcoin::bitcoin_backend_transfer(
                dst_address.clone(),
                order.base.crypto.amount as u64,
                tx_type,
                order.base.crypto.rune_utxos.clone(),
            )
            .await?;

            management::bitcoin::spawn_bitcoin_tx_listener(
                tx_id,
                management::bitcoin::BitcoinTransactionAction::CompleteOrder {
                    order_id: order.base.id,
                    amount: order.base.crypto.amount as u64,
                    onramper_address: dst_address.clone(),
                },
                dst_address,
                rune_id,
                0,
            );

            Ok(())
        }
        BlockchainAsset::Solana { spl_token } => {
            let amt_nat = candid::Nat::from(order.base.crypto.amount);

            // Send payout to onramper on Solana
            let sig = match spl_token.clone() {
                Some(mint) => {
                    solana_backend_send_spl_token(mint, onramper.clone(), amt_nat).await?
                }
                None => {
                    solana_backend_send_sol(
                        onramper.clone(),
                        candid::Nat::from(order.base.crypto.amount),
                    )
                    .await?
                }
            };

            // After L1 confirm, settle escrow + mark completed
            spawn_solana_tx_listener(
                sig,
                SolanaTransactionAction::CompleteOrder {
                    order_id: order.base.id,
                    amount: order.base.crypto.amount as u64,
                    onramper,         // to whom we paid
                    token: spl_token, // mint if SPL
                },
                0,
            );

            Ok(())
        }
    }
}

async fn handle_icp_payment_completion(
    order: &LockedOrder,
    ledger_principal: &Principal,
) -> Result<()> {
    let onramper_principal = Principal::from_text(&order.onramper.address.address).unwrap();

    let amount = NumTokens::from(order.base.crypto.amount);
    let fee = get_icp_token(ledger_principal)?.fee;

    let to_account = Account {
        owner: onramper_principal,
        subaccount: None,
    };
    ICPRamp::transfer(
        *ledger_principal,
        to_account,
        amount - order.base.crypto.fee,
        Some(fee),
    )
    .await?;

    orders::unset_processing_order(&order.base.id)?;
    super::order::set_order_completed(order.base.id)?;

    Ok(())
}
