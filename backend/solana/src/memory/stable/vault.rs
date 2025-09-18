use crate::memory::stable::{MEMORY_MANAGER, Memory};
use ic_stable_structures::{StableBTreeMap, memory_manager::MemoryId};
use icramp_types::solana::vault::{Address, VaultEntry};
use std::cell::RefCell;

thread_local! {
    pub static OFFRAMPER_VAULTS: RefCell<StableBTreeMap<Address, VaultEntry, Memory>> =
        RefCell::new(
            StableBTreeMap::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0)))
            )
        );

    pub static ONRAMPER_VAULTS: RefCell<StableBTreeMap<Address, VaultEntry, Memory>> =
        RefCell::new(
            StableBTreeMap::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
            )
        );
}
