use candid::{Encode, Principal};
use icramp_types::solana::{
    ed25519::Ed25519KeyName,
    setup::{InitArg, InstallArg},
};
use pocket_ic::{PocketIc, PocketIcBuilder};
use sol_rpc_types::{SolanaCluster, SupportedRpcProviderId};
use std::path::PathBuf;

// const SOLANA_BACKEND_WASM: &str = "../../target/wasm32-unknown-unknown/release/solana_backend.wasm";
// const SOL_RPC_WASM_GZ: &str = "../fixtures/wasm/sol_rpc_canister.wasm.gz";
pub const SOL_RPC_CANISTER_ID: &str = "tghme-zyaaa-aaaar-qarca-cai";
const INIT_CYCLES: u128 = 2_000_000_000_000;

const SOL_RPC_WASM_GZ_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../fixtures/wasm/sol_rpc_canister.wasm.gz"
));

const ENV_SOLANA_BACKEND_WASM: &str = "SOLANA_BACKEND_WASM";

fn read_backend_wasm() -> Vec<u8> {
    if let Ok(p) = std::env::var(ENV_SOLANA_BACKEND_WASM) {
        return std::fs::read(&p).unwrap_or_else(|e| {
            panic!(
                "SOLANA_BACKEND_WASM env set to {:?} but failed to read: {}",
                p, e
            )
        });
    }

    // Resolve from this crate’s directory: <repo>/tests/solana_tests
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // Workspace root = tests/solana_tests/.. /..
    let workspace_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("failed to resolve workspace root from CARGO_MANIFEST_DIR");

    let candidate =
        workspace_root.join("target/wasm32-unknown-unknown/release/solana_backend.wasm");

    std::fs::read(&candidate).unwrap_or_else(|_| {
        panic!(
            "Backend wasm not found at {:?}.\n\
             Build it first:\n  \
             cargo build -p solana_backend --release --target wasm32-unknown-unknown\n\
             Or set {} to an explicit path.",
            candidate, ENV_SOLANA_BACKEND_WASM
        )
    })
}

pub(crate) fn setup_solana_backend() -> (PocketIc, Principal) {
    let _ = dotenvy::dotenv();
    unsafe { std::env::set_var("POCKET_IC_BIN", "/usr/local/bin/pocket-ic") };

    let pic = PocketIcBuilder::new()
        .with_ii_subnet()
        .with_application_subnet()
        .build();

    deploy_sol_rpc_canister(&pic);

    let canister_id = pic.create_canister();
    let wasm = read_backend_wasm();

    let sol_rpc_canister_id = Principal::from_text(SOL_RPC_CANISTER_ID).unwrap();
    let arg = InstallArg::Reinstall(InitArg {
        sol_rpc_canister_id: Some(sol_rpc_canister_id),
        ed25519_key_name: Ed25519KeyName::LocalDevelopment,
        network: SolanaCluster::Devnet,
        proxy_url: "https://example.xyz".to_string(),
    });
    let arg = Encode!(&arg).expect("Failed to encode init args");

    pic.add_cycles(canister_id, INIT_CYCLES);
    pic.install_canister(canister_id, wasm, arg, None);

    ic_cdk::println!("Deployed canister ID: {}", canister_id);
    (pic, canister_id)
}

fn deploy_sol_rpc_canister(pic: &PocketIc) {
    let nns_root_canister_id: Principal =
        Principal::from_text("r7inp-6aaaa-aaaaa-aaabq-cai").unwrap();
    let sol_rpc_canister_id = Principal::from_text(SOL_RPC_CANISTER_ID).unwrap();
    let actual_canister_id = pic
        .create_canister_with_id(Some(nns_root_canister_id), None, sol_rpc_canister_id)
        .unwrap();
    assert_eq!(actual_canister_id, sol_rpc_canister_id);

    let sol_wasm = SOL_RPC_WASM_GZ_BYTES.to_vec();
    let install_args = sol_rpc_types::InstallArgs::default();

    pic.add_cycles(sol_rpc_canister_id, INIT_CYCLES);
    pic.install_canister(
        sol_rpc_canister_id,
        sol_wasm,
        Encode!(&install_args).unwrap(),
        Some(nns_root_canister_id),
    );

    if let (Ok(ankr), Ok(alchemy)) = (
        std::env::var("SOL_ANKR_KEY"),
        std::env::var("SOL_ALCHEMY_KEY"),
    ) {
        let update_args = vec![
            (SupportedRpcProviderId::AlchemyDevnet, Some(alchemy)),
            (SupportedRpcProviderId::AnkrDevnet, Some(ankr)),
        ];
        pic.update_call(
            sol_rpc_canister_id,
            nns_root_canister_id,
            "updateApiKeys",
            Encode!(&update_args).unwrap(),
        )
        .expect("updateApiKeys failed");
    }
}
