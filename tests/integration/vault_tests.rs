use bitcoin_backend::types::{RuneID, VaultEntry};
use icramp_types::bitcoin::errors::Result;

use crate::{
    common::helpers::{query_call, update_call},
    integration::env::get_bitcoin_env,
};

#[test]
fn test_btc_vault() {
    let mut env = get_bitcoin_env()
        .take()
        .expect("Bitcoin env not initialized");
    let offramper = "bcrt1qofframper".to_string();
    let onramper = "bcrt1qonramper".to_string();

    // check vault amount
    let vault_res: Result<VaultEntry> = query_call(
        &env.pic,
        env.canister_id,
        "get_offramper_deposits",
        (offramper.clone(),),
    )
    .expect("failed to query get offramper vault");
    assert!(vault_res.is_err());

    // Step 1: Deposit BTC for offramper
    let deposit_result: Result<()> = update_call(
        &mut env.pic,
        env.canister_id,
        "deposit_to_address_vault",
        (offramper.clone(), 1_000_000u64, None::<RuneID>),
    )
    .expect("Failed to query deposit to vault");
    assert!(deposit_result.is_ok(), "Failed to deposit to vault");

    // check vault amount
    let vault_res: Result<VaultEntry> = query_call(
        &env.pic,
        env.canister_id,
        "get_offramper_deposits",
        (offramper.clone(),),
    )
    .expect("failed to query get offramper vault");
    assert!(vault_res.is_ok());
    assert_eq!(vault_res.unwrap().bitcoin_balance, 1_000_000);

    // Step 2: Cancel the deposit
    let cancel_result: Result<()> = update_call(
        &mut env.pic,
        env.canister_id,
        "cancel_deposit",
        (offramper.clone(), 500_000u64, None::<RuneID>), // Cancel 0.005 BTC
    )
    .expect("Failed to query cancel deposit");
    assert!(cancel_result.is_ok(), "Failed to cancel deposit");

    // check vault amount for offramper
    let vault_res: Result<VaultEntry> = query_call(
        &env.pic,
        env.canister_id,
        "get_offramper_deposits",
        (offramper.clone(),),
    )
    .expect("failed to query get offramper vault");
    assert!(vault_res.is_ok());
    assert_eq!(vault_res.unwrap().bitcoin_balance, 500_000);

    // check vault amount for onramper
    let vault_res: Result<VaultEntry> = query_call(
        &env.pic,
        env.canister_id,
        "get_onramper_deposits",
        (offramper.clone(),),
    )
    .expect("failed to query get onramper vault");
    assert!(vault_res.is_err());

    // Step 3: Lock funds for onramper
    let lock_result: Result<()> = update_call(
        &mut env.pic,
        env.canister_id,
        "lock_funds",
        (
            offramper.clone(),
            onramper.clone(),
            500_000u64,
            None::<RuneID>,
        ),
    )
    .expect("Failed to query lock funds");
    assert!(lock_result.is_ok(), "Failed to lock funds");

    // check vault amount for offramper
    let vault_res: Result<VaultEntry> = query_call(
        &env.pic,
        env.canister_id,
        "get_offramper_deposits",
        (offramper.clone(),),
    )
    .expect("failed to query get offramper vault");
    assert!(vault_res.is_ok());
    assert_eq!(vault_res.unwrap().bitcoin_balance, 0);

    // check vault amount for onramper
    let vault_res: Result<VaultEntry> = query_call(
        &env.pic,
        env.canister_id,
        "get_onramper_deposits",
        (onramper.clone(),),
    )
    .expect("failed to query get onramper vault");
    assert!(vault_res.is_ok());
    assert_eq!(vault_res.unwrap().bitcoin_balance, 500_000);

    // Step 4: Unlock
    let unlock_result: Result<()> = update_call(
        &mut env.pic,
        env.canister_id,
        "unlock_funds",
        (
            offramper.clone(),
            onramper.clone(),
            500_000u64,
            None::<RuneID>,
        ),
    )
    .expect("Failed to query unlock funds");
    assert!(unlock_result.is_ok(), "Failed to unlock funds");

    // check vault amount for offramper
    let vault_res: Result<VaultEntry> = query_call(
        &env.pic,
        env.canister_id,
        "get_offramper_deposits",
        (offramper.clone(),),
    )
    .expect("failed to query get offramper vault");
    assert!(vault_res.is_ok());
    assert_eq!(vault_res.unwrap().bitcoin_balance, 500_000);

    // check vault amount for onramper
    let vault_res: Result<VaultEntry> = query_call(
        &env.pic,
        env.canister_id,
        "get_onramper_deposits",
        (onramper.clone(),),
    )
    .expect("failed to query get onramper vault");
    assert!(vault_res.is_ok());
    assert_eq!(vault_res.unwrap().bitcoin_balance, 0);
}

