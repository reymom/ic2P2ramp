use crate::common::{
    bitcoin::{
        generate_new_bitcoin_address, get_ord_rune_balance, mine_blocks, send_runes_from_ord,
        track_block_sync,
    },
    helpers::update_call,
};
use crate::integration::env::get_bitcoin_env;

use icramp_types::bitcoin::{
    errors::{BitcoinError, Result},
    inscription::Inscription,
    runes::{Etching, RuneID, RuneMetadata},
    transfer::TransactionType,
};

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

    let p2pkh_address = env.get_p2pkh_address();
    assert!(!p2pkh_address.is_empty(), "P2PKH address is empty");
    ic_cdk::println!("p2pkh_address = {:?}", p2pkh_address);

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

    let p2tr_raw_key_spend_address = env.get_p2tr_raw_key_spend_address();
    assert!(
        !p2tr_raw_key_spend_address.is_empty(),
        "P2PTR raw key spend address is empty"
    );
    ic_cdk::println!(
        "p2tr_raw_key_spend_address = {:?}",
        p2tr_raw_key_spend_address
    );

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
fn test_get_p2tr_script_spend_rune_etching() {
    let mut binding = get_bitcoin_env();
    let env = binding.as_mut().unwrap();

    let rune_in = RuneMetadata {
        id: RuneID::new(146, 1).expect("error constructing runeID"),
        name: "ZZZ•DOG•TO•THE•MOON".to_string(),
        symbol: "🐕".to_string(),
        divisibility: 0,
        cap: 90_u128,
        premine: 1_000_u128,
    };

    let p2tr_script_spend_address =
        env.get_p2tr_script_spend_address(TransactionType::RuneEtching(Etching {
            metadata: rune_in,
            spacers: None,
            amount: None,
            height: None,
            offset: None,
        }));
    assert!(
        !p2tr_script_spend_address.is_empty(),
        "P2TR script spend address is empty"
    );
    ic_cdk::println!(
        "p2tr_script_spend_address = {:?}",
        p2tr_script_spend_address
    );

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
fn test_get_p2tr_script_spend_inscription() {
    let mut binding = get_bitcoin_env();
    let env = binding.as_mut().unwrap();

    let p2tr_script_spend_address =
        env.get_p2tr_script_spend_address(TransactionType::OrdinalInscription(Inscription {
            content: "dummy content".to_string(),
            content_type: "dummy content type".to_string(),
            metadata: None,
        }));
    assert!(
        !p2tr_script_spend_address.is_empty(),
        "P2TR script spend address is empty"
    );
    ic_cdk::println!(
        "p2tr_script_spend_address = {:?}",
        p2tr_script_spend_address
    );

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
fn test_transfer_to_p2pkh_address() {
    test_btc_transfer(TransactionType::LegacyBitcoin)
}

#[test]
fn test_transfer_to_taproot_address() {
    test_btc_transfer(TransactionType::TaprootBitcoin)
}

fn test_btc_transfer(tx_type: TransactionType) {
    let mut binding = get_bitcoin_env();
    let env = binding.as_mut().unwrap();

    let own_address = match tx_type {
        TransactionType::LegacyBitcoin => env.get_p2pkh_address(),
        TransactionType::TaprootBitcoin | TransactionType::RuneTransfer(_) => {
            env.get_p2tr_raw_key_spend_address()
        }
        _ => panic!("Not supported"),
    };
    ic_cdk::println!("own_address = {:?}", own_address);

    // Generate a new bitcoind address
    let new_address = generate_new_bitcoin_address();
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
        (new_address.clone(), 1_000u64, tx_type.clone()),
    )
    .expect("Failed to query test transfer");
    assert!(matches!(
        tx_id.unwrap_err(),
        BitcoinError::InsufficientBalance(_)
    ));

    // Deposit Bitcoin to the address via mining
    let blocks_to_mine = 101;
    mine_blocks(&own_address, blocks_to_mine);

    // Allow canister to process mined blocks
    track_block_sync(&mut env.pic, 101);

    // Check initial balance
    let balance: Result<u64> = update_call(
        &mut env.pic,
        env.canister_id,
        "get_btc_balance",
        (own_address.clone(),),
    )
    .expect("Failed to query balance");

    assert!(balance.is_ok(), "Failed to get balance: {:?}", balance);

    // Perform transfer
    let amount = 100_000u64;
    let tx_id: Result<String> = update_call(
        &mut env.pic,
        env.canister_id,
        "test_transfer",
        (new_address.clone(), amount, tx_type.clone()),
    )
    .expect("Failed to query test transfer");
    assert!(tx_id.is_ok(), "Failed to execute transfer: {:?}", tx_id);
    ic_cdk::println!("Transfer succeeded. TX ID: {:?}", tx_id.clone().unwrap());

    // Check updated balance
    let mine_address = generate_new_bitcoin_address();
    assert!(
        !mine_address.is_empty(),
        "Failed to generate a new Bitcoin address"
    );

    // Deposit Bitcoin to the address via mining
    mine_blocks(&mine_address, blocks_to_mine);

    // Allow canister to process mined blocks
    track_block_sync(&mut env.pic, 101);

    let updated_balance: Result<u64> = update_call(
        &mut env.pic,
        env.canister_id,
        "get_btc_balance",
        (own_address.clone(),),
    )
    .expect("Failed to query balance after transfer");
    assert!(
        updated_balance.is_ok(),
        "Failed to get bitcoin balance: {:?}",
        updated_balance
    );
    let updated_balance: u64 = updated_balance.unwrap();
    ic_cdk::println!("Balance after transfer: {} sats", updated_balance);
    // assert!(
    //     updated_balance. < balance.unwrap(),
    //     "Updated balance should be less than initial balance"
    // );

    // to do, get the rune balance conditionally
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
        updated_balance,
    );

    env.print_canister_logs();
}

