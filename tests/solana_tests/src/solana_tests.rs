use candid::{Encode, Nat, Principal};
use icramp_types::solana::errors::Result;
use sol_rpc_types::TokenAmount;

use crate::{
    env::get_solana_env,
    helpers::mock::{
        pump_and_mock_http, responder_balance_zero_for_mint, responder_create_ata_flow,
        responder_send_sol, responder_send_spl_token_create_dest_ata,
    },
    setup::SOL_RPC_CANISTER_ID,
};

#[test]
fn test_solana_canister_init() {
    let binding = get_solana_env();
    let env = binding.as_ref().unwrap();

    let query_stats = env
        .pic
        .canister_status(env.canister_id, None)
        .unwrap()
        .query_stats;
    assert_eq!(query_stats.num_calls_total, Nat::from(0u32));

    assert!(
        !env.canister_id.to_string().is_empty(),
        "Canister ID is empty!"
    );
}

#[test]
fn test_get_sol_account_balance() {
    let mut binding = get_solana_env();
    let env = binding.as_mut().unwrap();

    let account = env.get_canister_account();
    assert!(!account.is_empty(), "canister account is empty");
    ic_cdk::println!("account = {:?}", account);

    let msg_id = env
        .pic
        .submit_call(
            env.canister_id,
            Principal::anonymous(),
            "get_balance",
            Encode!(&account.clone()).unwrap(),
        )
        .expect("submit_call");

    pump_and_mock_http(
        &env.pic,
        30,
        |_| serde_json::json!({ "context": { "slot": 0 }, "value": 0u64 }),
    );

    let bytes = env.pic.await_call(msg_id).expect("await_call");
    let api_result: Result<Nat> = candid::decode_one(&bytes).expect("decode candid");
    let balance = api_result.expect("backend returned Err");
    assert_eq!(balance, Nat::from(0u32));
}

#[test]
fn test_get_spl_token_balance_zero() {
    let mut binding = get_solana_env();
    let env = binding.as_mut().unwrap();

    let owner = env.get_canister_account();
    let mint = "So11111111111111111111111111111111111111112".to_string();

    let msg = env
        .pic
        .submit_call(
            env.canister_id,
            Principal::anonymous(),
            "get_spl_token_balance",
            Encode!(&owner.clone(), &mint.clone()).unwrap(),
        )
        .unwrap();

    pump_and_mock_http(&env.pic, 80, responder_balance_zero_for_mint(mint.clone()));

    let bytes = env.pic.await_call(msg).unwrap();
    let res: Result<TokenAmount> = candid::decode_one(&bytes).unwrap();
    assert_eq!(res.unwrap().amount, "0");
}

#[test]
fn test_create_ata_when_missing() {
    let mut binding = get_solana_env();
    let env = binding.as_mut().unwrap();

    let mint = "So11111111111111111111111111111111111111112".to_string();

    let sol_rpc_canister_id = Principal::from_text(SOL_RPC_CANISTER_ID).unwrap();
    let msg = env
        .pic
        .submit_call(
            env.canister_id,
            Principal::anonymous(),
            "create_associated_token_account",
            Encode!(&Some(sol_rpc_canister_id), &mint.clone()).unwrap(),
        )
        .unwrap();

    pump_and_mock_http(&env.pic, 80, responder_create_ata_flow(mint.clone()));

    let bytes = env.pic.await_call(msg).unwrap();
    let res: Result<String> = candid::decode_one(&bytes).unwrap();
    assert!(!res.unwrap().is_empty());
}

#[test]
fn test_send_sol_happy_path() {
    let mut binding = get_solana_env();
    let env = binding.as_mut().unwrap();
    let dst = env.get_canister_account();

    let sol_rpc_canister_id = Principal::from_text(SOL_RPC_CANISTER_ID).unwrap();
    let msg = env
        .pic
        .submit_call(
            env.canister_id,
            Principal::anonymous(),
            "send_sol",
            Encode!(
                &Some(sol_rpc_canister_id),
                &dst.clone(),
                &Nat::from(1_000_000u64)
            )
            .unwrap(),
        )
        .unwrap();

    pump_and_mock_http(&env.pic, 80, responder_send_sol());

    let bytes = env.pic.await_call(msg).unwrap();
    let res: Result<String> = candid::decode_one(&bytes).unwrap();
    assert!(res.unwrap().len() > 40);
}

#[test]
fn test_send_spl_token_creates_dest_ata() {
    let mut binding = get_solana_env();
    let env = binding.as_mut().unwrap();

    let mint = "So11111111111111111111111111111111111111112".to_string();
    let to = env.get_canister_account();

    let sol_rpc_canister_id = Principal::from_text(SOL_RPC_CANISTER_ID).unwrap();
    let msg = env
        .pic
        .submit_call(
            env.canister_id,
            Principal::anonymous(),
            "send_spl_token",
            Encode!(
                &Some(sol_rpc_canister_id),
                &mint.clone(),
                &to.clone(),
                &Nat::from(10u64)
            )
            .unwrap(),
        )
        .unwrap();

    pump_and_mock_http(
        &env.pic,
        100,
        responder_send_spl_token_create_dest_ata(mint.clone()),
    );

    let bytes = env.pic.await_call(msg).unwrap();
    let res: Result<String> = candid::decode_one(&bytes).unwrap();
    assert!(res.unwrap().len() > 40);
}
