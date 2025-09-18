use candid::{Encode, Principal};
use icramp_types::solana::errors::{Result, SolanaError};

use crate::env::get_solana_env;
use crate::helpers::mock::{
    pump_and_mock_http, responder_bad_b64_len_token, responder_mint_decimals_ok_token,
    responder_mint_decimals_ok_token2022, responder_missing_mint_data_token, responder_wrong_owner,
};

const MINT_USDC: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
const MINT_FAKE_SYS: &str = "11111111111111111111111111111111";
const MINT_FAKE_2: &str = "11111111111111111111111111111112";

#[test]
fn test_is_token_supported_unregistered() {
    let mut binding = get_solana_env();
    let env = binding.as_mut().unwrap();

    let res = env.supported_token(MINT_FAKE_2.to_string());
    assert!(matches!(res, Err(SolanaError::UnsupportedToken(_))));
}

#[test]
fn test_get_token_info_invalid_pubkey() {
    let mut binding = get_solana_env();
    let env = binding.as_mut().unwrap();

    let res = env.get_token("not-a-pubkey".to_string());
    assert!(
        res.is_err(),
        "invalid pubkey must error via validate_token_mint"
    );
}

#[test]
fn test_register_token_invalid_pubkey_rejected() {
    let mut binding = get_solana_env();
    let env = binding.as_mut().unwrap();

    // Fails before any HTTP outcalls, so no pump needed.
    let res = env.register_tokens(vec![(
        "not-a-pubkey".to_string(),
        "BAD".to_string(),
        "BAD".to_string(),
    )]);
    assert!(matches!(res, Err(SolanaError::ParsePubkeyError(_))));
}

#[test]
fn test_register_token_wrong_owner_rejected() {
    let mut binding = get_solana_env();
    let env = binding.as_mut().unwrap();

    let msg = env
        .pic
        .submit_call(
            env.canister_id,
            Principal::anonymous(),
            "register_tokens",
            Encode!(&vec![(
                MINT_FAKE_SYS.to_string(),
                "FAKE".to_string(),
                "FAKE".to_string()
            )])
            .unwrap(),
        )
        .unwrap();

    pump_and_mock_http(&env.pic, 20, responder_wrong_owner());

    let bytes = env.pic.await_call(msg).unwrap();
    let res: Result<()> = candid::decode_one(&bytes).unwrap();
    assert!(matches!(res, Err(SolanaError::UnsupportedToken(_))));
}

#[test]
fn test_register_token_bad_decimal_len_rejected() {
    let mut binding = get_solana_env();
    let env = binding.as_mut().unwrap();

    let msg = env
        .pic
        .submit_call(
            env.canister_id,
            Principal::anonymous(),
            "register_tokens",
            Encode!(&vec![(
                MINT_FAKE_2.to_string(),
                "ODD".to_string(),
                "ODD".to_string()
            )])
            .unwrap(),
        )
        .unwrap();

    // Owner = Token program, then data slice returns TWO bytes (6,0) → backend should error.
    pump_and_mock_http(&env.pic, 20, responder_bad_b64_len_token(6, 0));

    let bytes = env.pic.await_call(msg).unwrap();
    let res: Result<()> = candid::decode_one(&bytes).unwrap();
    // Any RpcError here is fine; most specific backend text: "Expected 1 byte, got 2"
    assert!(matches!(res, Err(SolanaError::RpcError(_))))
}

#[test]
fn test_register_and_query_success_token_program() {
    let mut binding = get_solana_env();
    let env = binding.as_mut().unwrap();

    let mint = "So11111111111111111111111111111111111111112".to_string();

    let msg = env
        .pic
        .submit_call(
            env.canister_id,
            Principal::anonymous(),
            "register_tokens",
            Encode!(&vec![(mint.clone(), "wSOL".to_string(), "SOL".to_string())]).unwrap(),
        )
        .unwrap();

    // Token program owner + decimals = 9
    pump_and_mock_http(&env.pic, 20, responder_mint_decimals_ok_token(9));

    let bytes = env.pic.await_call(msg).unwrap();
    let res: Result<()> = candid::decode_one(&bytes).unwrap();
    res.expect("register_tokens should succeed");

    let info = env.get_token(mint.clone()).unwrap();
    assert_eq!(info.decimals, 9);
    assert_eq!(info.symbol, "wSOL");
    assert_eq!(info.rate_symbol, "SOL");

    assert!(env.supported_token(mint).is_ok());
}

#[test]
fn test_register_and_query_success_token_2022() {
    let mut binding = get_solana_env();
    let env = binding.as_mut().unwrap();

    let mint = "So11111111111111111111111111111111111111112".to_string();

    let msg = env
        .pic
        .submit_call(
            env.canister_id,
            Principal::anonymous(),
            "register_tokens",
            Encode!(&vec![(
                mint.clone(),
                "USDC_DEV".to_string(),
                "USDC".to_string()
            )])
            .unwrap(),
        )
        .unwrap();

    // Token-2022 owner + decimals = 6
    pump_and_mock_http(&env.pic, 20, responder_mint_decimals_ok_token2022(6));

    let bytes = env.pic.await_call(msg).unwrap();
    let res: Result<()> = candid::decode_one(&bytes).unwrap();
    res.expect("register_tokens (2022) should succeed");

    let info = env.get_token(mint.clone()).unwrap();
    assert_eq!(info.decimals, 6);
    assert_eq!(info.symbol, "USDC_DEV");
    assert_eq!(info.rate_symbol, "USDC");

    assert!(env.supported_token(mint).is_ok());
}

#[test]
fn test_get_token_info_unregistered_err() {
    let mut binding = get_solana_env();
    let env = binding.as_mut().unwrap();

    // Use a mint that this test file never registers
    let res = env.get_token(MINT_USDC.to_string());
    assert!(matches!(res, Err(SolanaError::UnsupportedToken(_))));
}

#[test]
fn test_register_missing_mint_data_rejected() {
    let mut binding = get_solana_env();
    let env = binding.as_mut().unwrap();

    let msg = env
        .pic
        .submit_call(
            env.canister_id,
            Principal::anonymous(),
            "register_tokens",
            Encode!(&vec![(MINT_FAKE_2, "MISS".to_string(), "MISS".to_string())]).unwrap(),
        )
        .unwrap();

    pump_and_mock_http(&env.pic, 20, responder_missing_mint_data_token());

    let bytes = env.pic.await_call(msg).unwrap();
    let res: Result<()> = candid::decode_one(&bytes).unwrap();
    assert!(res.is_err(), "missing mint account data slice must error");
}
