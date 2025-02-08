pub mod setup;
pub mod helpers;

pub fn setup_pic() -> pocket_ic::PocketIc {
    pocket_ic::PocketIcBuilder::new()
        .with_bitcoin_subnet()
        .with_application_subnet()
        .build()
}