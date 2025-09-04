use std::{str::FromStr, time::Duration};

use bitcoin::address::Address as BitcoinAddress;
use candid::Principal;
use email_address::EmailAddress;
use ethers_core::types::{Address, H160};

use crate::{
    errors::{BlockchainError, Result},
    outcalls::pricing::rates,
};

use super::{
    errors::UserError,
    types::{AddressType, exchange_rate::RateAsset},
};

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
    if let Err(e) = rx.await {
        ic_cdk::println!("Delay timer error: {:?}", e);
    }
}

pub fn parse_address(address: String) -> Result<H160> {
    address.parse().map_err(|e| {
        BlockchainError::EthersAbiError(format!("Invalid address error: {:?}", e)).into()
    })
}

pub fn validate_evm_address(evm_address: &str) -> Result<()> {
    Address::from_str(evm_address)
        .map_err(|_| BlockchainError::InvalidAddress(AddressType::EVM))?;
    Ok(())
}

pub fn validate_icp_address(icp_address: &str) -> Result<()> {
    Principal::from_text(icp_address)
        .map_err(|_| BlockchainError::InvalidAddress(AddressType::ICP))?;
    Ok(())
}

pub fn validate_bitcoin_address(bitcoin_address: &str) -> Result<()> {
    BitcoinAddress::from_str(bitcoin_address)
        .map_err(|_| BlockchainError::InvalidAddress(AddressType::Bitcoin))?;
    Ok(())
}

pub fn validate_email(address: &str) -> Result<()> {
    if EmailAddress::is_valid(address) {
        Ok(())
    } else {
        Err(UserError::InvalidEmail.into())
    }
}

pub fn validate_solana_address(_solana_address: &str) -> Result<()> {
    // solana_sdk::pubkey::Pubkey::from_str(solana_address).map_err(|_| BlockchainError::InvalidAddress)?;
    Ok(())
}

pub async fn get_eth_token_rate(token_symbol: String) -> Result<f64> {
    let base_asset = RateAsset::Crypto {
        symbol: "ETH".to_string(),
    };
    let quote_asset = RateAsset::Crypto {
        symbol: token_symbol,
    };

    match rates::get_exchange_rate(base_asset, quote_asset).await {
        Ok(rate) => Ok(rate),
        Err(err) => Err(err),
    }
}

pub async fn get_btc_token_rate(rune_id: String) -> Result<f64> {
    let base_asset = RateAsset::Rune { name: rune_id };
    let quote_asset = RateAsset::Crypto {
        symbol: "BTC".to_string(),
    };

    match rates::get_exchange_rate(base_asset, quote_asset).await {
        Ok(rate) => Ok(rate),
        Err(err) => Err(err),
    }
}

pub async fn get_sol_token_rate(symbol: String, mint: Option<String>) -> Result<f64> {
    rates::get_exchange_rate(
        RateAsset::Solana {
            symbol: "SOL".to_string(),
            mint: None,
        },
        RateAsset::Solana { symbol, mint },
    )
    .await
    .map(Ok)?
}
