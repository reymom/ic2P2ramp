mod api;
mod memory;
mod model;
mod ordinals;
mod vault;
mod wallet;

use ic_cdk::api::management_canister::bitcoin::{BitcoinNetwork, Utxo};
use std::collections::HashMap;

use errors::{Result, VaultError};
use memory::{
    heap::config::{KEY_NAME, NETWORK},
    stable::{
        utxos::add_rune_utxo_entries,
        vault::{OFFRAMPER_VAULTS, ONRAMPER_VAULTS},
    },
};
pub use model::types::{
    errors,
    runes::{RuneID, RuneMetadata},
    transfer::TransactionType,
    utxo::RuneUTXOEntry,
    vault::VaultEntry,
    wallet::WalletConfig,
    Address,
};
use ordinals::inscription::{self, Inscription};
use wallet::utxos::get_tx_utxos;

#[ic_cdk::init]
pub fn init(network: BitcoinNetwork) {
    NETWORK.with(|n| n.set(network));

    KEY_NAME.with(|key_name| {
        key_name.replace(String::from(match network {
            BitcoinNetwork::Regtest => "dfx_test_key",
            BitcoinNetwork::Mainnet | BitcoinNetwork::Testnet => "test_key_1",
        }))
    });
}

// ----
// TEST
// ----

/// Returns the balance of the given bitcoin address.
#[ic_cdk::update]
pub async fn get_btc_balance(address: String) -> Result<u64> {
    let network = NETWORK.with(|n| n.get());
    api::bitcoin::get_balance(network, address).await
}

#[ic_cdk::update]
pub async fn test_transfer(
    dst_address: String,
    amount: u64,
    tx_type: TransactionType,
) -> Result<String> {
    let tx_id = wallet::send::send_btc_or_ordinal(dst_address, amount, tx_type).await?;
    Ok(tx_id.to_string())
}

#[ic_cdk::update]
pub async fn get_utxos(
    address: String,
    tx_type: TransactionType,
) -> Result<(Vec<Utxo>, HashMap<Utxo, RuneUTXOEntry>)> {
    let (btc_utxos, rune_utxos) = get_tx_utxos(
        tx_type.clone().get_wallet_config(),
        address.to_string(),
        tx_type,
    )
    .await?;

    ic_cdk::println!("[send_script_spend] Rune UTXOs = {:?}", rune_utxos);
    ic_cdk::println!("[send_script_spend] BTC UTXOs = {:?}", btc_utxos);

    return Ok((btc_utxos, rune_utxos));
}

#[ic_cdk::query]
pub async fn get_rune_utxos(rune_id: RuneID) -> Vec<RuneUTXOEntry> {
    memory::stable::utxos::get_rune_utxos(&rune_id)
}

// --------
// END TEST
// --------

// ---------
// ADDRESSES
// ---------

/// Returns the P2PKH address of this canister at a specific derivation path.
#[ic_cdk::update]
pub async fn get_p2pkh_address() -> Result<String> {
    wallet::p2pkh::get_address(WalletConfig::for_p2pkh())
        .await
        .map(|addr| addr.to_string())
}

/// Returns the P2TR address of this canister at a specific derivation path.
#[ic_cdk::update]
pub async fn get_p2tr_raw_key_spend_address() -> Result<String> {
    wallet::p2tr_raw_key_spend::get_address(WalletConfig::for_p2tr_raw_key())
        .await
        .map(|addr| addr.to_string())
}

/// Returns the P2TR address of this canister at a specific derivation path.
/// Necessary for sending and receiving runes.
#[ic_cdk::update]
pub async fn get_p2tr_script_spend_address(tx_type: TransactionType) -> Result<String> {
    wallet::p2tr_script_spend::get_address(WalletConfig::for_p2tr_script(), tx_type)
        .await
        .map(|addr| addr.0.to_string())
}

// -------
// CONFIGS
// -------

#[ic_cdk::query]
pub fn get_registered_runes() -> Result<Vec<RuneMetadata>> {
    memory::heap::config::get_registered_runes()
}

#[ic_cdk::update]
pub fn register_runes(runes: Vec<RuneMetadata>) -> Result<()> {
    memory::heap::config::register_runes(runes)
}

#[ic_cdk::query]
pub fn get_serialized_rune_metadata(rune_id: RuneID) -> Result<(RuneMetadata, String)> {
    let rune_metadata = memory::heap::config::get_rune_metadata(&rune_id)?;
    let serialized_metadata = rune_metadata.to_string();
    Ok((rune_metadata, serialized_metadata))
}

#[ic_cdk::query]
pub fn validate_rune(rune_id: RuneID) -> Result<()> {
    memory::heap::config::is_rune_supported(&rune_id)?;

    Ok(())
}

// -----
// VAULT
// -----

#[ic_cdk::query]
pub fn get_offramper_deposits(offramper: Address) -> Result<VaultEntry> {
    OFFRAMPER_VAULTS
        .with_borrow(|vaults| vaults.get(&offramper))
        .ok_or_else(|| VaultError::AddressVaultNotFound.into())
}

#[ic_cdk::query]
pub fn get_onramper_deposits(onramper: Address) -> Result<VaultEntry> {
    ONRAMPER_VAULTS
        .with_borrow(|vaults| vaults.get(&onramper))
        .ok_or_else(|| VaultError::AddressVaultNotFound.into())
}

#[ic_cdk::update]
pub fn deposit_to_address_vault(
    offramper: Address,
    amount: u64,
    rune: Option<RuneID>,
    utxos: Vec<RuneUTXOEntry>,
) -> Result<()> {
    vault::deposit::deposit_to_vault(offramper, amount, rune.clone())?;

    if let Some(rune_id) = rune {
        add_rune_utxo_entries(rune_id, utxos);
    }

    Ok(())
}

// TODO: do the transfer to the offramper here
#[ic_cdk::update]
pub fn cancel_deposit(offramper: Address, amount: u64, rune: Option<RuneID>) -> Result<()> {
    vault::deposit::cancel_deposit(offramper, amount, rune)

    // TODO: transfer to offramper
}

#[ic_cdk::update]
pub fn lock_funds(
    offramper: Address,
    onramper: Address,
    amount: u64,
    rune: Option<RuneID>,
) -> Result<()> {
    vault::lock::lock_funds(offramper, onramper, amount, rune)
}

#[ic_cdk::update]
pub fn unlock_funds(
    offramper: Address,
    onramper: Address,
    amount: u64,
    runes: Option<RuneID>,
) -> Result<()> {
    vault::lock::unlock_funds(offramper, onramper, amount, runes)
}

#[ic_cdk::update]
pub async fn complete_order_and_send(
    onramper_address: Address,
    amount: u64,
    tx_type: TransactionType,
) -> Result<String> {
    // Send Bitcoin or Runes
    let tx_id =
        wallet::send::send_btc_or_ordinal(onramper_address.clone(), amount, tx_type).await?;

    // Clear the locked funds in the vault
    // vault::complete::complete_order(onramper_address, amount, rune)?;

    Ok(tx_id.to_string())
}

// -----------
// INSCRIPTION
// -----------
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

pub async fn send_ordinals_inscription(dst_address: String, amount: u64) -> Result<bitcoin::Txid> {
    wallet::send::send_btc_or_ordinal(dst_address, amount, TransactionType::OrdinalTransfer).await
}

ic_cdk::export_candid!();
