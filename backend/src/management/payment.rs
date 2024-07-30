use std::time::Duration;

use crate::{
    evm::{transaction, vault::Ic2P2ramp},
    model::errors::Result,
};

pub async fn handle_evm_payment_completion(
    order_id: u64,
    chain_id: u64,
    gas: Option<u32>,
) -> Result<()> {
    let tx_hash = Ic2P2ramp::release_funds(order_id, chain_id, gas).await?;
    transaction::spawn_transaction_checker(
        tx_hash,
        chain_id,
        60,
        Duration::from_secs(4),
        move || {
            // Update order state to completed
            match super::order::set_order_completed(order_id) {
                Ok(_) => {
                    ic_cdk::println!("[verify_transaction] order {:?} completed", order_id)
                }
                Err(e) => ic_cdk::trap(format!("could not complete order: {:?}", e).as_str()),
            }
        },
    );
    Ok(())
}
