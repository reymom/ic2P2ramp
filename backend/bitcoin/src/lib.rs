mod api;
pub mod memory;
mod model;
mod ordinals;
mod vault;
mod wallet;

use ic_btc_interface::{GetBlockHeadersResponse, Network, Utxo};
use icramp_types::bitcoin::{
    Address,
    errors::{BitcoinError, Result, VaultError},
    inscription::Inscription,
    runes::{RuneID, RuneMetadata, RuneUTXOEntry},
    setup::InstallArg,
    transfer::TransactionType,
    vault::VaultEntry,
};

use api::unisat::fetch_rune_utxos;
use memory::{
    heap::{
        config::{KEY_NAME, get_network, set_network},
        state::{State, get_state, initialize_state, read_state},
        upgrade,
    },
    stable::vault::{OFFRAMPER_VAULTS, ONRAMPER_VAULTS},
};
use model::types::wallet::WalletConfig;
use ordinals::inscription;

#[ic_cdk::pre_upgrade]
fn pre_upgrade() {
    upgrade::pre_upgrade()
}

#[ic_cdk::post_upgrade]
fn post_upgrade(install_arg: InstallArg) {
    ic_cdk::println!(
        "[post_upgrade]: upgrade canister executed with install_arg: {:?}",
        install_arg
    );

    match install_arg {
        InstallArg::Reinstall(_) => ic_cdk::trap("InitArg not valid for reinstall"),
        InstallArg::Upgrade(update_arg) => {
            upgrade::post_upgrade(update_arg.clone());
        }
    }

    let state = get_state();
    ic_cdk::println!("[post_upgrade]: state = {:?}", state);
}

#[ic_cdk::init]
pub fn init(install_arg: InstallArg) {
    match install_arg {
        InstallArg::Reinstall(init_arg) => {
            let network = init_arg.network;
            set_network(network);
            KEY_NAME.with(|key_name| {
                key_name.replace(String::from(match network {
                    Network::Regtest => "dfx_test_key",
                    Network::Mainnet | Network::Testnet => "test_key_1",
                }))
            });
            let state = State {
                btc_principal: candid::Principal::from_text(match network {
                    Network::Regtest | Network::Testnet => "g4xu7-jiaaa-aaaan-aaaaq-cai",
                    Network::Mainnet => "ghsi2-tqaaa-aaaan-aaaca-cai",
                })
                .expect("invalid bitcoin canister principal"),
                proxy_url: init_arg.proxy_url,
                unisat: init_arg.unisat,
            };
            ic_cdk::println!("[init]: state = {:?}", state);
            initialize_state(state);
        }
        InstallArg::Upgrade(_) => ic_cdk::trap("UpdateArg not valid for reinstall"),
    }
}

/// Returns the balance of the given bitcoin address.
#[ic_cdk::update]
pub async fn get_btc_balance(address: String) -> Result<u64> {
    let network = get_network();
    let btc_principal: candid::Principal = read_state(|s| s.btc_principal);
    api::bitcoin::get_balance(network, btc_principal, address).await
}

#[ic_cdk::update]
pub async fn get_btc_block_headers() -> Result<GetBlockHeadersResponse> {
    let network = get_network();
    let btc_principal = read_state(|s| s.btc_principal);
    api::bitcoin::get_block_headers(network, btc_principal, 0, Some(u32::MAX)).await
}

#[ic_cdk::update]
pub async fn transfer(
    dst_address: String,
    amount: u64,
    tx_type: TransactionType,
    rune_utxos: Option<Vec<RuneUTXOEntry>>,
) -> Result<String> {
    // If this is a rune transfer, validate the provided rune UTXOs.
    if let TransactionType::RuneTransfer(ref rune_id) = tx_type {
        memory::heap::config::is_rune_supported(rune_id)?;

        let utxos = rune_utxos
            .as_ref()
            .ok_or_else(|| BitcoinError::InvalidInput("Missing rune UTXOs".to_string()))?;
        let total_runes: u64 = utxos.iter().map(|u| u.rune_amount).sum();
        if total_runes != amount {
            return Err(BitcoinError::InvalidInput(format!(
                "Rune UTXOs total {} does not equal expected amount {}",
                total_runes, amount
            )));
        }

        let address = wallet::p2tr_raw_key_spend::get_address(WalletConfig::for_p2tr_raw_key())
            .await
            .map(|addr| addr.to_string())?;
        let fetched_utxos = fetch_rune_utxos(&address, rune_id.clone()).await?;

        ic_cdk::println!("[transfer] Provided rune UTXOs: {:?}", utxos);
        ic_cdk::println!("[transfer] Fetched rune UTXOs: {:?}", fetched_utxos);

        let provided_set: std::collections::HashSet<_> = utxos.iter().cloned().collect();
        let fetched_set: std::collections::HashSet<_> = fetched_utxos.into_iter().collect();
        if !provided_set.is_subset(&fetched_set) {
            return Err(BitcoinError::InvalidInput(
                "Provided rune UTXOs are not a subset of fetched UTXOs".to_string(),
            ));
        }
    }

    let tx_id = wallet::send::send_btc_or_ordinal(dst_address, amount, tx_type, rune_utxos).await?;
    Ok(tx_id.to_string())
}

