use ic_cdk::api::management_canister::bitcoin::BitcoinNetwork;

use crate::memory::heap::config::{DERIVATION_PATH, KEY_NAME, NETWORK};

pub enum TaprootUseCase {
    Standard,
    RuneTransfer(String),
    Inscription(Vec<u8>),
}

#[derive(Clone)]
pub struct WalletConfig {
    pub key_name: String,
    pub network: BitcoinNetwork,
    pub derivation_path: Vec<Vec<u8>>,
}

pub enum AddressType {
    P2PKH,
    P2TRRawKey,
    P2TRScript,
}

impl WalletConfig {
    fn new(address_type: AddressType) -> Self {
        let mut derivation_path = DERIVATION_PATH.with(|d| d.clone());
        match address_type {
            AddressType::P2PKH => (),
            AddressType::P2TRRawKey => derivation_path.push(b"key_spend".to_vec()),
            AddressType::P2TRScript => {
                derivation_path.push(b"script_spend".to_vec());
            }
        }

        Self {
            key_name: KEY_NAME.with(|kn| kn.borrow().to_string()),
            network: NETWORK.with(|n| n.get()),
            derivation_path,
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
