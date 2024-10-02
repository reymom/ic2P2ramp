mod init;
pub mod logs;
mod rate;
mod state;
mod storage;
pub mod upgrade;

pub use init::InitArg;
pub use rate::*;
pub use state::*;
pub use storage::*;
pub use upgrade::UpdateArg;

#[derive(candid::CandidType, candid::Deserialize, Debug)]
pub enum InstallArg {
    Reinstall(InitArg),
    Upgrade(Option<UpdateArg>),
}

pub fn setup_timers() {
    ic_cdk_timers::set_timer(std::time::Duration::ZERO, || {
        ic_cdk::spawn(async {
            let public_key = crate::evm::signer::get_public_key().await;
            let evm_address = crate::evm::signer::pubkey_bytes_to_address(&public_key);
            ic_cdk::println!("[setup_timers] evm_address = {}", evm_address);
            mutate_state(|s| {
                s.ecdsa_pub_key = Some(public_key);
                s.evm_address = Some(evm_address);
            });
        })
    });
}
