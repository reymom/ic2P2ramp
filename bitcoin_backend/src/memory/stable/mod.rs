pub mod utxos;
pub mod vault;

use ic_stable_structures::{
    memory_manager::{MemoryManager, VirtualMemory},
    DefaultMemoryImpl,
};

pub type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    static MEMORY_MANAGER: std::cell::RefCell<MemoryManager<DefaultMemoryImpl>> =
        std::cell::RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

}
