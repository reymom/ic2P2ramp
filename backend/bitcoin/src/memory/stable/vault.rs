use std::cell::RefCell;

use ic_stable_structures::{memory_manager::MemoryId, StableBTreeMap};

use crate::model::types::{vault::VaultEntry, Address};

use super::{Memory, MEMORY_MANAGER};

thread_local! {
    pub static OFFRAMPER_VAULTS: RefCell<StableBTreeMap<Address, VaultEntry, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
        )
    );

    pub static ONRAMPER_VAULTS: RefCell<StableBTreeMap<Address, VaultEntry, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))),
        )
    );
}
