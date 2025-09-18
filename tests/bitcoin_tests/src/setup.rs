use std::{
    fs,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

use candid::{Encode, Principal};
use ic_btc_interface::{Config, Network};
use icramp_types::bitcoin::setup::{InitArg, InstallArg, UnisatConfig};
use pocket_ic::{PocketIc, PocketIcBuilder};

pub const BTC_RPC_URL: &str = "http://127.0.0.1:18443";
pub const RPC_USER: &str = "icp";
pub const RPC_PASSWORD: &str = "test";
pub const BTC_CANISTER_ID: &str = "g4xu7-jiaaa-aaaan-aaaaq-cai";

const BITCOIN_BACKEND_WASM: &str =
    "../..target/wasm32-unknown-unknown/release/bitcoin_backend.wasm";
const IC_BTC_WASM_GZ: &str = "../fixtures/wasm/ic-btc-canister.wasm.gz";
const INIT_CYCLES: u128 = 2_000_000_000_000;

pub(crate) fn setup_bitcoin_backend() -> (PocketIc, Principal) {
    unsafe { std::env::set_var("POCKET_IC_BIN", "/usr/local/bin/pocket-ic") };
    let pic = PocketIcBuilder::new()
        .with_bitcoin_subnet()
        .with_ii_subnet()
        .with_application_subnet()
        .with_bitcoind_addr(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            18444,
        ))
        .build();

    let bitcoin_backend_canister = pic.create_canister();
    let wasm =
        fs::read(BITCOIN_BACKEND_WASM).expect("Bitcoin wasm file not found, run 'dfx build'.");

    let arg = InstallArg::Reinstall(InitArg {
        network: Network::Regtest,
        proxy_url: "https://example.xyz".to_string(),
        unisat: UnisatConfig {
            api_url: "https://open-api-testnet4.unisat.io".to_string(),
            api_key: std::env::var("UNISAT_API_KEY").unwrap_or_default(),
        },
    });
    let arg = Encode!(&arg).expect("Failed to encode init args");
    pic.add_cycles(bitcoin_backend_canister, INIT_CYCLES);
    pic.install_canister(bitcoin_backend_canister, wasm, arg, None);

    deploy_bitcoin_testnet_canister(&pic);

    ic_cdk::println!("Deployed canister ID: {}", bitcoin_backend_canister);
    (pic, bitcoin_backend_canister)
}

fn deploy_bitcoin_testnet_canister(pic: &PocketIc) {
    // The NNS root canister should be the controller of the bitcoin testnet canister.
    let nns_root_canister_id: Principal =
        Principal::from_text("r7inp-6aaaa-aaaaa-aaabq-cai").unwrap();
    let btc_canister_id = Principal::from_text(BTC_CANISTER_ID).unwrap();
    let actual_canister_id = pic
        .create_canister_with_id(Some(nns_root_canister_id), None, btc_canister_id)
        .unwrap();
    assert_eq!(actual_canister_id, btc_canister_id);

    let btc_wasm = fs::read(IC_BTC_WASM_GZ).expect("Failed to read Bitcoin canister wasm file");
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
