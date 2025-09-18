use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use canhttp::http::json::JsonRpcRequest;
use pocket_ic::PocketIc;
use pocket_ic::common::rest::{
    CanisterHttpHeader, CanisterHttpReply, CanisterHttpResponse, MockCanisterHttpResponse,
};
use serde_json::Value;
use std::cell::RefCell;

use crate::helpers::json::{
    first_param_str, resp_block, resp_ctx_null, resp_ctx_owner, resp_ctx_owner_with_b64,
    resp_latest_blockhash, resp_prioritization_fees_zero, resp_recent_blockhash, resp_send_sig,
    resp_slot, resp_token_balance,
};

// ---------- Common constants (valid base58 sizes) ----------
const FAKE_BLOCKHASH_32: &str = "11111111111111111111111111111111"; // 32 chars → 32 zero bytes
const FAKE_SIG_64: &str = "1111111111111111111111111111111111111111111111111111111111111111"; // 64 chars → 64 zero bytes
const SYS_PROGRAM_ID: &str = "11111111111111111111111111111111";
const TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
const TOKEN_2022_PROGRAM_ID: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";

/// Pump execution and answer all pending HTTP outcalls using `responder`.
pub fn pump_and_mock_http(
    pic: &PocketIc,
    rounds: usize,
    mut responder: impl FnMut(&JsonRpcRequest<Value>) -> Value,
) {
    for _ in 0..rounds {
        // Drain everything currently queued; tick after each drain
        loop {
            let reqs = pic.get_canister_http();
            if reqs.is_empty() {
                break;
            }

            for req in reqs {
                let jr: JsonRpcRequest<Value> =
                    serde_json::from_slice(&req.body).expect("json-rpc request parse");
                let result = responder(&jr);

                let body = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id":      jr.id(),
                    "result":  result,
                })
                .to_string()
                .into_bytes();

                pic.mock_canister_http_response(MockCanisterHttpResponse {
                    subnet_id: req.subnet_id,
                    request_id: req.request_id,
                    response: CanisterHttpResponse::CanisterHttpReply(CanisterHttpReply {
                        status: 200,
                        headers: vec![CanisterHttpHeader {
                            name: "content-type".into(),
                            value: "application/json".into(),
                        }],
                        body,
                    }),
                    additional_responses: vec![],
                });
            }

            // let the canister consume the replies and enqueue follow-ups
            pic.tick();
        }

        // progress timers/rounds even if nothing was queued this iteration
        pic.tick();
    }
}

// ======================================================================
// Scenario builders: return closures we can hand to `pump_and_mock_http`.
// ======================================================================

/// Balance: SPL mint owner (legacy Token program) + zero token balance on derived ATA.
pub fn responder_balance_zero_for_mint(
    mint: String,
) -> impl FnMut(&JsonRpcRequest<Value>) -> Value {
    move |jr: &JsonRpcRequest<Value>| match jr.method() {
        // Mint owner lookup → return Token program
        "getAccountInfo" | "getAccountInfoWithContext" | "getAccountInfoWithOpts" => {
            if first_param_str(jr) == Some(mint.as_str()) {
                resp_ctx_owner("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")
            } else {
                // Any other account info probe → say "missing"
                resp_ctx_null()
            }
        }
        // Then the client queries the derived ATA balance → 0
        "getTokenAccountBalance" => resp_token_balance("0", 6),

        // Harmless defaults some paths use:
        "getSlot" => resp_slot(1_234_567),
        "getRecentPrioritizationFees" => resp_prioritization_fees_zero(),
        _ => Value::Null,
    }
}

