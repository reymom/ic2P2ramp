#[cfg(feature = "canister")]
mod api;
#[cfg(feature = "canister")]
mod memory;
#[cfg(feature = "types")]
mod model;
#[cfg(feature = "canister")]
mod ordinals;
#[cfg(feature = "types")]
pub mod types;
#[cfg(feature = "canister")]
mod vault;
#[cfg(feature = "canister")]
mod wallet;

#[cfg(feature = "canister")]
use ic_btc_interface::{GetBlockHeadersResponse, Network, Utxo};
#[cfg(feature = "canister")]
use std::collections::HashMap;

#[cfg(feature = "canister")]
use crate::types::errors::{BitcoinError, Result, VaultError};
#[cfg(feature = "canister")]
use crate::types::{
    Address, Inscription, RuneID, RuneMetadata, RuneUTXOEntry, TransactionType, VaultEntry,
};
#[cfg(feature = "canister")]
use memory::{
    heap::config::{BTC_PRINCIPAL, KEY_NAME, NETWORK},
    stable::{
        utxos::add_rune_utxo_entries,
        vault::{OFFRAMPER_VAULTS, ONRAMPER_VAULTS},
    },
};
#[cfg(feature = "canister")]
use model::types::wallet::WalletConfig;
#[cfg(feature = "canister")]
use ordinals::inscription;
#[cfg(feature = "canister")]
use wallet::utxos::get_tx_utxos;

#[cfg(feature = "canister")]
#[ic_cdk::init]
pub fn init(network: Network) {
    NETWORK.with(|n| n.set(network));

    KEY_NAME.with(|key_name| {
        key_name.replace(String::from(match network {
            Network::Regtest => "dfx_test_key",
            Network::Mainnet | Network::Testnet => "test_key_1",
        }))
    });

    BTC_PRINCIPAL.with(|principal| {
        principal.replace(
            candid::Principal::from_text(match network {
                Network::Regtest | Network::Testnet => "g4xu7-jiaaa-aaaan-aaaaq-cai",
                Network::Mainnet => "ghsi2-tqaaa-aaaan-aaaca-cai",
            })
            .expect("invalid bitcoin canister principal"),
        );
    });
}

// ----
// TEST
// ----

/// Returns the balance of the given bitcoin address.
#[cfg(feature = "canister")]
#[ic_cdk::update]
pub async fn get_btc_balance(address: String) -> Result<u64> {
    let network = NETWORK.with(|n| n.get());
    let btc_principal: candid::Principal = BTC_PRINCIPAL.with(|bp| *bp.borrow());
    api::bitcoin::get_balance(network, btc_principal, address).await
}

#[cfg(feature = "canister")]
#[ic_cdk::update]
pub async fn get_btc_block_headers() -> Result<GetBlockHeadersResponse> {
    let network = NETWORK.with(|n| n.get());
    let btc_principal: candid::Principal = BTC_PRINCIPAL.with(|bp| *bp.borrow());
    api::bitcoin::get_block_headers(network, btc_principal, 0, Some(u32::MAX)).await
}

#[cfg(feature = "canister")]
#[ic_cdk::update]
pub async fn test_transfer(
    dst_address: String,
    amount: u64,
    tx_type: TransactionType,
) -> Result<String> {
    let tx_id = wallet::send::send_btc_or_ordinal(dst_address, amount, tx_type).await?;
    Ok(tx_id.to_string())
}

#[cfg(feature = "canister")]
#[ic_cdk::update]
pub async fn get_utxos(
    address: String,
    tx_type: TransactionType,
) -> Result<(Vec<Utxo>, HashMap<Utxo, RuneUTXOEntry>)> {
    let wallet_config = match tx_type {
        TransactionType::LegacyBitcoin => WalletConfig::for_p2pkh(),
        TransactionType::RuneTransfer(_)
        | TransactionType::TaprootBitcoin
        | TransactionType::OrdinalTransfer => WalletConfig::for_p2tr_raw_key(),
        TransactionType::RuneEtching(_) => WalletConfig::for_p2tr_script(),
        TransactionType::OrdinalInscription(_) => WalletConfig::for_p2tr_script(),
    };
    let (btc_utxos, rune_utxos) = get_tx_utxos(wallet_config, address.to_string(), tx_type).await?;

    ic_cdk::println!("[send_script_spend] Rune UTXOs = {:?}", rune_utxos);
    ic_cdk::println!("[send_script_spend] BTC UTXOs = {:?}", btc_utxos);

    return Ok((btc_utxos, rune_utxos));
}

