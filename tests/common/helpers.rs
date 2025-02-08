use std::str::FromStr;

use bitcoincore_rpc::{bitcoin::Address, Auth, Client, RpcApi};
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

/// Mine `n` blocks and send rewards to the specified Bitcoin address.
pub fn mine_blocks(bitcoin_address: &str, n: u64) {
    const BTC_RPC_URL: &str = "http://127.0.0.1:18443";
    const RPC_USER: &str = "icp";
    const RPC_PASSWORD: &str = "test";

    let btc_rpc = Client::new(
        BTC_RPC_URL,
        Auth::UserPass(RPC_USER.to_string(), RPC_PASSWORD.to_string()),
    )
    .expect("Failed to create Bitcoin RPC client");

    let address = Address::from_str(bitcoin_address)
        .expect("Invalid Bitcoin address")
        .assume_checked();

    btc_rpc
        .generate_to_address(n, &address)
        .expect("Failed to mine blocks");
}

/// Progress the IC state by a specified number of ticks.
pub fn tick_many(env: &mut PocketIc, count: usize) {
    for _ in 0..count {
        env.tick();
    }
}

/// Generate a new Bitcoin address using the regtest wallet.
pub fn generate_new_bitcoin_address(
    btc_rpc_url: &str,
    rpc_user: &str,
    rpc_password: &str,
) -> String {
    let btc_rpc = Client::new(
        btc_rpc_url,
        Auth::UserPass(rpc_user.to_string(), rpc_password.to_string()),
    )
    .expect("Failed to create Bitcoin RPC client");

    let new_address = btc_rpc
        .get_new_address(None, None)
        .expect("Failed to generate a new Bitcoin address");
    new_address.assume_checked().to_string()
}