/// Create ATA: mint owner → Token program; first non-mint `getAccountInfo` = exists; second = missing.
/// Also provides blockhash + sendTransaction response.
pub fn responder_create_ata_flow(mint: String) -> impl FnMut(&JsonRpcRequest<Value>) -> Value {
    let non_mint_probe = RefCell::new(0usize);
    move |jr: &JsonRpcRequest<Value>| match jr.method() {
        "getAccountInfo" | "getAccountInfoWithContext" | "getAccountInfoWithOpts" => {
            if first_param_str(jr) == Some(mint.as_str()) {
                // Mint owner is legacy Token program
                resp_ctx_owner("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")
            } else {
                // Probe order: first non-mint account exists, second is missing
                let i = {
                    let mut c = non_mint_probe.borrow_mut();
                    *c += 1;
                    *c
                };
                if i == 1 {
                    resp_ctx_owner("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")
                } else {
                    resp_ctx_null()
                }
            }
        }
        "getSlot" => resp_slot(1_234_567),
        "getBlock" => resp_block(FAKE_BLOCKHASH_32, 1_234_566),
        "sendTransaction" => resp_send_sig(FAKE_SIG_64),
        _ => Value::Null,
    }
}

/// Send SOL: blockhash + (optional) fee helpers + signature.
pub fn responder_send_sol() -> impl FnMut(&JsonRpcRequest<Value>) -> Value {
    move |jr: &JsonRpcRequest<Value>| match jr.method() {
        "getSlot" => resp_slot(1_234_567),
        "getBlock" => resp_block(FAKE_BLOCKHASH_32, 1_234_566),
        "getLatestBlockhash" => resp_latest_blockhash(FAKE_BLOCKHASH_32),
        "getRecentBlockhash" => resp_recent_blockhash(FAKE_BLOCKHASH_32, 5_000),
        "getRecentPrioritizationFees" => resp_prioritization_fees_zero(),
        "sendTransaction" => resp_send_sig(FAKE_SIG_64),
        _ => Value::Null,
    }
}

/// Send SPL: mint owner; source ATA exists, dest ATA missing; blockhash + signature.
pub fn responder_send_spl_token_create_dest_ata(
    mint: String,
) -> impl FnMut(&JsonRpcRequest<Value>) -> Value {
    let non_mint_probe = RefCell::new(0usize);
    move |jr: &JsonRpcRequest<Value>| match jr.method() {
        "getAccountInfo" | "getAccountInfoWithContext" | "getAccountInfoWithOpts" => {
            if first_param_str(jr) == Some(mint.as_str()) {
                resp_ctx_owner("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")
            } else {
                let i = {
                    let mut c = non_mint_probe.borrow_mut();
                    *c += 1;
                    *c
                };
                if i == 1 {
                    // Source ATA exists
                    resp_ctx_owner("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")
                } else {
                    // Dest ATA missing → the canister will create it
                    resp_ctx_null()
                }
            }
        }
        "getSlot" => resp_slot(1_234_560),
        "getBlock" => resp_block(FAKE_BLOCKHASH_32, 1_234_559),
        "getLatestBlockhash" => resp_latest_blockhash(FAKE_BLOCKHASH_32),
        "getRecentBlockhash" => resp_recent_blockhash(FAKE_BLOCKHASH_32, 5_000),
        "getRecentPrioritizationFees" => resp_prioritization_fees_zero(),
        "sendTransaction" => resp_send_sig(FAKE_SIG_64),
        _ => Value::Null,
    }
}

/// Token registry helper:
/// Respond that any probed mint account is owned by **System Program** (not Token/Token-2022).
/// This makes `fetch_mint_decimals` return `UnsupportedToken` *before* issuing the dataSlice call.
/// NOTE: We rely on `resp_ctx_owner` which includes `space: 0` to avoid panics in sol-rpc client.
pub fn responder_wrong_owner() -> impl FnMut(&JsonRpcRequest<Value>) -> Value {
    move |jr: &JsonRpcRequest<Value>| match jr.method() {
        "getAccountInfo" | "getAccountInfoWithContext" | "getAccountInfoWithOpts" => {
            resp_ctx_owner(SYS_PROGRAM_ID)
        }
        "getRecentPrioritizationFees" => resp_prioritization_fees_zero(),
        "getSlot" => resp_slot(1_234_567),
        _ => Value::Null,
    }
}

