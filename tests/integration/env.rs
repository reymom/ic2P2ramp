use candid::Principal;
use lazy_static::lazy_static;
use pocket_ic::PocketIc;
use std::sync::Mutex;

use bitcoin_backend::types::{RuneID, TransactionType};
use icramp_types::bitcoin::errors::Result;

use crate::common::{helpers::update_call, setup::setup_bitcoin_backend};

lazy_static! {
    pub static ref BITCOIN_ENV: Mutex<Option<BitcoinTestEnv>> = Mutex::new(None);
}

pub struct BitcoinTestEnv {
    pub pic: PocketIc,
    pub canister_id: Principal,
    pub btc_addresses: BitcoinAddresses,
    pub rune: Option<RuneID>,
}

#[derive(Default)]
pub struct BitcoinAddresses {
    p2pkh_address: Option<String>,
    p2tr_raw_key_address: Option<String>,
    p2tr_script_spend_address: Option<String>,
}

pub fn get_bitcoin_env() -> std::sync::MutexGuard<'static, Option<BitcoinTestEnv>> {
    let mut env = BITCOIN_ENV
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    if env.is_none() {
        *env = Some(BitcoinTestEnv::new(None));
    }

    env
}

impl BitcoinTestEnv {
    pub fn new(rune: Option<RuneID>) -> Self {
        let (pic, canister_id) = setup_bitcoin_backend();

        Self {
            pic,
            canister_id,
            btc_addresses: BitcoinAddresses::default(),
            rune,
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

    pub fn get_p2pkh_address(&mut self) -> String {
        if let Some(ref addr) = self.btc_addresses.p2pkh_address {
            return addr.clone();
        }

        let fetched_address: Result<String> =
            update_call(&mut self.pic, self.canister_id, "get_p2pkh_address", ())
                .expect("Failed to get P2PKH address");

        let addr = fetched_address.expect("P2PKH address fetch failed");
        self.btc_addresses.p2pkh_address = Some(addr.clone());
        addr
    }

    pub fn get_p2tr_raw_key_spend_address(&mut self) -> String {
        if let Some(ref addr) = self.btc_addresses.p2tr_raw_key_address {
            return addr.clone();
        }

        let fetched_address: Result<String> = update_call(
            &mut self.pic,
            self.canister_id,
            "get_p2tr_raw_key_spend_address",
            (),
        )
        .expect("Failed to get P2TR Raw Key Spend address");

        let addr = fetched_address.expect("P2TR Raw Key address fetch failed");
        self.btc_addresses.p2tr_raw_key_address = Some(addr.clone());
        addr
    }

    pub fn get_p2tr_script_spend_address(&mut self, tx_type: TransactionType) -> String {
        if let Some(ref addr) = self.btc_addresses.p2tr_script_spend_address {
            return addr.clone();
        }

        let fetched_address: Result<String> = update_call(
            &mut self.pic,
            self.canister_id,
            "get_p2tr_script_spend_address",
            (tx_type,),
        )
        .expect("Failed to get P2TR Script Spend address");

        let addr = fetched_address.expect("P2TR Script Spend address fetch failed");
        self.btc_addresses.p2tr_script_spend_address = Some(addr.clone());
        addr
    }
}
