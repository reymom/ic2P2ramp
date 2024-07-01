use std::str::FromStr;
use std::time::Duration;

use ethers_core::types::{Address, H160};

use crate::errors::{RampError, Result};

/// Introduces an asynchronous delay for the specified duration.
///
/// This function leverages the `ic_cdk_timers::set_timer` function to create a delay. The `set_timer`
/// function schedules a task to be executed after a specified duration. Here, the task sends a signal
/// through a one-shot channel once the delay period has passed. The function then awaits the receipt
/// of this signal, effectively causing an asynchronous delay.
///
/// This approach is preferable in an asynchronous context compared to using blocking functions like
/// `std::thread::sleep`, as it avoids blocking the entire thread, allowing other asynchronous tasks
/// to progress.
///
/// # Parameters
/// - `duration`: The amount of time to delay. This is specified as a `std::time::Duration`.
///
/// # Example
/// ```rust
/// async fn example_usage() {
///     println!("Starting delay...");
///     delay(Duration::from_secs(2)).await;
///     println!("Delay finished.");
/// }
/// ```
pub async fn delay(duration: Duration) {
    let (tx, rx) = futures::channel::oneshot::channel();
    ic_cdk_timers::set_timer(duration, move || {
        let _ = tx.send(());
    });
    rx.await.unwrap();
}

pub fn parse_address(address: String) -> Result<H160> {
    address
        .parse()
        .map_err(|e| RampError::EthersAbiError(format!("Invalid address error: {:?}", e)))
}

pub fn validate_evm_address(evm_address: &str) -> Result<()> {
    Address::from_str(evm_address).map_err(|_| RampError::InvalidAddress)?;
    Ok(())
}