pub fn responder_bad_b64_len_token(
    first: u8,
    second: u8,
) -> impl FnMut(&JsonRpcRequest<Value>) -> Value {
    let step = RefCell::new(0usize);
    move |jr: &JsonRpcRequest<Value>| match jr.method() {
        "getAccountInfo" | "getAccountInfoWithContext" | "getAccountInfoWithOpts" => {
            let i = {
                let mut s = step.borrow_mut();
                *s += 1;
                *s
            };
            if i == 1 {
                // owner probe
                resp_ctx_owner(TOKEN_PROGRAM_ID)
            } else {
                // decimals slice → return TWO bytes (should cause "Expected 1 byte, got 2")
                let b64 = STANDARD.encode([first, second]);
                resp_ctx_owner_with_b64(TOKEN_PROGRAM_ID, &b64)
            }
        }
        "getRecentPrioritizationFees" => resp_prioritization_fees_zero(),
        "getSlot" => resp_slot(1_234_567),
        _ => Value::Null,
    }
}

pub fn responder_mint_decimals_ok_token(
    decimals: u8,
) -> impl FnMut(&JsonRpcRequest<Value>) -> Value {
    let step = RefCell::new(0usize);
    move |jr: &JsonRpcRequest<Value>| match jr.method() {
        "getAccountInfo" | "getAccountInfoWithContext" | "getAccountInfoWithOpts" => {
            let i = {
                let mut s = step.borrow_mut();
                *s += 1;
                *s
            };
            if i == 1 {
                // owner probe
                resp_ctx_owner(TOKEN_PROGRAM_ID)
            } else {
                // decimals slice → exactly 1 byte
                let b64 = STANDARD.encode([decimals]);
                resp_ctx_owner_with_b64(TOKEN_PROGRAM_ID, &b64)
            }
        }
        "getRecentPrioritizationFees" => resp_prioritization_fees_zero(),
        "getSlot" => resp_slot(1_234_567),
        _ => Value::Null,
    }
}

// OK: owner = Token-2022, decimals = <byte>
pub fn responder_mint_decimals_ok_token2022(
    decimals: u8,
) -> impl FnMut(&JsonRpcRequest<Value>) -> Value {
    let step = RefCell::new(0usize);
    move |jr: &JsonRpcRequest<Value>| match jr.method() {
        "getAccountInfo" | "getAccountInfoWithContext" | "getAccountInfoWithOpts" => {
            let i = {
                let mut s = step.borrow_mut();
                *s += 1;
                *s
            };
            if i == 1 {
                resp_ctx_owner(TOKEN_2022_PROGRAM_ID)
            } else {
                let b64 = STANDARD.encode([decimals]);
                resp_ctx_owner_with_b64(TOKEN_2022_PROGRAM_ID, &b64)
            }
        }
        "getRecentPrioritizationFees" => resp_prioritization_fees_zero(),
        "getSlot" => resp_slot(1_234_567),
        _ => Value::Null,
    }
}

// ERROR: owner ok, but data-slice probe returns null (Mint account not found)
pub fn responder_missing_mint_data_token() -> impl FnMut(&JsonRpcRequest<Value>) -> Value {
    let step = RefCell::new(0usize);
    move |jr: &JsonRpcRequest<Value>| match jr.method() {
        "getAccountInfo" | "getAccountInfoWithContext" | "getAccountInfoWithOpts" => {
            let i = {
                let mut s = step.borrow_mut();
                *s += 1;
                *s
            };
            if i == 1 {
                resp_ctx_owner(TOKEN_PROGRAM_ID)
            } else {
                // second probe (with data_slice) => null (Ok(None)) → "Mint account not found"
                resp_ctx_null()
            }
        }
        "getRecentPrioritizationFees" => resp_prioritization_fees_zero(),
        "getSlot" => resp_slot(1_234_567),
        _ => Value::Null,
    }
}
