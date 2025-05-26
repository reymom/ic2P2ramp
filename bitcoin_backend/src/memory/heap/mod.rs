pub mod config;
pub mod init;
pub mod state;
pub mod upgrade;

pub use init::InitArg;
pub use upgrade::UpdateArg;

#[derive(candid::CandidType, candid::Deserialize, Debug)]
pub enum InstallArg {
    Reinstall(InitArg),
    Upgrade(Option<UpdateArg>),
}
