use candid::Principal;
use ic_btc_interface::Network;

use crate::memory::heap::{
    config::{get_derivation_path, get_key_name, get_network},
    state::read_state,
};

#[derive(Clone)]
pub struct WalletConfig {
    pub key_name: String,
    pub network: Network,
    pub derivation_path: Vec<Vec<u8>>,
    pub btc_principal: Principal,
}

pub enum AddressType {
    P2PKH,
    P2TRRawKey,
    P2TRScript,
}

impl WalletConfig {
    fn new(address_type: AddressType) -> Self {
        let mut derivation_path = get_derivation_path();
        match address_type {
            AddressType::P2PKH => (),
            AddressType::P2TRRawKey => derivation_path.push(b"key_spend".to_vec()),
            AddressType::P2TRScript => {
                get_derivation_path().push(b"script_spend".to_vec());
            }
        }
        let btc_principal = read_state(|s| s.btc_principal);

        Self {
            key_name: get_key_name(),
            network: get_network(),
            derivation_path,
            btc_principal,
        }
    }

    pub fn for_p2pkh() -> Self {
        Self::new(AddressType::P2PKH)
    }

    pub fn for_p2tr_raw_key() -> Self {
        Self::new(AddressType::P2TRRawKey)
    }

    pub fn for_p2tr_script() -> Self {
        Self::new(AddressType::P2TRScript)
    }
}
