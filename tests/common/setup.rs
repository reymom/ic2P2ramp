use std::{
    fs,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::SystemTime,
};

use candid::{Encode, Principal};
use ic_btc_interface::{Config, Network};
use ic_cdk::api::management_canister::bitcoin::BitcoinNetwork;
use pocket_ic::{PocketIc, PocketIcBuilder};

const BITCOIN_BACKEND_WASM: &str = "../target/wasm32-unknown-unknown/release/bitcoin_backend.wasm";

const _BACKEND_WASM: &str = "../target/wasm32-unknown-unknown/release/backend.wasm";

const INIT_CYCLES: u128 = 2_000_000_000_000;

pub(crate) fn setup_bitcoin_backend() -> (PocketIc, Principal) {
    std::env::set_var("POCKET_IC_BIN", "/usr/local/bin/pocket-ic");
    let pic = PocketIcBuilder::new()
        .with_bitcoin_subnet()
        .with_ii_subnet()
        .with_application_subnet()
        .with_bitcoind_addr(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            18444,
        ))
        .build();
    pic.set_time(SystemTime::now());

    let bitcoin_backend_canister = pic.create_canister();
    pic.add_cycles(bitcoin_backend_canister, INIT_CYCLES);
    let arg = Encode!(&BitcoinNetwork::Regtest).expect("Failed to encode init args");
    let wasm = fs::read(BITCOIN_BACKEND_WASM).expect("Wasm file not found, run 'dfx build'.");
    pic.install_canister(bitcoin_backend_canister, wasm, arg, None);

    deploy_bitcoin_testnet_canister(&pic);

    ic_cdk::println!("Deployed canister ID: {}", bitcoin_backend_canister);
    (pic, bitcoin_backend_canister)
}

pub fn deploy_bitcoin_testnet_canister(pic: &PocketIc) {
    // The NNS root canister should be the controller of the bitcoin testnet canister.
    let nns_root_canister_id: Principal =
        Principal::from_text("r7inp-6aaaa-aaaaa-aaabq-cai").unwrap();
    let btc_canister_id = Principal::from_text("g4xu7-jiaaa-aaaan-aaaaq-cai").unwrap();
    let actual_canister_id = pic
        .create_canister_with_id(Some(nns_root_canister_id), None, btc_canister_id)
        .unwrap();
    assert_eq!(actual_canister_id, btc_canister_id);

    let btc_wasm_path = "./wasms/ic-btc-canister.wasm.gz";
    let btc_wasm = fs::read(btc_wasm_path).expect("Failed to read Bitcoin canister wasm file");
    let args = Config {
        network: Network::Regtest,
        ..Default::default()
    };
    pic.install_canister(
        btc_canister_id,
        btc_wasm,
        Encode!(&args).unwrap(),
        Some(nns_root_canister_id),
    );
}
