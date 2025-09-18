use candid::{CandidType, Principal, decode_one, encode_args, utils::ArgumentEncoder};

use ic_cdk::api::management_canister::main::CanisterId;
use pocket_ic::PocketIc;
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
        Ok(response) => decode_one(&response).map_err(|e| e.to_string()),
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
        Ok(response) => decode_one(&response).map_err(|e| format!("error decoding: {}", e)),
        Err(e) => Err(format!("Update call failed: {}", e)),
    }
}