#[cfg(feature = "canister")]
#[ic_cdk::query]
pub async fn get_rune_utxos(rune_id: RuneID) -> Vec<RuneUTXOEntry> {
    memory::stable::utxos::get_rune_utxos(&rune_id)
}

#[cfg(feature = "canister")]
#[ic_cdk::query]
pub async fn get_canister_rune_amount(rune_id: RuneID) -> Result<u64> {
    Ok(memory::stable::utxos::get_rune_utxos(&rune_id)
        .iter()
        .map(|r| r.rune_amount)
        .sum())
}

// --------
// END TEST
// --------

#[cfg(feature = "canister")]
#[ic_cdk::update]
pub async fn estimate_bitcoin_transaction_fee() -> Result<u64> {
    let fee_per_byte = wallet::get_fee_per_byte(
        NETWORK.with(|n| n.get()),
        BTC_PRINCIPAL.with(|bp| *bp.borrow()),
    )
    .await?;

    // Estimate transaction size in vBytes
    let estimated_size = 200; // max cut for P2TR spend transaction in vBytes
    let estimated_fee = estimated_size as u64 * fee_per_byte;

    ic_cdk::println!(
        "[estimate_bitcoin_transaction_fee] Estimated Fee: {} sats",
        estimated_fee
    );

    Ok(estimated_fee)
}

// ---------
// ADDRESSES
// ---------

/// Returns the P2PKH address of this canister at a specific derivation path.
#[cfg(feature = "canister")]
#[ic_cdk::update]
pub async fn get_p2pkh_address() -> Result<String> {
    wallet::p2pkh::get_address(WalletConfig::for_p2pkh())
        .await
        .map(|addr| addr.to_string())
}

/// Returns the P2TR address of this canister at a specific derivation path.
#[cfg(feature = "canister")]
#[ic_cdk::update]
pub async fn get_p2tr_raw_key_spend_address() -> Result<String> {
    wallet::p2tr_raw_key_spend::get_address(WalletConfig::for_p2tr_raw_key())
        .await
        .map(|addr| addr.to_string())
}

/// Returns the P2TR address of this canister at a specific derivation path.
/// Necessary for sending and receiving runes.
#[cfg(feature = "canister")]
#[ic_cdk::update]
pub async fn get_p2tr_script_spend_address(tx_type: TransactionType) -> Result<String> {
    wallet::p2tr_script_spend::get_address(WalletConfig::for_p2tr_script(), tx_type)
        .await
        .map(|addr| addr.0.to_string())
}

// -------
// CONFIGS
// -------

#[cfg(feature = "canister")]
#[ic_cdk::update]
async fn withdraw_bitcoin_fees(destination_address: Address, amount: u64) -> Result<String> {
    ic_cdk::println!(
        "[withdraw_bitcoin_fees] Withdrawing {} sats to {}",
        amount,
        destination_address
    );

    let fee_per_byte = wallet::get_fee_per_byte(
        NETWORK.with(|n| n.get()),
        BTC_PRINCIPAL.with(|bp| *bp.borrow()),
    )
    .await?;

    let estimated_size = 140;
    let estimated_fee = estimated_size as u64 * fee_per_byte;

    let net_amount = amount.saturating_sub(estimated_fee);
    if net_amount == 0 {
        return Err(BitcoinError::InvalidInput("Fees greater than amount".to_string()).into());
    }

    let tx_id = wallet::send::send_btc_or_ordinal(
        destination_address,
        net_amount,
        TransactionType::TaprootBitcoin,
    )
    .await?;

    ic_cdk::println!(
        "[withdraw_bitcoin_fees] Withdrawal completed. TX ID: {}",
        tx_id
    );

    Ok(tx_id.to_string())
}

#[cfg(feature = "canister")]
#[ic_cdk::query]
pub fn get_registered_runes() -> Result<Vec<RuneMetadata>> {
    memory::heap::config::get_registered_runes()
}

#[cfg(feature = "canister")]
#[ic_cdk::update]
pub fn register_runes(runes: Vec<RuneMetadata>) -> Result<()> {
    memory::heap::config::register_runes(runes)
}

#[cfg(feature = "canister")]
#[ic_cdk::query]
pub fn get_serialized_rune_metadata(rune_id: RuneID) -> Result<(RuneMetadata, String)> {
    let rune_metadata = memory::heap::config::get_rune_metadata(&rune_id)?;
    let serialized_metadata = rune_metadata.to_string();
    Ok((rune_metadata, serialized_metadata))
}

#[cfg(feature = "canister")]
#[ic_cdk::query]
pub fn validate_rune(rune_id: RuneID) -> Result<()> {
    memory::heap::config::is_rune_supported(&rune_id)?;

    Ok(())
}

