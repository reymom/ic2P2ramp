use candid::{decode_one, encode_args, utils::ArgumentEncoder, CandidType, Principal};

use pocket_ic::{management_canister::CanisterId, PocketIc, WasmResult};
use serde::de::DeserializeOwned;

/// Executes a query call on the specified canister and decodes the response into the expected type.
pub fn query_call<T, R>(
    pic: &PocketIc,
    canister_id: CanisterId,
    method: &str,
    args: T,
) -> Result<R, String>
where
    T: CandidType + ArgumentEncoder,
    R: CandidType + DeserializeOwned,
{
    match pic.query_call(
        canister_id,
        Principal::anonymous(),
        method,
        encode_args(args).unwrap(),
    ) {
        Ok(WasmResult::Reply(response)) => decode_one(&response).map_err(|e| e.to_string()),
        Ok(WasmResult::Reject(err)) => Err(format!("Query rejected: {}", err)),
        Err(e) => Err(format!("Query call failed: {}", e)),
    }
}

/// Executes an update call on the specified canister and decodes the response into the expected type.
pub fn update_call<T, R>(
    pic: &mut PocketIc,
    canister_id: CanisterId,
    method: &str,
    args: T,
) -> Result<R, String>
where
    T: CandidType + ArgumentEncoder,
    R: CandidType + DeserializeOwned,
{
    match pic.update_call(
        canister_id,
        Principal::anonymous(),
        method,
        encode_args(args).unwrap(),
    ) {
        Ok(WasmResult::Reply(response)) => {
            decode_one(&response).map_err(|e| format!("error decoding: {}", e))
        }
        Ok(WasmResult::Reject(err)) => Err(format!("Update rejected: {}", err)),
        Err(e) => Err(format!("Update call failed: {}", e)),
    }
}