#[test]
fn test_runes_vault() {
    let mut env = get_bitcoin_env()
        .take()
        .expect("Bitcoin env not initialized");
    let offramper = "bcrt1qofframper".to_string();
    let onramper = "bcrt1qonramper".to_string();
    let rune_id = RuneID::new(146, 1).unwrap();

    // check vault amount
    let vault_res: Result<VaultEntry> = query_call(
        &env.pic,
        env.canister_id,
        "get_offramper_deposits",
        (offramper.clone(),),
    )
    .expect("failed to query get offramper vault");
    assert!(vault_res.is_err());

    // Step 1: Deposit Runes
    let deposit_rune_result: Result<()> = update_call(
        &mut env.pic,
        env.canister_id,
        "deposit_to_address_vault",
        (offramper.clone(), 100u64, rune_id.clone()), // Deposit 100 Runes
    )
    .expect("Failed to query deposit rune");
    assert!(deposit_rune_result.is_ok(), "Failed to deposit Runes");

    // check vault amount
    let vault_res: Result<VaultEntry> = query_call(
        &env.pic,
        env.canister_id,
        "get_offramper_deposits",
        (offramper.clone(),),
    )
    .expect("failed to query get offramper vault");
    assert!(vault_res.is_ok());
    assert_eq!(vault_res.clone().unwrap().runes.len(), 1);
    assert_eq!(vault_res.unwrap().runes.get(&rune_id).unwrap(), &100);

    // Step 2: Cancel some part of the deposit
    let cancel_result: Result<()> = update_call(
        &mut env.pic,
        env.canister_id,
        "cancel_deposit",
        (offramper.clone(), 50u64, rune_id.clone()), // Cancel 50 runes
    )
    .expect("Failed to query cancel deposit");
    assert!(cancel_result.is_ok(), "Failed to cancel deposit");

    // check vault amount for offramper
    let vault_res: Result<VaultEntry> = query_call(
        &env.pic,
        env.canister_id,
        "get_offramper_deposits",
        (offramper.clone(),),
    )
    .expect("failed to query get offramper vault");
    assert!(vault_res.is_ok());
    assert_eq!(vault_res.clone().unwrap().runes.len(), 1);
    assert_eq!(vault_res.unwrap().runes.get(&rune_id).unwrap(), &50);

    // check vault amount for onramper
    let vault_res: Result<VaultEntry> = query_call(
        &env.pic,
        env.canister_id,
        "get_onramper_deposits",
        (offramper.clone(),),
    )
    .expect("failed to query get onramper vault");
    assert!(vault_res.is_err());

    // Step 2: Lock funds for onramper
    let lock_result: Result<()> = update_call(
        &mut env.pic,
        env.canister_id,
        "lock_funds",
        (offramper.clone(), onramper.clone(), 50u64, rune_id.clone()),
    )
    .expect("Failed to query lock funds");
    assert!(lock_result.is_ok(), "Failed to lock funds");

    // check vault amount for offramper
    let vault_res: Result<VaultEntry> = query_call(
        &env.pic,
        env.canister_id,
        "get_offramper_deposits",
        (offramper.clone(),),
    )
    .expect("failed to query get offramper vault");
    assert!(vault_res.is_ok());
    assert_eq!(vault_res.clone().unwrap().runes.len(), 0);

    // check vault amount for onramper
    let vault_res: Result<VaultEntry> = query_call(
        &env.pic,
        env.canister_id,
        "get_onramper_deposits",
        (onramper.clone(),),
    )
    .expect("failed to query get onramper vault");
    assert_eq!(vault_res.clone().unwrap().runes.len(), 1);
    assert_eq!(vault_res.unwrap().runes.get(&rune_id).unwrap(), &50);

    // Step 3: Unlock
    let unlock_result: Result<()> = update_call(
        &mut env.pic,
        env.canister_id,
        "unlock_funds",
        (offramper.clone(), onramper.clone(), 50u64, rune_id.clone()),
    )
    .expect("Failed to query unlock funds");
    assert!(unlock_result.is_ok(), "Failed to unlock funds");

    // check vault amount for offramper
    let vault_res: Result<VaultEntry> = query_call(
        &env.pic,
        env.canister_id,
        "get_offramper_deposits",
        (offramper.clone(),),
    )
    .expect("failed to query get offramper vault");
    assert!(vault_res.is_ok());
    assert_eq!(vault_res.clone().unwrap().runes.len(), 1);
    assert_eq!(vault_res.unwrap().runes.get(&rune_id).unwrap(), &50);

    // check vault amount for onramper
    let vault_res: Result<VaultEntry> = query_call(
        &env.pic,
        env.canister_id,
        "get_onramper_deposits",
        (onramper.clone(),),
    )
    .expect("failed to query get onramper vault");
    assert_eq!(vault_res.clone().unwrap().runes.len(), 0);
}

