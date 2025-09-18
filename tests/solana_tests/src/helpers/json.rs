use canhttp::http::json::JsonRpcRequest;
use serde_json::{Value, json};

/// Convenience: extract first string param (e.g., mint/ATA pubkey)
pub(in crate::helpers) fn first_param_str(jr: &JsonRpcRequest<Value>) -> Option<&str> {
    jr.params()?
        .as_array()
        .and_then(|a| a.get(0))
        .and_then(|v| v.as_str())
}

pub(in crate::helpers) fn resp_ctx_null() -> Value {
    json!({ "context": { "slot": 0 }, "value": null })
}

pub(in crate::helpers) fn resp_ctx_owner(owner_program: &str) -> Value {
    json!({
        "context": { "slot": 0 },
        "value": {
            "data": ["", "base64"],
            "executable": false,
            "lamports": 0,
            "owner": owner_program,
            "rentEpoch": 0,
            "space": 0
        }
    })
}

pub(in crate::helpers) fn resp_ctx_owner_with_b64(owner_program: &str, b64: &str) -> Value {
    json!({
        "context": { "slot": 0 },
        "value": {
            "data": [b64, "base64"],
            "executable": false,
            "lamports": 0,
            "owner": owner_program,
            "rentEpoch": 0,
            "space": 0
        }
    })
}

pub(in crate::helpers) fn resp_token_balance(amount: &str, decimals: u8) -> Value {
    json!({
        "context": { "slot": 0 },
        "value": {
            "amount": amount,
            "decimals": decimals,
            "uiAmount": null,
            "uiAmountString": amount
        }
    })
}

pub(in crate::helpers) fn resp_slot(slot: u64) -> Value {
    json!(slot)
}

pub(in crate::helpers) fn resp_block(blockhash: &str, parent_slot: u64) -> Value {
    json!({
        "previousBlockhash": blockhash,
        "blockhash": blockhash,
        "parentSlot": parent_slot,
        "blockTime": 1u64,
        "blockHeight": parent_slot + 1,
        "rewards": []
    })
}

pub(in crate::helpers) fn resp_latest_blockhash(blockhash: &str) -> Value {
    json!({
        "context": { "slot": 1u64 },
        "value": { "blockhash": blockhash, "lastValidBlockHeight": 999_999u64 }
    })
}

pub(in crate::helpers) fn resp_recent_blockhash(blockhash: &str, lamports_per_sig: u64) -> Value {
    json!({
        "context": { "slot": 1u64 },
        "value": { "blockhash": blockhash, "feeCalculator": { "lamportsPerSignature": lamports_per_sig } }
    })
}

pub(in crate::helpers) fn resp_prioritization_fees_zero() -> Value {
    json!([{ "slot": 1u64, "prioritizationFee": 0u64 }])
}

pub(in crate::helpers) fn resp_send_sig(sig: &str) -> Value {
    json!(sig)
}
