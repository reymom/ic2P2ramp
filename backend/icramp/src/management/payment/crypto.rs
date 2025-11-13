use crate::{
    management::verifiers::{
        verify_bitcoin_deposit, verify_evm_transaction, verify_solana_deposit,
    },
    model::{
        errors::Result,
        memory::stable::orders::{self, append_fill_if_new},
        types::{
            BlockchainAsset,
            icp::is_icp_token_supported,
            orders::{DepositInput, FillRecord},
        },
    },
};

/// Verifies that a user-supplied transaction really corresponds to the
/// expected on-chain deposit (amount, asset, addresses, freshness) or transaction (from, to, amount, asset).
///
/// Returns the chain-specific tx id / signature if one exists for that asset.
/// - `sender` is the "offramper" address for deposits in `create_order` and "onramper" in verification of "pay-with-crypto" paths.
/// - `receiver` is the "vault" smart contract address for deposits in `create_order` and "offramper" in verification of "pay-with-crypto" paths.
/// - `amount` is in native units (lamports / wei / smallest token units).
pub async fn verify_crypto_transaction(
    asset: &BlockchainAsset,
    deposit_input: Option<DepositInput>,
    sender: &str,
    // receiver: &str,
    amount: u128,
) -> Result<Option<String>> {
    match asset {
        BlockchainAsset::EVM {
            chain_id,
            token_address,
        } => {
            verify_evm_transaction(
                *chain_id,
                token_address.clone(),
                deposit_input,
                sender,
                // receiver,
                amount,
            )
            .await
        }

        BlockchainAsset::ICP { ledger_principal } => {
            // We cannot verify ICP txs from the canister; we only check token support
            // and trust the frontend agent flow.
            is_icp_token_supported(ledger_principal)?;
            Ok(None)
        }

        BlockchainAsset::Bitcoin { rune_id } => {
            verify_bitcoin_deposit(rune_id.clone(), deposit_input, amount).await
        }

        BlockchainAsset::Solana { spl_token } => {
            verify_solana_deposit(spl_token.clone(), deposit_input, amount).await
        }
    }
}

/// Called once a Bitcoin pay-with-crypto tx has been confirmed on L1.
/// This mirrors the PayPal “verified → paid” state transition, but for BTC.
pub async fn on_bitcoin_payment_confirmed(order_id: u64, txid: String) -> Result<()> {
    ic_cdk::println!(
        "[on_bitcoin_payment_confirmed] order_id = {}, txid = {}",
        order_id,
        txid
    );
    let order = orders::get_order(&order_id)?.locked()?;

    // 1) Record payment id = tx hash and mark the order as “paid” (pre-vault-release state).
    crate::memory::stable::orders::set_payment_id(order_id, txid.clone())?;
    crate::management::order::mark_order_as_paid(order_id)?;

    // 2) Append a fill record like PayPal, but with payment_id = txid and tx_id = Some(txid).
    let total = order.base.crypto.amount.max(1);
    let locked = order.lock_amount;
    let fee_part: u128 = (order.base.crypto.fee.saturating_mul(locked)) / total;

    append_fill_if_new(
        order_id,
        FillRecord {
            payer_user_id: order.onramper.user_id,
            payer: order.onramper.address.clone(),
            provider: order.onramper.provider.clone(),
            fiat: order.price,
            offramper_fee: order.offramper_fee,
            crypto_amount: locked,
            crypto_fee: fee_part,
            payment_id: txid.clone(),
            tx_id: None,
            created_at: ic_cdk::api::time(),
        },
    )?;

    // 3) Mark this payment tx as processed so it cannot be reused to pay other orders.
    crate::model::memory::stable::spent_transactions::mark_tx_hash_as_processed(txid.clone());

    // 4) Now trigger the vault-side release of the order base asset to the onramper.
    //    For BTC orders this will go through the existing Bitcoin “CompleteOrder” listener,
    //    for EVM/SOL orders it will trigger the respective vault operations.
    super::handle_payment_completion(&order).await
}
