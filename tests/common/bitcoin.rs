use std::{process::Command, str::FromStr};

use bitcoincore_rpc::{bitcoin::Address, Auth, Client, RpcApi};
use candid::Principal;

use ic_btc_interface::{
    GetBlockHeadersRequest, GetBlockHeadersResponse, GetUtxosRequest, GetUtxosResponse,
    NetworkInRequest,
};
use pocket_ic::PocketIc;

use crate::common::{
    helpers::{query_call, update_call},
    setup::BTC_CANISTER_ID,
};

use super::setup::{BTC_RPC_URL, RPC_PASSWORD, RPC_USER};

/// Mine `n` blocks and send rewards to the specified Bitcoin address.
pub fn mine_blocks(bitcoin_address: &str, n: u64) {
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

/// Generate a new Bitcoin address using the regtest wallet.
pub fn generate_new_bitcoin_address() -> String {
    let btc_rpc = Client::new(
        format!("{}/wallet/testwallet", BTC_RPC_URL).as_str(),
        Auth::UserPass(RPC_USER.to_string(), RPC_PASSWORD.to_string()),
    )
    .expect("Failed to create Bitcoin RPC client");

    let new_address = btc_rpc
        .get_new_address(None, None)
        .expect("Failed to generate a new Bitcoin address");
    new_address.assume_checked().to_string()
}

/// Progress the IC state by a specified number of ticks.
// pub fn tick_many(env: &mut PocketIc, count: usize) {
//     for _ in 0..count {
//         env.tick();
//     }
// }

/// Automate tracking of Bitcoin syncing after mining.
pub fn track_block_sync(env: &mut PocketIc, blocks_to_mine: u32) {
    let initial_height = get_bitcoin_tip_height(env).unwrap_or(0);
    let expected_height = initial_height + blocks_to_mine;
    tick_until_synced(env, expected_height);
}

fn tick_until_synced(env: &mut PocketIc, expected_height: u32) {
    for tick in 0..1500 {
        env.tick();
        if let Some(current_height) = get_bitcoin_tip_height(env) {
            ic_cdk::println!(
                "Bitcoin canister sync: current height = {}, expected = {}",
                current_height,
                expected_height
            );

            if current_height >= expected_height {
                ic_cdk::println!("Bitcoin canister fully synced at tick {}.", tick);
                return;
            }
        }
        // alternativamente:
        // if has_utxos(env, &p2pkh_address) {
        //     ic_cdk::println!("Bitcoin canister has UTXOs, sync is complete.");
        //     break;
        // }
    }

    panic!(
        "ICP Bitcoin canister failed to sync to height {} within 1500 ticks",
        expected_height
    );
}

fn get_bitcoin_tip_height(env: &mut PocketIc) -> Option<u32> {
    let request = GetBlockHeadersRequest {
        start_height: 0,
        end_height: None,
        network: NetworkInRequest::Regtest,
    };

    let response: Result<GetBlockHeadersResponse, String> = update_call(
        env,
        Principal::from_text(BTC_CANISTER_ID).unwrap(),
        "bitcoin_get_block_headers",
        (request,),
    );

    ic_cdk::println!("get_bitcoin_tip_height response: {:?}", response);
    match response {
        Ok(res) => Some(res.tip_height),
        Err(e) => {
            if e.contains("not fully synced") {
                None // Return None if it's still syncing
            } else {
                panic!("Unexpected error: {}", e);
            }
        }
    }
}

pub fn has_utxos(env: &mut PocketIc, btc_address: &str) -> bool {
    let response: Result<GetUtxosResponse, String> = query_call(
        env,
        Principal::from_text(BTC_CANISTER_ID).unwrap(),
        "bitcoin_get_utxos",
        (GetUtxosRequest {
            address: btc_address.to_string(),
            network: NetworkInRequest::Regtest,
            filter: None,
        },),
    );

    ic_cdk::println!("bitcoin_get_utxos response: {:?}", response);

    match response {
        Ok(res) => !res.utxos.is_empty(),
        Err(_) => false,
    }
}

pub fn get_ord_rune_balance(rune_name: &str) -> Option<u64> {
    let output = Command::new("docker")
        .args([
            "compose",
            "exec",
            "ord",
            "./bitcoin_backend/scripts/regtest/ord_wallet.sh",
            "balance",
        ])
        .output()
        .expect("Failed to execute ord_wallet.sh balance");

    if !output.status.success() {
        eprintln!(
            "Error getting rune balance: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return None;
    }

    let balance_json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("Failed to parse JSON output");

    // Extract the rune balance from JSON
    balance_json["runes"]
        .get(rune_name)
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<u64>().ok())
}

pub fn send_runes_from_ord(address: &str, rune_name: &str, amount: u64) -> bool {
    let output = Command::new("docker")
        .args([
            "compose",
            "exec",
            "ord",
            "./bitcoin_backend/scripts/regtest/ord_wallet.sh",
            "mint",
            "--rune",
            rune_name,
            "--fee-rate",
            "1",
        ])
        .output()
        .expect("Failed to execute ord_wallet.sh mint");

    if output.status.success() {
        ic_cdk::println!(
            "Successfully sent {} runes '{}' to {}",
            amount,
            rune_name,
            address
        );
        true
    } else {
        eprintln!(
            "Error sending runes: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        false
    }
}
