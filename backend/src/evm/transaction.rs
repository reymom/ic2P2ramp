use super::rpc::{
    GetTransactionReceiptResult, MultiGetTransactionReceiptResult, MultiSendRawTransactionResult,
    RpcConfig, SendRawTransactionResult, SendRawTransactionStatus, TransactionReceipt, CANISTER_ID,
};
use crate::state::read_state;

pub async fn send_raw_transaction(tx: String, chain_id: u64) -> SendRawTransactionStatus {
    let rpc_providers = read_state(|s| {
        s.rpc_services
            .get(&chain_id)
            .cloned()
            .ok_or("Unsupported chain ID")
    })
    .unwrap();
    let cycles = 10_000_000_000;

    let arg: Option<RpcConfig> = None;
    let res = ic_cdk::api::call::call_with_payment128(
        CANISTER_ID,
        "eth_sendRawTransaction",
        (rpc_providers, arg, tx),
        cycles,
    )
    .await;
    match res {
        Ok((res,)) => match res {
            MultiSendRawTransactionResult::Consistent(status) => match status {
                SendRawTransactionResult::Ok(status) => status,
                SendRawTransactionResult::Err(e) => {
                    ic_cdk::trap(format!("Error: {:?}", e).as_str());
                }
            },
            MultiSendRawTransactionResult::Inconsistent(_) => {
                ic_cdk::trap("Status is inconsistent");
            }
        },
        Err(e) => ic_cdk::trap(format!("Error: {:?}", e).as_str()),
    }
}

pub async fn check_transaction_receipt(
    tx_hash: String,
    chain_id: u64,
) -> Result<Option<TransactionReceipt>, String> {
    let rpc_providers = read_state(|s| {
        s.rpc_services
            .get(&chain_id)
            .cloned()
            .ok_or("Unsupported chain ID")
    })
    .unwrap();

    let cycles = 10_000_000_000;
    let arg: Option<RpcConfig> = None;
    let res: Result<(MultiGetTransactionReceiptResult,), _> =
        ic_cdk::api::call::call_with_payment128(
            CANISTER_ID,
            "eth_getTransactionReceipt",
            (rpc_providers.clone(), arg, tx_hash.clone()),
            cycles,
        )
        .await;

    ic_cdk::println!("[check_transaction_receipt] res = {:?}", res);
    match res {
        Ok((res,)) => match res {
            MultiGetTransactionReceiptResult::Consistent(res) => match res {
                GetTransactionReceiptResult::Ok(res) => Ok(res),
                GetTransactionReceiptResult::Err(e) => {
                    Err(format!("[check_transaction_receipt] Error: {:?}", e))
                }
            },
            MultiGetTransactionReceiptResult::Inconsistent(_) => {
                Err("Status is inconsistent".to_string())
            }
        },
        Err(e) => Err(format!("[check_transaction_receipt] Error: {:?}", e)),
    }
}
