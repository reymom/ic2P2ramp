use candid::Principal;
use lazy_static::lazy_static;
use pocket_ic::PocketIc;
use std::sync::Mutex;

use crate::common::setup::setup_bitcoin_backend;

lazy_static! {
    pub static ref BITCOIN_ENV: Mutex<Option<BitcoinTestEnv>> = Mutex::new(None);
}

pub struct BitcoinTestEnv {
    pub pic: PocketIc,
    pub canister_id: Principal,
    pub btc_addresses: BitcoinAddresses,
}

#[derive(Default)]
pub struct BitcoinAddresses {
    pub p2pkh_address: Option<String>,
    pub p2tr_raw_key_address: Option<String>,
    pub p2tr_script_spend_address: Option<String>,
}

pub fn get_bitcoin_env() -> std::sync::MutexGuard<'static, Option<BitcoinTestEnv>> {
    let mut env = BITCOIN_ENV
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    if env.is_none() {
        *env = Some(BitcoinTestEnv::new());
    }

    env
}

impl BitcoinTestEnv {
    pub fn new() -> Self {
        let (pic, canister_id) = setup_bitcoin_backend();

        Self {
            pic,
            canister_id,
            btc_addresses: BitcoinAddresses::default(),
        }
    }

    pub fn print_canister_logs(&self) {
        let logs = self
            .pic
            .fetch_canister_logs(self.canister_id, candid::Principal::anonymous())
            .expect("Failed to fetch logs");

        for log in logs {
            let content =
                String::from_utf8(log.content).unwrap_or_else(|_| "<Invalid UTF-8>".to_string());
            ic_cdk::println!("Log [{}]: {}", log.timestamp_nanos, content);
        }
    }
}

impl BitcoinAddresses {
    pub fn set_p2pkh_address(&mut self, address: &str) {
        self.p2pkh_address = Some(address.to_string());
    }

    pub fn set_p2tr_raw_key_address(&mut self, address: &str) {
        self.p2tr_raw_key_address = Some(address.to_string());
    }

    pub fn set_p2tr_script_spend_address(&mut self, address: &str) {
        self.p2tr_script_spend_address = Some(address.to_string());
    }
}
