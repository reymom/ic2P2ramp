use candid::CandidType;
use ethers_core::abi::Address;
use evm_rpc_canister_types::LogEntry;
use num_traits::ToPrimitive;

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

    pub fn expired(&self, current_block: candid::Nat) -> Result<()> {
        if let Some(event_block) = self.block {
            if current_block - event_block > TXS_THRESHOLD_DISCARD_BLOCKS {
                return Err(BlockchainError::EvmLogError("Log event expired".to_string()).into());
            }
        }
        Ok(())
    }
}

/// Represents different types of log events that can occur.
/// In this case, we are only handling the `Deposit` event.
#[derive(CandidType, Debug)]
pub enum LogEvent {
    Deposit(DepositEvent),
}

/// Parses a `LogEntry` from an Ethereum transaction log and attempts to extract
/// a `Deposit` event if the log matches the expected event signature and format.
///
/// The expected event signature corresponds to the following Solidity event:
///
/// ```solidity
/// event Deposit(address indexed user, address indexed token, uint256 amount);
/// ```
///
/// ## Topics:
///
/// - `topics[0]`: The hashed event signature for the `Deposit` event. This is a unique
///   identifier for the event derived from its name and parameters.
/// - `topics[1]`: The `offramper` (user) address involved in the deposit.
///   This is a 32-byte hex string that represents the Ethereum address of the user.
/// - `topics[2]`: The `token` address involved in the transaction. If this is a native
///   token like Ether (ETH), this address will be `0x0000000000000000000000000000000000000000`.
///
/// ## Data:
///
/// - `data`: A non-indexed parameter that represents the amount of the deposit in `uint256`.
///
/// ## Parameters:
///
/// - `log`: A `LogEntry` that contains the event data emitted by the Ethereum blockchain.
///
/// ## Returns:
///
/// Returns a `Result` containing the parsed `LogEvent::Deposit` event or an error if the log entry
/// does not match the expected format.
///
/// ## Errors:
///
/// - Returns `BlockchainError::EvmLogError` if:
///   - The log does not contain exactly 3 topics.
///   - The event signature does not match the expected `Deposit` event signature.
///   - There is an issue parsing the log data (e.g., invalid hexadecimal format).
///
/// ## Example Usage:
///
/// ```rust
/// let log_event = parse_deposit_event(&log_entry)?;
/// match log_event {
///     LogEvent::Deposit(deposit) => {
///         println!("Deposit Event: {:?}", deposit);
///     }
/// }
/// ```
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