#[test]
fn test_transfer_runes() {
    let mut binding = get_bitcoin_env();
    let env = binding.as_mut().unwrap();

    let rune_id = env.rune.clone().unwrap();
    let rune_name = "ZZZ•DOG•TO•THE•MOON";

    let own_address = env.get_p2tr_raw_key_spend_address();
    ic_cdk::println!("Taproot address = {:?}", own_address);

    // Generate a new bitcoind address
    let new_address = generate_new_bitcoin_address();
    assert!(
        !new_address.is_empty(),
        "Failed to generate a new Bitcoin address"
    );
    ic_cdk::println!("New Bitcoin address: {:?}", new_address);

    // Fund the canister with some runes
    let amount = 50u64;
    let success = send_runes_from_ord(&own_address, rune_name, amount);
    assert!(success, "Failed to send runes from ord");

    // Deposit Bitcoin to the address via mining
    let blocks_to_mine = 101;
    mine_blocks(&own_address, blocks_to_mine);

    // Allow canister to process mined blocks
    track_block_sync(&mut env.pic, 101);

    // Check initial rune balance in the canister
    let balance: Result<u64> = update_call(
        &mut env.pic,
        env.canister_id,
        "get_canister_rune_amount",
        (rune_id.clone(),),
    )
    .expect("Failed to query rune balance");
    assert!(balance.is_ok(), "Failed to get rune balance: {:?}", balance);
    let balance: u64 = balance.unwrap();
    assert_eq!(balance, amount, "balance is not properly set");

    // Perform transfer
    let tx_id: Result<String> = update_call(
        &mut env.pic,
        env.canister_id,
        "test_transfer",
        (
            new_address.clone(),
            amount,
            TransactionType::RuneTransfer(rune_id.clone()),
        ),
    )
    .expect("Failed to query test transfer");
    assert!(tx_id.is_ok(), "Failed to execute transfer: {:?}", tx_id);
    ic_cdk::println!("Transfer succeeded. TX ID: {:?}", tx_id.clone().unwrap());

    // Check updated balance
    let mine_address = generate_new_bitcoin_address();
    assert!(
        !mine_address.is_empty(),
        "Failed to generate a new Bitcoin address"
    );

    mine_blocks(&mine_address, blocks_to_mine);
    track_block_sync(&mut env.pic, 101);

    // Check initial rune balance
    let balance = get_ord_rune_balance(rune_name).expect("Failed to fetch rune balance");
    assert_eq!(balance, amount, "Rune balance is incorrect");

    let updated_balance: Result<u64> = update_call(
        &mut env.pic,
        env.canister_id,
        "get_canister_rune_amount",
        (rune_id,),
    )
    .expect("Failed to query balance after transfer");
    assert!(
        updated_balance.is_ok(),
        "Failed to get bitcoin balance: {:?}",
        updated_balance
    );
    let updated_balance: u64 = updated_balance.unwrap();
    ic_cdk::println!("Balance after transfer: {} sats", updated_balance);
    assert_eq!(updated_balance, 0, "Updated balance should be 0");
}
