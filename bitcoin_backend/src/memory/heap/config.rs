use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
};

use ic_btc_interface::Network;

use crate::model::types::{
    errors::{BitcoinError, Result},
    runes::{RuneID, RuneMetadata},
};

thread_local! {
    // The bitcoin network to connect to.
    //
    // When developing locally this should be `Regtest`.
    // When deploying to the IC this should be `Testnet`.
    // `Mainnet` is currently unsupported.
    pub static NETWORK: Cell<Network> = Cell::new(Network::Testnet);

    // The derivation path to use for the threshold key.
    pub static DERIVATION_PATH: Vec<Vec<u8>> = vec![];

    // The ECDSA key name.
    pub static KEY_NAME: RefCell<String> = RefCell::new(String::from(""));

    // The canister address.
    pub static BTC_PRINCIPAL: RefCell<candid::Principal> = RefCell::new(candid::Principal::anonymous());

    // Registered Runes: rune ID is BLOCK:TX
    static RUNES: RefCell<HashMap<RuneID, RuneMetadata>> = RefCell::new(HashMap::new());
}

pub fn get_registered_runes() -> Result<Vec<RuneMetadata>> {
    Ok(RUNES
        .with(|runes| runes.borrow().clone())
        .into_values()
        .collect())
}

pub fn register_runes(rune_list: Vec<RuneMetadata>) -> Result<()> {
    RUNES.with_borrow_mut(|runes| {
        for rune in rune_list {
            rune.id.validate()?;
            runes.insert(rune.id.clone(), rune);
        }
        Ok(())
    })
}

pub fn is_rune_supported(rune_id: &RuneID) -> Result<()> {
    rune_id.validate()?;
    RUNES.with_borrow(|runes| {
        if !runes.contains_key(rune_id) {
            Err(BitcoinError::UnsupportedRune(rune_id.to_string()))
        } else {
            Ok(())
        }
    })
}

pub fn get_rune_metadata(rune_id: &RuneID) -> Result<RuneMetadata> {
    rune_id.validate()?;
    let runes = RUNES.with(|runes| runes.borrow().clone());
    runes
        .get(rune_id)
        .cloned()
        .ok_or_else(|| BitcoinError::UnsupportedRune(rune_id.to_string()))
}
