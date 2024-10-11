use std::str;

use candid::{self, CandidType, Principal};
use evm_rpc_canister_types::{EvmRpcCanister, SendRawTransactionStatus};
use serde::Deserialize;

pub const CANISTER_ID: Principal =
    Principal::from_slice(b"\x00\x00\x00\x00\x02\x30\x00\xCC\x01\x01"); // 7hfb6-caaaa-aaaar-qadga-cai

pub const EVM_RPC: EvmRpcCanister = EvmRpcCanister(CANISTER_ID);

#[derive(CandidType, Deserialize)]
pub enum CustomTransactionStatus {
    Ok(Option<String>),
    NonceTooLow,
    NonceTooHigh,
    InsufficientFunds,
    ReplacementUnderpriced,
}

impl From<SendRawTransactionStatus> for CustomTransactionStatus {
    fn from(status: SendRawTransactionStatus) -> CustomTransactionStatus {
        status.into()
    }
}
