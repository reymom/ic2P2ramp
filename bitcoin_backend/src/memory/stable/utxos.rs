use std::cell::RefCell;

use ic_stable_structures::{memory_manager::MemoryId, StableBTreeMap};

use crate::model::types::{
    runes::RuneID,
    utxo::{RuneUTXOEntry, RuneUTXOList},
};

use super::{Memory, MEMORY_MANAGER};

thread_local! {
    pub static RUNE_UTXO_STORAGE: RefCell<StableBTreeMap<RuneID, RuneUTXOList, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2))),
        )
    );
}

pub fn list_rune_utxos() -> Vec<(RuneID, RuneUTXOEntry)> {
    RUNE_UTXO_STORAGE.with_borrow(|storage| {
        storage
            .iter()
            .map(|(rune_id, utxos)| {
                utxos
                    .0
                    .iter()
                    .map(|utxo| (rune_id.clone(), utxo.clone()))
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect()
    })
}

pub fn get_rune_utxos(rune_id: &RuneID) -> Vec<RuneUTXOEntry> {
    RUNE_UTXO_STORAGE.with_borrow(|storage| {
        storage
            .get(rune_id)
            .unwrap_or_else(|| RuneUTXOList(vec![]))
            .0
    })
}

pub fn add_rune_utxo_entries(rune_id: RuneID, utxos: Vec<RuneUTXOEntry>) {
    RUNE_UTXO_STORAGE.with_borrow_mut(|storage| {
        let mut existing_utxos = storage
            .get(&rune_id)
            .unwrap_or_else(|| RuneUTXOList(vec![]));
        existing_utxos.0.extend(utxos);
        storage.insert(rune_id, existing_utxos);
    });
}

pub fn remove_rune_utxo_entries(rune_id: &RuneID, utxos: Vec<RuneUTXOEntry>) {
    RUNE_UTXO_STORAGE.with_borrow_mut(|storage| {
        if let Some(mut existing_utxos) = storage.get(rune_id) {
            existing_utxos.0.retain(|utxo| {
                !utxos
                    .iter()
                    .any(|spent| spent.txid == utxo.txid && spent.vout == utxo.vout)
            });

            if existing_utxos.0.is_empty() {
                storage.remove(rune_id); // ✅ Remove entire entry if empty
            } else {
                storage.insert(rune_id.clone(), existing_utxos);
            }
        }
    });
}
