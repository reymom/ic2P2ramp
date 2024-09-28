use candid::CandidType;
use ethers_core::abi::Address;
use num_traits::ToPrimitive;

use super::rpc::LogEntry;
use crate::errors::{BlockchainError, Result, SystemError};

const TXS_THRESHOLD_DISCARD_BLOCKS: u128 = 30 * 7 * 24 * 60 * 5; // Assuming 5 blocks per minute

const DEPOSIT_EVENT_SIGNATURE: &str =
    "0x5548c837ab068cf56a2c2479df0882a4922fd203edb7517321831d95078c5f62";

#[derive(CandidType, Debug)]
pub struct DepositEvent {
    pub user: String,
    pub token: Option<String>,
    pub amount: u128,
    pub block: Option<u128>,
}

impl DepositEvent {
    fn new(user: &str, token: Option<String>, amount: u128, block: Option<u128>) -> Self {
        Self {
            user: user.to_string(),
            token,
            amount,
            block,
        }
    }

    pub fn expired(&self, current_block: u128) -> Result<()> {
        if let Some(event_block) = self.block {
            if current_block - event_block > TXS_THRESHOLD_DISCARD_BLOCKS {
                return Err(BlockchainError::EvmLogError("Log event expired".to_string()).into());
            }
        }
        Ok(())
    }
}

#[derive(CandidType, Debug)]
pub enum LogEvent {
    Deposit(DepositEvent),
}

/// topics[0]: The hashed event signature.
/// topics[1]: The offramper (user) address.
/// topics[2]: The token address (or zero for native ETH).
/// data: The amount transferred in uint256.
pub fn parse_deposit_event(log: &LogEntry) -> Result<LogEvent> {
    if log.topics.len() != 3 {
        return Err(BlockchainError::EvmLogError("Invalid number of topics".to_string()).into());
    }

    // Event signature: first topic, we compare it to the known Deposit event signature
    if log.topics[0] != DEPOSIT_EVENT_SIGNATURE {
        return Err(BlockchainError::EvmLogError("Not a Deposit event".to_string()).into());
    }

    // Indexed parameter 1: user address
    let user_address = format!("0x{}", &log.topics[1][26..]);

    // Indexed parameter 2: token address (in case it's native token, it's 0x0 address)
    let token_address = format!("0x{}", &log.topics[2][26..]);
    let token_address = if token_address == format!("{:#x}", Address::zero()) {
        None
    } else {
        Some(token_address)
    };

    // Non-indexed data: amount (hexadecimal to u128)
    let amount_hex = &log.data[2..]; // Remove the '0x' prefix
    let amount =
        u128::from_str_radix(amount_hex, 16).map_err(|e| SystemError::ParseError(e.to_string()))?;

    Ok(LogEvent::Deposit(DepositEvent::new(
        &user_address,
        token_address,
        amount,
        log.blockNumber.clone().and_then(|block| block.0.to_u128()),
    )))
}
