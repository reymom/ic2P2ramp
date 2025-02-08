use crate::{
    common::helpers::{generate_new_bitcoin_address, mine_blocks, tick_many, update_call},
    integration::env::get_bitcoin_env,
};

use bitcoin_backend::{errors::Result, RuneID};

#[test]
fn test_bitcoin_canister_init() {
    let binding = get_bitcoin_env();
    let env = binding.as_ref().unwrap();

    let query_stats = env
        .pic
        .canister_status(env.canister_id, None)
        .unwrap()
        .query_stats;
    assert_eq!(query_stats.num_calls_total, candid::Nat::from(0u32));

    assert!(
        !env.canister_id.to_string().is_empty(),
        "Canister ID is empty!"
    );
}

#[test]
fn test_get_p2pkh_address_balance() {
    let mut binding = get_bitcoin_env();
    let env = binding.as_mut().unwrap();

    // Fetch the addresses
    let p2pkh: Result<String> = update_call(&mut env.pic, env.canister_id, "get_p2pkh_address", ())
        .expect("Failed to get P2PKH address");

    assert!(p2pkh.is_ok(), "Failed to get P2PKH address: {:?}", p2pkh);

    let p2pkh_address = p2pkh.unwrap();
    assert!(!p2pkh_address.is_empty(), "P2PKH address is empty");
    ic_cdk::println!("p2pkh_address = {:?}", p2pkh_address);
    env.btc_addresses.set_p2pkh_address(&p2pkh_address);

    let balance: Result<u64> = update_call(
        &mut env.pic,
        env.canister_id,
        "get_btc_balance",
        (p2pkh_address,),
    )
    .expect("Query failed");

    assert!(
        balance.is_ok(),
        "Failed to get bitcoin balance: {:?}",
        balance
    );

    let balance = balance.unwrap();
    assert_eq!(balance, 0);
}

#[test]
fn test_get_p2tr_raw_key_spend_address_balance() {
    let mut binding = get_bitcoin_env();
    let env = binding.as_mut().unwrap();

    let p2tr_raw_key_spend: Result<String> = update_call(
        &mut env.pic,
        env.canister_id,
        "get_p2tr_raw_key_spend_address",
        (),
    )
    .expect("Failed to get P2TR raw key spend address");

    assert!(
        p2tr_raw_key_spend.is_ok(),
        "Failed to get P2TR raw key spend address: {:?}",
        p2tr_raw_key_spend
    );

    let p2tr_raw_key_spend_address = p2tr_raw_key_spend.unwrap();
    assert!(
        !p2tr_raw_key_spend_address.is_empty(),
        "P2PTR raw key spend address is empty"
    );
    ic_cdk::println!(
        "p2tr_raw_key_spend_address = {:?}",
        p2tr_raw_key_spend_address
    );
    env.btc_addresses
        .set_p2tr_raw_key_address(&p2tr_raw_key_spend_address);

    let balance: Result<u64> = update_call(
        &mut env.pic,
        env.canister_id,
        "get_btc_balance",
        (p2tr_raw_key_spend_address,),
    )
    .expect("Query failed");

    assert!(
        balance.is_ok(),
        "Failed to get bitcoin balance: {:?}",
        balance
    );

    let balance = balance.unwrap();
    assert_eq!(balance, 0);
}

#[test]
fn test_get_p2tr_script_spend_address_balance() {
    let mut binding = get_bitcoin_env();
    let env = binding.as_mut().unwrap();

    let p2tr_script_spend: Result<String> = update_call(
        &mut env.pic,
        env.canister_id,
        "get_p2tr_script_spend_address",
        (),
    )
    .expect("Failed to get P2TR script spend address");

    assert!(
        p2tr_script_spend.is_ok(),
        "Failed to get P2TR script spend address: {:?}",
        p2tr_script_spend
    );

    let p2tr_script_spend_address = p2tr_script_spend.unwrap();
    assert!(
        !p2tr_script_spend_address.is_empty(),
        "P2TR script spend address is empty"
    );
    ic_cdk::println!(
        "p2tr_script_spend_address = {:?}",
        p2tr_script_spend_address
    );
    env.btc_addresses
        .set_p2tr_script_spend_address(&p2tr_script_spend_address);

    let balance: Result<u64> = update_call(
        &mut env.pic,
        env.canister_id,
        "get_btc_balance",
        (p2tr_script_spend_address.clone(),),
    )
    .expect("Query failed");

    assert!(
        balance.is_ok(),
        "Failed to get bitcoin balance: {:?}",
        balance
    );

    let balance = balance.unwrap();
    assert_eq!(balance, 0);
}

