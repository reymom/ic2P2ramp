use candid::CandidType;
use ethers_core::abi::Address;
use evm_rpc_canister_types::LogEntry;
use num_traits::ToPrimitive;

use crate::errors::{BlockchainError, Result, SystemError};

const TXS_THRESHOLD_DISCARD_BLOCKS: u128 = 30 * 7 * 24 * 60 * 5; // Assuming 5 blocks per minute

const DEPOSIT_EVENT_SIGNATURE: &str =
    "0x5548c837ab068cf56a2c2479df0882a4922fd203edb7517321831d95078c5f62";

const TRANSFER_EVENT_SIGNATURE: &str =
    "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";

#[derive(CandidType, Debug)]
pub struct DepositEvent {
    pub user: String,
    pub token: Option<String>,
    pub amount: u128,
    pub block: Option<u128>,
}

#[derive(CandidType, Debug)]
pub struct TransferEvent {
    pub from: String,
    pub to: String,
    pub value: u128,
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
    Transfer(TransferEvent),
}

/// Parses a `LogEntry` from an Ethereum transaction log and attempts to extract
/// a `Deposit` or `Transfer` event if the log matches the expected event signature and format.
///
/// The expected event signatures corresponds to the following Solidity events:
///
/// ```solidity
/// event Deposit(address indexed user, address indexed token, uint256 amount);
/// event Transfer(address indexed from, address indexed to, uint256 value)
/// ```
///
/// ## Topics:
///
/// - `topics[0]`: The hashed event signature for the event. Unique
///   identifier for the event derived from its name and parameters.
/// - `topics[1]`: The `offramper`/`from` address involved in the deposit/transfer.
///   This is a 32-byte hex string that represents the Ethereum address of the user.
/// - `topics[2]`: The `token` address involved in the deposit transaction
///   or the `to` address involved in the transfer transaction. If this is a native
///   token like Ether (ETH), this address will be `0x0000000000000000000000000000000000000000`.
///
/// ## Data:
///
/// - `data`: A non-indexed parameter that represents the amount of the deposit/value in `uint256`.
///
/// ## Parameters:
///
/// - `log`: A `LogEntry` that contains the event data emitted by the Ethereum blockchain.
///
/// ## Returns:
///
/// Returns a `Result` containing the parsed `LogEvent::Deposit` or `LogEvent::Transfer` event
/// or an error if the log entry does not match the expected format.
///
/// ## Errors:
///
/// - Returns `BlockchainError::EvmLogError` if:
///   - The log does not contain exactly 3 topics.
///   - The event signature does not match the expected `Deposit` or `Transfer` event signatures.
///   - There is an issue parsing the log data (e.g., invalid hexadecimal format).
///
/// ## Example Usage:
///
/// ```rust
/// let log_event = parse_log_event(&log_entry)?;
/// match log_event {
///     LogEvent::Deposit(deposit) => {
///         println!("Deposit Event: {:?}", deposit);
///     }
///     LogEvent::Transfer(value) => {
///         println!("Transfer Event: {:?}", value);
///     }
/// }
/// ```
pub fn parse_log_event(log: &LogEntry) -> Result<LogEvent> {
    if log.topics.len() != 3 {
        return Err(BlockchainError::EvmLogError("Invalid number of topics".to_string()).into());
    }

    // ---- Deposit event ----
    if log.topics.len() == 3 && log.topics[0] == DEPOSIT_EVENT_SIGNATURE {
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
        let amount = u128::from_str_radix(amount_hex, 16)
            .map_err(|e| SystemError::ParseError(e.to_string()))?;

        return Ok(LogEvent::Deposit(DepositEvent::new(
            &user_address,
            token_address,
            amount,
            log.blockNumber.clone().and_then(|block| block.0.to_u128()),
        )));
    }

    // ---- ERC20 Transfer event ----
    if log.topics.len() == 3 && log.topics[0] == TRANSFER_EVENT_SIGNATURE {
        let from = format!("0x{}", &log.topics[1][26..]);
        let to = format!("0x{}", &log.topics[2][26..]);

        let value_hex = &log.data[2..];
        let value = u128::from_str_radix(value_hex, 16)
            .map_err(|e| SystemError::ParseError(e.to_string()))?;

        return Ok(LogEvent::Transfer(TransferEvent {
            from,
            to,
            value,
            block: log.blockNumber.clone().and_then(|b| b.0.to_u128()),
        }));
    }

    Err(BlockchainError::EvmLogError("Unsupported event".into()).into())
}