#[test]
fn test_transfer_vault_integration() {
    let mut binding = get_bitcoin_env();
    let env = binding.as_mut().unwrap();
    let offramper = "bcrt1qofframper".to_string();
    let onramper = "bcrt1qonramper".to_string();

    // Step 1: Deposit BTC for offramper
    let deposit_result: Result<()> = update_call(
        &mut env.pic,
        env.canister_id,
        "deposit_to_address_vault",
        (offramper.clone(), 1_000_000u64, None::<RuneID>),
    )
    .expect("Failed to deposit to vault");
    assert!(deposit_result.is_ok(), "Failed to deposit to vault");

    // Step 2: Lock funds by onramper
    let lock_result: Result<()> = update_call(
        &mut env.pic,
        env.canister_id,
        "lock_funds",
        (
            offramper.clone(),
            onramper.clone(),
            1_000_000u64,
            None::<RuneID>,
        ),
    )
    .expect("Failed to lock funds");
    assert!(lock_result.is_ok(), "Failed to lock funds");

    // Step 3: Transfer funds
    let tx_id: Result<String> = update_call(
        &mut env.pic,
        env.canister_id,
        "complete_order_and_send",
        (onramper.clone(), 1_000_000u64, None::<RuneID>, false),
    )
    .expect("Failed to transfer funds");
    assert!(tx_id.is_ok(), "Failed to execute transfer: {:?}", tx_id);

    ic_cdk::println!("Transfer succeeded. TX ID: {:?}", tx_id.unwrap());

    // Step 4: Verify vault state
    let offramper_vault: Result<VaultEntry> = query_call(
        &env.pic,
        env.canister_id,
        "get_offramper_deposits",
        (offramper,),
    )
    .expect("Failed to query offramper vault");
    assert!(offramper_vault.is_err(), "Offramper vault should be empty");

    let onramper_vault: Result<VaultEntry> = query_call(
        &env.pic,
        env.canister_id,
        "get_onramper_deposits",
        (onramper,),
    )
    .expect("Failed to query onramper vault");
    assert!(onramper_vault.is_err(), "Onramper vault should be empty");
}
