use candid::{Encode, Principal};
use icramp_types::solana::errors::{Result, SolanaError};

use crate::{
    env::{SolanaTestEnv, get_solana_env},
    helpers::mock::{pump_and_mock_http, responder_mint_decimals_ok_token},
};

// tiny helper: register a test token via mocked RPC (Token program + 1-byte decimals)
fn register_token_ok(env: &mut SolanaTestEnv, mint: &str, symbol: &str, rate: &str, decimals: u8) {
    let msg = env
        .pic
        .submit_call(
            env.canister_id,
            Principal::anonymous(),
            "register_tokens",
            Encode!(&vec![(
                mint.to_string(),
                symbol.to_string(),
                rate.to_string()
            )])
            .unwrap(),
        )
        .unwrap();

    pump_and_mock_http(&env.pic, 20, responder_mint_decimals_ok_token(decimals));

    let bytes = env.pic.await_call(msg).unwrap();
    let res: Result<()> = candid::decode_one(&bytes).unwrap();
    res.expect("register_tokens should succeed");
}

#[test]
fn test_sol_vault() {
    let mut env = get_solana_env().take().expect("Solana env not initialized");
    let offramper = "DoQEH6tJLWYP3z5RNtqrk2AYojrCJBPzdqMqD6FwhAMy".to_string();
    let onramper = "FoQFH6tJLWYP3z5RNtqrk2AYojrCJBPzdqMqD6FwhAMy".to_string();

    // check vault amount
    let vault_res = env.get_offramper_vault(offramper.clone());
    assert!(vault_res.is_err());

    // Step 1: Deposit SOL as offramper
    let amount = 1_000_000_000u64;
    let deposit_res = env.deposit(offramper.clone(), amount, None);
    assert!(deposit_res.is_ok(), "Error in deposit result");

    let vault_res = env.get_offramper_vault(offramper.clone());
    assert!(vault_res.is_ok());
    assert_eq!(vault_res.unwrap().lamports, amount);

    // Step 2: Cancel part of the deposit
    let cancelled_amount = 200_000_000u64;
    let cancel_result = env.cancel(offramper.clone(), cancelled_amount, None);
    assert!(cancel_result.is_ok(), "Failed to cancel deposit");

    // check vault amount for offramper
    let vault_res = env.get_offramper_vault(offramper.clone());
    assert!(vault_res.is_ok());
    assert_eq!(vault_res.unwrap().lamports, amount - cancelled_amount);

    // check vault amount for onramper
    let vault_res = env.get_onramper_vault(offramper.clone());
    assert!(vault_res.is_err());

    let lock_res = env.lock(
        offramper.clone(),
        onramper.clone(),
        amount - cancelled_amount,
        None,
    );
    assert!(lock_res.is_ok(), "Failed to lock funds");

    // check vault amount for offramper
    let vault_res = env.get_offramper_vault(offramper.clone());
    assert!(vault_res.is_ok());
    assert_eq!(vault_res.unwrap().lamports, 0);

    // check vault amount for onramper
    let vault_res = env.get_onramper_vault(onramper.clone());
    assert!(vault_res.is_ok());
    assert_eq!(vault_res.unwrap().lamports, amount - cancelled_amount);

    // Step 3: Unlock part of the locked amount
    let unlock_amount = 300_000_000u64;
    let unlock_res = env.unlock(offramper.clone(), onramper.clone(), unlock_amount, None);
    assert!(unlock_res.is_ok(), "Failed to unlock funds");

    // check vault amount for offramper
    let vault_res = env.get_offramper_vault(offramper.clone());
    assert!(vault_res.is_ok());
    assert_eq!(vault_res.unwrap().lamports, unlock_amount);

    // check vault amount for onramper
    let vault_res = env.get_onramper_vault(onramper.clone());
    assert!(vault_res.is_ok());
    assert_eq!(
        vault_res.unwrap().lamports,
        amount - cancelled_amount - unlock_amount
    );

    // Step 4: Complete the locked amount
    let complete_res = env.complete(
        onramper.clone(),
        amount - cancelled_amount - unlock_amount,
        None,
    );
    assert!(complete_res.is_ok());

    // check vault amount for onramper
    let vault_res = env.get_onramper_vault(onramper.clone());
    assert!(vault_res.is_ok());
    assert_eq!(vault_res.unwrap().lamports, 0);
}

