use icramp_types::bitcoin::{
    errors::Result,
    runes::{RuneID, RuneMetadata},
};

use super::env::get_bitcoin_env;
use testkit::helpers::{query_call, update_call};

#[test]
fn test_register_rune() {
    let mut env = get_bitcoin_env()
        .take()
        .expect("Bitcoin env not initialized");

    let rune_in = RuneMetadata {
        id: RuneID::new(146, 1).expect("error constructing runeID"),
        name: "ZZZ•DOG•TO•THE•MOON".to_string(),
        symbol: "🐕".to_string(),
        divisibility: 0,
        cap: 90_u128,
        premine: 1_000_u128,
    };

    let rune_id = rune_in.id.clone();
    // check if rune is valid before update
    let res: Result<()> = query_call(
        &env.pic,
        env.canister_id,
        "validate_rune",
        (rune_id.clone(),),
    )
    .expect("failed to query validate rune");
    assert!(res.is_err());

    let res: Result<()> = update_call(
        &mut env.pic,
        env.canister_id,
        "register_runes",
        (vec![rune_in.clone()],),
    )
    .expect("register rune call failed");
    assert!(res.is_ok());

    // check if rune is valid after update
    let res: Result<()> = query_call(
        &env.pic,
        env.canister_id,
        "validate_rune",
        (rune_id.clone(),),
    )
    .expect("failed to query validate rune");
    assert!(res.is_ok());

    let rune_out: Result<(RuneMetadata, String)> = query_call(
        &env.pic,
        env.canister_id,
        "get_serialized_rune_metadata",
        (rune_id,),
    )
    .expect("get rune metadata call failed");
    assert!(rune_out.is_ok());
    assert_eq!(rune_in, rune_out.unwrap().0);
}

#[test]
fn test_get_registered_runes() {
    let mut env = get_bitcoin_env()
        .take()
        .expect("Bitcoin env not initialized");

    let mut expected_runes = vec![];

    // Create and register multiple runes
    for i in 1..=5 {
        let rune = RuneMetadata {
            id: RuneID::new(100 + i, i as u32).expect("error constructing runeID"),
            name: format!("RUNE•TEST•{}", i),
            symbol: format!("🔶{}", i),
            divisibility: i as u8,
            cap: (i as u128) * 1_000,
            premine: (i as u128) * 100,
        };

        let res: Result<()> = update_call(
            &mut env.pic,
            env.canister_id,
            "register_runes",
            (vec![rune.clone()],),
        )
        .expect("register rune call failed");

        assert!(res.is_ok(), "Failed to register rune {}", i);

        expected_runes.push(rune);
    }

    // Fetch all registered runes
    let registered_runes: Result<Vec<RuneMetadata>> =
        query_call(&env.pic, env.canister_id, "get_registered_runes", ())
            .expect("get registered runes call failed");

    assert!(registered_runes.is_ok(), "Failed to fetch registered runes");
    let registered_runes = registered_runes.unwrap();

    // Ensure the registered runes match the expected runes
    assert_eq!(
        expected_runes.len(),
        registered_runes.len(),
        "Mismatch in number of registered runes"
    );

    for rune in expected_runes.clone() {
        assert!(
            registered_runes.contains(&rune),
            "Expected rune {:?} not found in registered runes",
            rune
        );
    }

    let rune_in = expected_runes[expected_runes.len() - 1].clone();
    let rune_out: Result<(RuneMetadata, String)> = query_call(
        &env.pic,
        env.canister_id,
        "get_serialized_rune_metadata",
        (rune_in.clone().id,),
    )
    .expect("get rune metadata call failed");
    assert!(rune_out.is_ok());
    assert_eq!(rune_in, rune_out.unwrap().0);
}
