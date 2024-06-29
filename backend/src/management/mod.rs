use ethers_core::types::Address;
use std::str::FromStr;

use crate::errors::{RampError, Result};

pub mod order;
pub mod user;

pub fn validate_evm_address(evm_address: &str) -> Result<()> {
    Address::from_str(evm_address).map_err(|_| RampError::InvalidAddress)?;
    Ok(())
}