#[test]
fn test_token_vault() {
    let mut env = get_solana_env().take().expect("Solana env not initialized");
    let offramper = "DoQEH6tJLWYP3z5RNtqrk2AYojrCJBPzdqMqD6FwhAMy".to_string();
    let onramper = "FoQFH6tJLWYP3z5RNtqrk2AYojrCJBPzdqMqD6FwhAMy".to_string();

    // pick any valid 32-byte pubkey string for mint (owner is mocked by the responder)
    let mint = "11111111111111111111111111111112".to_string();

    // 0) Non-registered tokens should not be allowed
    let amount = 1_000_000u64; // 1.000000 with 6 decimals
    let res = env.deposit(offramper.clone(), amount, Some(mint.clone()));
    assert!(matches!(res, Err(SolanaError::UnsupportedToken(_))));

    // 1) Register token (Token program owner, decimals=6 like USDC)
    register_token_ok(&mut env, &mint, "USDC", "USDC", 6);

    // 2) Initial state: offramper has no vault entry yet
    let v = env.get_offramper_vault(offramper.clone());
    assert!(v.is_err());

    // 3) Deposit token as offramper
    let res = env.deposit(offramper.clone(), amount, Some(mint.clone()));
    assert!(res.is_ok(), "deposit token failed");

    // offramper holds the tokens, lamports stay 0
    let v = env
        .get_offramper_vault(offramper.clone())
        .expect("offramper vault");
    assert_eq!(v.lamports, 0);
    assert_eq!(v.tokens.get(&mint).copied().unwrap_or(0), amount);

    // 4) Cancel part of the deposit
    let cancelled = 200_000u64;
    let res = env.cancel(offramper.clone(), cancelled, Some(mint.clone()));
    assert!(res.is_ok(), "cancel deposit failed");

    let v = env
        .get_offramper_vault(offramper.clone())
        .expect("offramper vault");
    assert_eq!(
        v.tokens.get(&mint).copied().unwrap_or(0),
        amount - cancelled
    );

    // onramper still has no vault entry
    assert!(env.get_onramper_vault(offramper.clone()).is_err());

    // 5) Lock remaining funds to onramper
    let to_lock = amount - cancelled;
    let res = env.lock(
        offramper.clone(),
        onramper.clone(),
        to_lock,
        Some(mint.clone()),
    );
    assert!(res.is_ok(), "lock funds failed");

    // offramper → 0; onramper → locked
    let v_off = env
        .get_offramper_vault(offramper.clone())
        .expect("offramper vault");
    assert_eq!(v_off.tokens.get(&mint).copied().unwrap_or(0), 0);

    let v_on = env
        .get_onramper_vault(onramper.clone())
        .expect("onramper vault");
    assert_eq!(v_on.tokens.get(&mint).copied().unwrap_or(0), to_lock);

    // 6) Unlock part of the locked amount back to offramper
    let unlock_amount = 300_000u64;
    let res = env.unlock(
        offramper.clone(),
        onramper.clone(),
        unlock_amount,
        Some(mint.clone()),
    );
    assert!(res.is_ok(), "unlock failed");

    let v_off = env
        .get_offramper_vault(offramper.clone())
        .expect("offramper vault");
    assert_eq!(v_off.tokens.get(&mint).copied().unwrap_or(0), unlock_amount);

    let v_on = env
        .get_onramper_vault(onramper.clone())
        .expect("onramper vault");
    assert_eq!(
        v_on.tokens.get(&mint).copied().unwrap_or(0),
        to_lock - unlock_amount
    );

    // 7) Complete the remaining locked amount
    let res = env.complete(
        onramper.clone(),
        to_lock - unlock_amount,
        Some(mint.clone()),
    );
    assert!(res.is_ok(), "complete failed");

    // onramper drained; offramper keeps the unlocked remainder
    let v_on = env
        .get_onramper_vault(onramper.clone())
        .expect("onramper vault");
    assert_eq!(v_on.tokens.get(&mint).copied().unwrap_or(0), 0);

    let v_off = env
        .get_offramper_vault(offramper.clone())
        .expect("offramper vault");
    assert_eq!(v_off.tokens.get(&mint).copied().unwrap_or(0), unlock_amount);
}