#[test]
fn test_use_stored_p2pkh_address() {
    let env = get_bitcoin_env();
    let env = env.as_ref().unwrap();

    let p2pkh_address = env.btc_addresses.p2pkh_address.as_ref();
    assert!(p2pkh_address.is_some(), "Failed to store p2pkh address");
}

#[test]
fn test_transfer_to_p2pkh_address() {
    let mut binding = get_bitcoin_env();
    let env = binding.as_mut().unwrap();

    let p2pkh_address = env
        .btc_addresses
        .p2pkh_address
        .as_ref()
        .expect("P2PKH address not set");
    ic_cdk::println!("p2pkh_address = {:?}", p2pkh_address);

    // Generate a new bitcoind address
    let new_address = generate_new_bitcoin_address("http://127.0.0.1:18443", "icp", "test");
    assert!(
        !new_address.is_empty(),
        "Failed to generate a new Bitcoin address"
    );
    ic_cdk::println!("New Bitcoin address: {:?}", new_address);

    // Transfer should fail without funding the canister address
    let tx_id: Result<String> = update_call(
        &mut env.pic,
        env.canister_id,
        "test_transfer",
        (new_address.clone(), 100_000u64, None::<RuneID>, false),
    )
    .expect("Failed to query test transfer");
    assert!(tx_id.is_err());

    // Deposit Bitcoin to the address via mining
    mine_blocks(&p2pkh_address, 101);

    // Allow canister to process mined blocks
    tick_many(&mut env.pic, 1500);

    // Check initial balance
    let balance: Result<u64> = update_call(
        &mut env.pic,
        env.canister_id,
        "get_btc_balance",
        (p2pkh_address.clone(),),
    )
    .expect("Failed to query balance");

    assert!(
        balance.is_ok(),
        "Failed to get bitcoin balance: {:?}",
        balance
    );

    let balance = balance.unwrap();
    ic_cdk::println!("P2PKH Balance before transfer: {} sats", balance);
    assert!(balance > 0, "Balance should be greater than 0 after mining");

    // Perform transfer
    let amount = 5_000_000u64;
    let tx_id: Result<String> = update_call(
        &mut env.pic,
        env.canister_id,
        "test_transfer",
        (new_address.clone(), amount, None::<RuneID>, false),
    )
    .expect("Failed to query test transfer");
    assert!(tx_id.is_ok(), "Failed to execute transfer: {:?}", tx_id);

    // Check updated balance
    let mine_address = generate_new_bitcoin_address("http://127.0.0.1:18443", "icp", "test");
    assert!(
        !mine_address.is_empty(),
        "Failed to generate a new Bitcoin address"
    );
    mine_blocks(&mine_address, 101);
    tick_many(&mut env.pic, 1500);

    let updated_balance: Result<u64> = update_call(
        &mut env.pic,
        env.canister_id,
        "get_btc_balance",
        (p2pkh_address.clone(),),
    )
    .expect("Failed to query balance after transfer");
    assert!(
        updated_balance.is_ok(),
        "Failed to get bitcoin balance: {:?}",
        updated_balance
    );
    let updated_balance = updated_balance.unwrap();
    ic_cdk::println!("P2PKH Balance after transfer: {} sats", balance);
    // assert!(
    //     updated_balance < balance,
    //     "Updated balance should be less than initial balance"
    // );

    let btc_balance: Result<u64> = update_call(
        &mut env.pic,
        env.canister_id,
        "get_btc_balance",
        (new_address.clone(),),
    )
    .expect("Failed to query balance after transfer");
    assert!(
        btc_balance.is_ok(),
        "Failed to get bitcoin balance: {:?}",
        btc_balance
    );
    let btc_balance = btc_balance.unwrap();
    ic_cdk::println!("Bitcoind Balance: {} sats", btc_balance);

    ic_cdk::println!(
        "Transfer succeeded. TX ID: {:?}, Updated Balance: {} sats",
        tx_id.unwrap(),
        updated_balance
    );

    env.print_canister_logs();
}

