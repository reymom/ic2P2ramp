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
    model::{memory::stable::orders, types::orders::FillRecord},
    types::{BlockchainAsset, icp::get_icp_token, orders::LockedOrder},
};

pub mod crypto;
pub mod paypal;
pub mod revolut;
pub mod stripe;

pub async fn handle_payment_completion(order: &LockedOrder) -> Result<()> {
    let offramper = order.base.offramper_address.address.clone();
    let onramper = order.onramper.address.address.clone();

    let total = order.base.crypto.amount.max(1);
    let locked = order.lock_amount;
    let fee_part: u128 = (order.base.crypto.fee.saturating_mul(locked)) / total;

    let pending = FillRecord {
        payer_user_id: order.onramper.user_id,
        payer: order.onramper.address.clone(),
        provider: order.onramper.provider.clone(),
        fiat: order.price,
        offramper_fee: order.offramper_fee,
        crypto_amount: locked,
        crypto_fee: fee_part,
        payment_id: order.payment_id.clone().unwrap_or_default(),
        tx_id: None,
        created_at: ic_cdk::api::time(),
    };
    let _ = crate::memory::stable::orders::set_pending_fill(order.base.id, pending);

    match order.base.crypto.asset.clone() {
        BlockchainAsset::EVM {
            chain_id,
            token_address,
        } => {
            let mut partial = order.base.crypto.clone();
            partial.amount = locked;
            partial.fee = fee_part;
            Ic2P2ramp::release_funds(
                order.base.id,
                offramper,
                onramper,
                partial,
                chain_id,
                token_address,
            )
            .await
        }
        BlockchainAsset::ICP { ledger_principal } => {
            handle_icp_payment_completion(order, &ledger_principal, fee_part).await
        }
        BlockchainAsset::Bitcoin { rune_id } => {
            let dst_address = order.onramper.address.address.clone();
            let tx_type = match rune_id.clone() {
                Some(rune_id) => TransactionType::RuneTransfer(rune_id),
                None => TransactionType::TaprootBitcoin,
            };
            let net = locked.saturating_sub(fee_part);
            let tx_id = bitcoin::bitcoin_backend_transfer(
                dst_address.clone(),
                net as u64,
                tx_type,
                order.base.crypto.rune_utxos.clone(),
            )
            .await?;

            management::bitcoin::spawn_bitcoin_tx_listener(
                tx_id,
                management::bitcoin::BitcoinTransactionAction::CompleteOrder {
                    order_id: order.base.id,
                    amount: net as u64,
                    onramper_address: dst_address.clone(),
                },
                dst_address,
                rune_id,
                0,
            );

            Ok(())
        }
        BlockchainAsset::Solana { spl_token } => {
            let amt_nat = locked.saturating_sub(fee_part);

            // Send payout to onramper on Solana
            let sig = match spl_token.clone() {
                Some(mint) => {
                    solana_backend_send_spl_token(mint, onramper.clone(), amt_nat.into()).await?
                }
                None => solana_backend_send_sol(onramper.clone(), amt_nat.into()).await?,
            };

            // After L1 confirm, settle escrow + mark completed
            spawn_solana_tx_listener(
                sig,
                SolanaTransactionAction::CompleteOrder {
                    order_id: order.base.id,
                    amount: amt_nat as u64,
                    onramper,
                    token: spl_token,
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
    fee_part: u128,
) -> Result<()> {
    let onramper_principal = Principal::from_text(&order.onramper.address.address).unwrap();

    let amount = NumTokens::from(order.lock_amount);
    let fee = get_icp_token(ledger_principal)?.fee;

    let to_account = Account {
        owner: onramper_principal,
        subaccount: None,
    };
    ICPRamp::transfer(*ledger_principal, to_account, amount - fee_part, Some(fee)).await?;

    let _ = crate::memory::stable::orders::finalize_pending_fill(order.base.id, None);
    orders::set_order_completed(order.base.id)?;

    Ok(())
}
