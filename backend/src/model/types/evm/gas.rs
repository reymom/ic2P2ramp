use candid::{CandidType, Deserialize};
use num_traits::ToPrimitive;

use crate::model::{
    errors::{BlockchainError, Result},
    memory::heap::{mutate_state, read_state},
};

use super::transaction::{TransactionAction, TransactionVariant};

const DEFAULT_PAST_DAY_BLOCKS: u64 = (24 * 60 * 60) / 12; // assuming 12 seconds per block

/// Represents a record of gas usage in a particular transaction on a blockchain.
/// This includes gas consumption, the gas price at the time, and the block number the transaction occurred in.
#[derive(Clone, Debug, Default, CandidType, Deserialize)]
pub struct GasRecord {
    gas: u64,
    gas_price: u128,
    block_number: u128,
}

/// Maintains a list of gas usage records for a specific action type (e.g., Commit, Release).
#[derive(Clone, Debug, Default, CandidType, Deserialize)]
pub struct GasUsage {
    records: Vec<GasRecord>,
}

/// Holds gas usage information for different transaction actions (commit, release native, release token)
/// on a specific chain.
#[derive(Clone, Debug, Default, CandidType, Deserialize)]
pub struct ChainGasTracking {
    pub commit_gas: GasUsage,
    pub uncommit_gas: GasUsage,
    pub cancel_token_gas: GasUsage,
    pub cancel_native_gas: GasUsage,
    pub release_token_gas: GasUsage,
    pub release_native_gas: GasUsage,
}

impl GasUsage {
    /// Records the gas usage for a particular transaction, saving the gas, gas price, and block number.
    pub fn record_gas_usage(&mut self, gas: u64, gas_price: u128, block_number: u128) {
        self.records.push(GasRecord {
            gas,
            gas_price,
            block_number,
        });
    }

    /// Computes the average gas usage and gas price for recent transactions based on the block number range.
    ///
    /// Parameters:
    /// - `current_block`: The current block number.
    /// - `max_blocks_in_past`: Maximum number of blocks in the past to consider for the average.
    ///
    /// Returns:
    /// - A tuple containing the average gas usage and the average gas price for transactions within the specified block range,
    ///   or `None` if no records are found.
    pub fn average_gas(
        &self,
        current_block: Option<u128>,
        max_blocks_in_past: u64,
    ) -> Option<(u64, u128)> {
        let current_block = match current_block {
            None => return None,
            Some(block) => block,
        };

        let relevant_records: Vec<_> = self
            .records
            .iter()
            .filter(|record| {
                current_block.saturating_sub(record.block_number) <= max_blocks_in_past as u128
            })
            .collect();

        if relevant_records.is_empty() {
            return None;
        }

        let length = relevant_records.len() as u128;

        let (total_gas, total_gas_price) =
            relevant_records
                .iter()
                .fold((0u128, 0u128), |(sum_gas, sum_gas_price), record| {
                    (
                        sum_gas + record.gas as u128,
                        sum_gas_price + record.gas_price,
                    )
                });

        Some(((total_gas / length) as u64, total_gas_price / length))
    }
}

/// Registers the gas usage for a specific chain and transaction action (e.g., Commit, Release).
///
/// Parameters:
/// - `chain_id`: The chain identifier.
/// - `gas`: Gas consumed by the transaction.
/// - `gas_price`: Gas price at the time of the transaction.
/// - `block_number`: Block number in which the transaction was included.
/// - `action_type`: The type of transaction action (e.g., Commit, Release).
///
/// Returns:
/// - `Result<()>`: Returns an error if the chain ID is not found.
pub fn register_gas_usage(
    chain_id: u64,
    gas: u64,
    gas_price: u128,
    block_number: u128,
    action_type: &TransactionAction,
) -> Result<()> {
    mutate_state(|state| {
        let chain_state = state
            .chains
            .get_mut(&chain_id)
            .ok_or(BlockchainError::ChainIdNotFound(chain_id))?;

        match action_type {
            TransactionAction::Commit => {
                let gas_tracking = &mut chain_state.gas_tracking.commit_gas;
                gas_tracking.record_gas_usage(gas, gas_price, block_number);
            }
            TransactionAction::Uncommit => {
                let gas_tracking = &mut chain_state.gas_tracking.uncommit_gas;
                gas_tracking.record_gas_usage(gas, gas_price, block_number);
            }
            TransactionAction::Release(TransactionVariant::Token) => {
                let gas_tracking = &mut chain_state.gas_tracking.release_token_gas;
                gas_tracking.record_gas_usage(gas, gas_price, block_number);
            }
            TransactionAction::Release(TransactionVariant::Native) => {
                let gas_tracking = &mut chain_state.gas_tracking.release_native_gas;
                gas_tracking.record_gas_usage(gas, gas_price, block_number);
            }
            _ => (),
        };

        Ok(())
    })
}

/// Retrieves the average gas usage and gas price for recent transactions based on the action type (Commit, Release).
///
/// Parameters:
/// - `chain_id`: The chain identifier.
/// - `current_block`: The current block number.
/// - `max_blocks_in_past`: The maximum number of past blocks to consider for the average.
/// - `action_type`: The transaction action to filter (Commit, Release).
///
/// Returns:
/// - `Result<Option<(u128, u128)>>`: A tuple with average gas and gas price, or `None` if no data is found.
pub fn get_average_gas(
    chain_id: u64,
    current_block: candid::Nat,
    max_blocks_in_past: Option<u64>,
    action_type: &TransactionAction,
) -> Result<Option<(u64, u128)>> {
    let max_blocks_in_past = if let Some(blocks) = max_blocks_in_past {
        blocks
    } else {
        DEFAULT_PAST_DAY_BLOCKS
    };

    read_state(|state| {
        let chain_state = state
            .chains
            .get(&chain_id)
            .ok_or(BlockchainError::ChainIdNotFound(chain_id))?;

        let gas_tracking = match action_type {
            TransactionAction::Commit => Ok(&chain_state.gas_tracking.commit_gas),
            TransactionAction::Uncommit => Ok(&chain_state.gas_tracking.uncommit_gas),
            TransactionAction::Cancel(TransactionVariant::Native) => {
                Ok(&chain_state.gas_tracking.cancel_native_gas)
            }
            TransactionAction::Cancel(TransactionVariant::Token) => {
                Ok(&chain_state.gas_tracking.cancel_token_gas)
            }
            TransactionAction::Release(TransactionVariant::Token) => {
                Ok(&chain_state.gas_tracking.release_token_gas)
            }
            TransactionAction::Release(TransactionVariant::Native) => {
                Ok(&chain_state.gas_tracking.release_native_gas)
            }
            _ => Err(BlockchainError::GasLogError(
                "Action is not being logged".into(),
            )),
        }?;

        Ok(gas_tracking.average_gas(current_block.0.to_u128(), max_blocks_in_past))
    })
}

/// Retrieves the entire gas tracking data for a given chain.
///
/// Parameters:
/// - `chain_id`: The chain identifier.
///
/// Returns:
/// - `Result<ChainGasTracking>`: A `ChainGasTracking` object for the specified chain.
pub fn get_gas_tracking(chain_id: u64) -> Result<ChainGasTracking> {
    read_state(|state| {
        Ok(state
            .chains
            .get(&chain_id)
            .ok_or(BlockchainError::ChainIdNotFound(chain_id))?
            .gas_tracking
            .clone())
    })
}