// #[test]
// fn test_transfer_to_taproot_address() {
//     let mut binding = get_bitcoin_env();
//     let env = binding.as_mut().unwrap();

//     let p2tr_address = env
//         .btc_addresses
//         .p2tr_raw_key_address
//         .as_ref()
//         .expect("P2TR address not set");
//     ic_cdk::println!("p2tr_address = {:?}", p2tr_address);

//     // Generate a new bitcoind address
//     let new_address = generate_new_bitcoin_address("http://127.0.0.1:18443", "icp", "test");
//     assert!(
//         !new_address.is_empty(),
//         "Failed to generate a new Bitcoin address"
//     );
//     ic_cdk::println!("New Bitcoin address: {:?}", new_address);

//     // Transfer should fail without funding the canister address
//     let tx_id: Result<String> = update_call(
//         &mut env.pic,
//         env.canister_id,
//         "test_transfer",
//         (new_address.clone(), 1_000u64, None::<RuneID>, false),
//     )
//     .expect("Failed to query test transfer");
//     assert!(tx_id.is_err());

//     // Deposit Bitcoin to the address via mining
//     mine_blocks(&p2tr_address, 101);

//     // Allow canister to process mined blocks
//     tick_many(&mut env.pic, 1250);

//     // Check initial balance
//     let balance: Result<u64> = update_call(
//         &mut env.pic,
//         env.canister_id,
//         "get_btc_balance",
//         (p2tr_address.clone(),),
//     )
//     .expect("Failed to query balance");

//     assert!(
//         balance.is_ok(),
//         "Failed to get bitcoin balance: {:?}",
//         balance
//     );

//     let balance = balance.unwrap();
//     ic_cdk::println!("P2TR Balance before transfer: {} sats", balance);
//     assert!(balance > 0, "Balance should be greater than 0 after mining");

//     // Perform transfer
//     let amount = 5_000_000u64;
//     let tx_id: Result<String> = update_call(
//         &mut env.pic,
//         env.canister_id,
//         "test_transfer",
//         (new_address.clone(), amount, None::<RuneID>, false),
//     )
//     .expect("Failed to query test transfer");
//     assert!(tx_id.is_ok(), "Failed to execute transfer: {:?}", tx_id);

//     // Check updated balance
//     let mine_address = generate_new_bitcoin_address("http://127.0.0.1:18443", "icp", "test");
//     assert!(
//         !mine_address.is_empty(),
//         "Failed to generate a new Bitcoin address"
//     );
//     mine_blocks(&mine_address, 101);
//     tick_many(&mut env.pic, 1500);

//     let updated_balance: Result<u64> = update_call(
//         &mut env.pic,
//         env.canister_id,
//         "get_btc_balance",
//         (p2tr_address.clone(),),
//     )
//     .expect("Failed to query balance after transfer");
//     assert!(
//         updated_balance.is_ok(),
//         "Failed to get bitcoin balance: {:?}",
//         updated_balance
//     );
//     let updated_balance = updated_balance.unwrap();
//     ic_cdk::println!("P2PKH Balance after transfer: {} sats", balance);
//     // assert!(
//     //     updated_balance < balance,
//     //     "Updated balance should be less than initial balance"
//     // );

//     let btc_balance: Result<u64> = update_call(
//         &mut env.pic,
//         env.canister_id,
//         "get_btc_balance",
//         (new_address.clone(),),
//     )
//     .expect("Failed to query balance after transfer");
//     assert!(
//         btc_balance.is_ok(),
//         "Failed to get bitcoin balance: {:?}",
//         btc_balance
//     );
//     let btc_balance = btc_balance.unwrap();
//     ic_cdk::println!("Bitcoind Balance: {} sats", btc_balance);

//     ic_cdk::println!(
//         "Transfer succeeded. TX ID: {:?}, Updated Balance: {} sats",
//         tx_id.unwrap(),
//         updated_balance
//     );

//     env.print_canister_logs();
// }
