mod heap;
mod init;
pub mod logs;
mod state;
pub mod upgrade;

pub use heap::*;
pub use init::InitArg;
pub use state::*;
pub use upgrade::UpdateArg;

#[derive(candid::CandidType, candid::Deserialize, Debug)]
pub enum InstallArg {
    Reinstall(InitArg),
    Upgrade(Option<UpdateArg>),
}
