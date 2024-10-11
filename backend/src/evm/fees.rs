use std::ops::{Add, Div, Mul};

use candid::Nat;
use ethers_core::types::U256;
use evm_rpc_canister_types::{
    Block, BlockTag, FeeHistory, FeeHistoryArgs, FeeHistoryResult, GetBlockByNumberResult,
    MultiFeeHistoryResult, MultiGetBlockByNumberResult,
};
use serde_bytes::ByteBuf;

use super::{helper::nat_to_u256, rpc::EVM_RPC};
use crate::{
    errors::{Result, SystemError},
    types::evm::chains,
};

#[derive(Clone, Debug)]
pub struct FeeEstimates {
    pub max_fee_per_gas: U256,
    pub max_priority_fee_per_gas: U256,
}

const MIN_SUGGEST_MAX_PRIORITY_FEE_PER_GAS: u32 = 1_500_000_000;

pub(super) async fn fee_history(
    chain_id: u64,
    block_count: Nat,
    newest_block: BlockTag,
    reward_percentiles: Option<Vec<u8>>,
) -> Result<FeeHistory> {
    let rpc_providers = chains::get_rpc_providers(chain_id)?;

    let fee_history_args: FeeHistoryArgs = FeeHistoryArgs {
        blockCount: block_count,
        newestBlock: newest_block,
        rewardPercentiles: reward_percentiles.map(ByteBuf::from),
    };

    let cycles = 10_000_000_000;

    match EVM_RPC
        .eth_fee_history(rpc_providers, None, fee_history_args, cycles)
        .await
    {
        Ok((res,)) => match res {
            MultiFeeHistoryResult::Consistent(fee_history) => match fee_history {
                FeeHistoryResult::Ok(fee_history) => fee_history.ok_or_else(|| {
                    SystemError::InternalError("Could not find fee history".to_string()).into()
                }),
                FeeHistoryResult::Err(e) => Err(SystemError::RpcError(format!("{:?}", e)))?,
            },
            MultiFeeHistoryResult::Inconsistent(_) => Err(SystemError::InternalError(
                "Fee history is inconsistent".to_string(),
            ))?,
        },
        Err((code, msg)) => Err(SystemError::ICRejectionError(code, msg))?,
    }
}

fn median_index(length: usize) -> usize {
    if length == 0 {
        panic!("Cannot find a median index for an array of length zero.");
    }
    (length - 1) / 2
}

pub async fn get_fee_estimates(block_count: u8, chain_id: u64) -> Result<FeeEstimates> {
    // Can be taken dynamically from proxy contract 0x420000000000000000000000000000000000000F
    if chain_id == 5003 || chain_id == 5000 {
        return Ok(FeeEstimates {
            max_fee_per_gas: 20000000.into(),
            max_priority_fee_per_gas: 0.into(),
        });
    }

    // we are setting the `max_priority_fee_per_gas` based on this article:
    // https://docs.alchemy.com/docs/maxpriorityfeepergas-vs-maxfeepergas
    // following this logic, the base fee will be derived from the block history automatically
    // and we only specify the maximum priority fee per gas (tip).
    // the tip is derived from the fee history of the last 9 blocks, more specifically
    // from the 95th percentile of the tip.
    let fee_history = fee_history(
        chain_id,
        Nat::from(block_count),
        BlockTag::Latest,
        Some(vec![95]),
    )
    .await?;

    let median_index = median_index(block_count.into());

    // baseFeePerGas
    let base_fee_per_gas = fee_history
        .baseFeePerGas
        .last()
        .ok_or_else(|| SystemError::InternalError("baseFeePerGas is empty".to_string()))?
        .clone();

    // obtain the 95th percentile of the tips for the past 9 blocks
    let mut percentile_95: Vec<Nat> = fee_history
        .reward
        .into_iter()
        .flat_map(|x| x.into_iter())
        .collect();
    // sort the tips in ascending order
    percentile_95.sort_unstable();
    // get the median by accessing the element in the middle
    // set tip to 0 if there are not enough blocks in case of a local testnet
    let median_reward = percentile_95
        .get(median_index)
        .unwrap_or(&Nat::from(0_u8))
        .clone();

    let max_priority_fee_per_gas = median_reward
        .clone()
        .max(Nat::from(MIN_SUGGEST_MAX_PRIORITY_FEE_PER_GAS));

    let max_fee_per_gas = base_fee_per_gas
        .clone()
        .add(max_priority_fee_per_gas.clone())
        .max(base_fee_per_gas)
        .mul(Nat::from(105u8))
        .div(Nat::from(100u8)); // Adding a cushion buffer

    Ok(FeeEstimates {
        max_fee_per_gas: nat_to_u256(&max_fee_per_gas),
        max_priority_fee_per_gas: nat_to_u256(&max_priority_fee_per_gas),
    })
}

pub async fn eth_get_latest_block(chain_id: u64, block_tag: BlockTag) -> Result<Block> {
    let rpc_providers = chains::get_rpc_providers(chain_id)?;

    let cycles = 10_000_000_000;
    match EVM_RPC
        .eth_get_block_by_number(rpc_providers, None, block_tag, cycles)
        .await
    {
        Ok((res,)) => match res {
            MultiGetBlockByNumberResult::Consistent(block_result) => match block_result {
                GetBlockByNumberResult::Ok(block) => Ok(block),
                GetBlockByNumberResult::Err(e) => Err(SystemError::RpcError(format!("{:?}", e)))?,
            },
            MultiGetBlockByNumberResult::Inconsistent(_) => Err(SystemError::InternalError(
                "Block Result is inconsistent".to_string(),
            ))?,
        },
        Err((code, message)) => Err(SystemError::ICRejectionError(code, message))?,
    }
}