// -----
// VAULT
// -----

#[cfg(feature = "canister")]
#[ic_cdk::query]
pub fn get_offramper_deposits(offramper: Address) -> Result<VaultEntry> {
    OFFRAMPER_VAULTS
        .with_borrow(|vaults| vaults.get(&offramper))
        .ok_or_else(|| VaultError::AddressVaultNotFound.into())
}

#[cfg(feature = "canister")]
#[ic_cdk::query]
pub fn get_onramper_deposits(onramper: Address) -> Result<VaultEntry> {
    ONRAMPER_VAULTS
        .with_borrow(|vaults| vaults.get(&onramper))
        .ok_or_else(|| VaultError::AddressVaultNotFound.into())
}

#[cfg(feature = "canister")]
#[ic_cdk::update]
pub fn deposit_to_address_vault(
    offramper: Address,
    amount: u64,
    rune: Option<RuneID>,
    utxos: Vec<RuneUTXOEntry>,
) -> Result<()> {
    if let Some(ref rune_id) = rune {
        memory::heap::config::is_rune_supported(rune_id)?;
    }

    vault::deposit::deposit_to_vault(offramper, amount, rune.clone())?;

    if let Some(rune_id) = rune {
        add_rune_utxo_entries(rune_id, utxos);
    }

    Ok(())
}

#[cfg(feature = "canister")]
#[ic_cdk::update]
pub async fn cancel_deposit(
    offramper: Address,
    amount: u64,
    rune: Option<RuneID>,
) -> Result<String> {
    let tx_type = if let Some(ref rune_id) = rune {
        memory::heap::config::is_rune_supported(&rune_id)?;
        TransactionType::RuneTransfer(rune_id.clone())
    } else {
        TransactionType::TaprootBitcoin
    };

    vault::deposit::cancel_deposit(offramper.clone(), amount, rune.clone())?;

    wallet::send::send_btc_or_ordinal(offramper, amount, tx_type)
        .await
        .map(|tx_id| tx_id.to_string())
}

#[cfg(feature = "canister")]
#[ic_cdk::update]
pub fn lock_funds(
    offramper: Address,
    onramper: Address,
    amount: u64,
    rune: Option<RuneID>,
) -> Result<()> {
    if let Some(ref rune_id) = rune {
        memory::heap::config::is_rune_supported(rune_id)?;
    }

    vault::lock::lock_funds(offramper, onramper, amount, rune)
}

#[cfg(feature = "canister")]
#[ic_cdk::update]
pub fn unlock_funds(
    offramper: Address,
    onramper: Address,
    amount: u64,
    rune: Option<RuneID>,
) -> Result<()> {
    if let Some(ref rune_id) = rune {
        memory::heap::config::is_rune_supported(rune_id)?;
    }

    vault::lock::unlock_funds(offramper, onramper, amount, rune)
}

#[cfg(feature = "canister")]
#[ic_cdk::update]
pub async fn complete_order_and_send(
    onramper_address: Address,
    amount: u64,
    rune: Option<RuneID>,
) -> Result<String> {
    let tx_type = if let Some(ref rune_id) = rune {
        memory::heap::config::is_rune_supported(&rune_id)?;
        TransactionType::RuneTransfer(rune_id.clone())
    } else {
        TransactionType::TaprootBitcoin
    };

    vault::complete::complete_order(onramper_address.clone(), amount, rune.clone())?;

    wallet::send::send_btc_or_ordinal(onramper_address, amount, tx_type)
        .await
        .map(|tx_id| tx_id.to_string())
}

// -----------
// INSCRIPTION
// -----------
#[cfg(feature = "canister")]
#[ic_cdk::update]
pub async fn inscribe_ordinals_inscription(
    content: String,
    content_type: String,
    metadata: Option<String>,
    dst_address: String,
    amount: u64,
) -> Result<String> {
    let inscription = Inscription {
        content,
        content_type,
        metadata,
    };
    let tx_id = inscription::inscribe_ordinal(
        WalletConfig::for_p2tr_script(),
        dst_address,
        amount,
        inscription,
    )
    .await?;

    Ok(tx_id.to_string())
}

#[cfg(feature = "canister")]
pub async fn send_ordinals_inscription(dst_address: String, amount: u64) -> Result<bitcoin::Txid> {
    wallet::send::send_btc_or_ordinal(dst_address, amount, TransactionType::OrdinalTransfer).await
}

#[cfg(feature = "canister")]
ic_cdk::export_candid!();