#[ic_cdk::update]
pub async fn get_utxos(address: String) -> Result<Vec<Utxo>> {
    let btc_principal = read_state(|s| s.btc_principal);
    let network = get_network();
    api::bitcoin::get_utxos(network, btc_principal, address.to_string()).await
}

#[ic_cdk::query]
pub async fn get_canister_rune_utxos(rune_id: RuneID) -> Result<Vec<RuneUTXOEntry>> {
    let address = wallet::p2tr_raw_key_spend::get_address(WalletConfig::for_p2tr_raw_key())
        .await
        .map(|addr| addr.to_string())?;

    fetch_rune_utxos(&address, rune_id).await
}

#[ic_cdk::query]
pub async fn get_canister_rune_amount(rune_id: RuneID) -> Result<u64> {
    let address = wallet::p2tr_raw_key_spend::get_address(WalletConfig::for_p2tr_raw_key())
        .await
        .map(|addr| addr.to_string())?;

    Ok(fetch_rune_utxos(&address, rune_id)
        .await?
        .iter()
        .map(|r| r.rune_amount)
        .sum())
}

// --------
// END TEST
// --------

#[ic_cdk::update]
pub async fn estimate_bitcoin_transaction_fee() -> Result<u64> {
    let fee_per_byte =
        wallet::get_fee_per_byte(get_network(), read_state(|s| s.btc_principal)).await?;

    // Estimate transaction size in vBytes
    let estimated_size = 200; // max cut for P2TR spend transaction in vBytes
    let estimated_fee = estimated_size as u64 * fee_per_byte / 1000;

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

#[ic_cdk::update]
async fn withdraw_bitcoin_fees(destination_address: Address, amount: u64) -> Result<String> {
    ic_cdk::println!(
        "[withdraw_bitcoin_fees] Withdrawing {} sats to {}",
        amount,
        destination_address
    );

    let fee_per_byte =
        wallet::get_fee_per_byte(get_network(), read_state(|s| s.btc_principal)).await?;

    let estimated_size = 140;
    let estimated_fee = estimated_size as u64 * fee_per_byte / 1000;

    let net_amount = amount.saturating_sub(estimated_fee);
    if net_amount == 0 {
        return Err(BitcoinError::InvalidInput(
            "Fees greater than amount".to_string(),
        ));
    }

    let tx_id = wallet::send::send_btc_or_ordinal(
        destination_address,
        net_amount,
        TransactionType::TaprootBitcoin,
        None,
    )
    .await?;

    ic_cdk::println!(
        "[withdraw_bitcoin_fees] Withdrawal completed. TX ID: {}",
        tx_id
    );

    Ok(tx_id.to_string())
}

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
) -> Result<()> {
    if let Some(ref rune_id) = rune {
        memory::heap::config::is_rune_supported(rune_id)?;
    }

    vault::deposit::deposit_to_vault(offramper, amount, rune.clone())?;

    Ok(())
}

#[ic_cdk::update]
pub async fn cancel_deposit(offramper: Address, amount: u64, rune: Option<RuneID>) -> Result<()> {
    if let Some(ref rune_id) = rune {
        memory::heap::config::is_rune_supported(rune_id)?;
    }

    vault::deposit::cancel_deposit(offramper.clone(), amount, rune.clone())?;

    Ok(())
}

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

#[ic_cdk::update]
pub async fn complete_order(
    onramper_address: Address,
    amount: u64,
    rune: Option<RuneID>,
) -> Result<()> {
    if let Some(ref rune_id) = rune {
        memory::heap::config::is_rune_supported(rune_id)?;
    }

    vault::complete::complete_order(onramper_address.clone(), amount, rune.clone())?;

    Ok(())
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
    wallet::send::send_btc_or_ordinal(dst_address, amount, TransactionType::OrdinalTransfer, None)
        .await
}

ic_cdk::export_candid!();
