use ethers_core::abi::ethereum_types::{Address, U256, U64};
use ethers_core::types::transaction::eip1559::Eip1559TransactionRequest;
use ethers_core::types::{Bytes, Signature};
use ethers_core::utils::keccak256;

use ic_cdk::api::management_canister::ecdsa::{
    ecdsa_public_key, sign_with_ecdsa, EcdsaPublicKeyArgument, SignWithEcdsaArgument,
};
use std::str::FromStr;

use crate::evm_rpc::{
    MultiSendRawTransactionResult, RpcConfig, RpcServices, SendRawTransactionResult,
    SendRawTransactionStatus, CANISTER_ID,
};
use crate::state::read_state;

pub struct SignRequest {
    pub chain_id: Option<U64>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub gas: U256,
    pub max_fee_per_gas: Option<U256>,
    pub max_priority_fee_per_gas: Option<U256>,
    pub value: Option<U256>,
    pub nonce: Option<U256>,
    pub data: Option<Vec<u8>>,
}

pub struct FeeEstimates {
    pub max_fee_per_gas: U256,
    pub max_priority_fee_per_gas: U256,
}

pub fn get_fee_estimates() -> FeeEstimates {
    let max_fee_per_gas = U256::from(100_000_000_000u64);
    let max_priority_fee_per_gas = U256::from(2_000_000_000);

    FeeEstimates {
        max_fee_per_gas,
        max_priority_fee_per_gas,
    }
}

pub async fn create_sign_request(
    value: U256,
    to: Option<String>,
    from: Option<String>,
    gas: U256,
    data: Option<Vec<u8>>,
    fee_estimates: FeeEstimates,
) -> SignRequest {
    let FeeEstimates {
        max_fee_per_gas,
        max_priority_fee_per_gas,
    } = fee_estimates;
    let nonce = read_state(|s| s.nonce);
    let rpc_providers = read_state(|s| s.rpc_services.clone());

    ic_cdk::println!(
        "[create_sign_request] rpc_providers.chain_id()) = {}",
        rpc_providers.chain_id()
    );
    SignRequest {
        chain_id: Some(rpc_providers.chain_id()),
        to,
        from,
        gas,
        max_fee_per_gas: Some(max_fee_per_gas),
        max_priority_fee_per_gas: Some(max_priority_fee_per_gas),
        data,
        value: Some(value),
        nonce: Some(nonce),
    }
}

pub async fn sign_transaction(req: SignRequest) -> String {
    const EIP1559_TX_ID: u8 = 2;

    let data = req.data.as_ref().map(|d| Bytes::from(d.clone()));

    let tx = Eip1559TransactionRequest {
        from: req
            .from
            .map(|from| Address::from_str(&from).expect("failed to parse the source address")),
        to: req.to.map(|from| {
            Address::from_str(&from)
                .expect("failed to parse the source address")
                .into()
        }),
        gas: Some(req.gas),
        value: req.value,
        data,
        nonce: req.nonce,
        access_list: Default::default(),
        max_priority_fee_per_gas: req.max_priority_fee_per_gas,
        max_fee_per_gas: req.max_fee_per_gas,
        chain_id: req.chain_id,
    };

    ic_cdk::println!("[sign_transaction] tx = {:?}", tx);

    let mut unsigned_tx_bytes = tx.rlp().to_vec();
    unsigned_tx_bytes.insert(0, EIP1559_TX_ID);

    let txhash = keccak256(&unsigned_tx_bytes);

    let key_id = read_state(|s| s.ecdsa_key_id.clone());

    let signature = sign_with_ecdsa(SignWithEcdsaArgument {
        message_hash: txhash.to_vec(),
        derivation_path: [].to_vec(),
        key_id,
    })
    .await
    .expect("failed to sign the transaction")
    .0
    .signature;

    let pubkey = read_state(|s| (s.ecdsa_pub_key.clone())).expect("public key should be set");

    let signature = Signature {
        v: y_parity(&txhash, &signature, &pubkey),
        r: U256::from_big_endian(&signature[0..32]),
        s: U256::from_big_endian(&signature[32..64]),
    };

    let mut signed_tx_bytes = tx.rlp_signed(&signature).to_vec();
    signed_tx_bytes.insert(0, EIP1559_TX_ID);

    format!("0x{}", hex::encode(&signed_tx_bytes))
}

fn y_parity(prehash: &[u8], sig: &[u8], pubkey: &[u8]) -> u64 {
    use ethers_core::k256::ecdsa::{RecoveryId, Signature, VerifyingKey};

    let orig_key = VerifyingKey::from_sec1_bytes(pubkey).expect("failed to parse the pubkey");
    let signature = Signature::try_from(sig).unwrap();
    for parity in [0u8, 1] {
        let recid = RecoveryId::try_from(parity).unwrap();
        let recovered_key = VerifyingKey::recover_from_prehash(prehash, &signature, recid)
            .expect("failed to recover key");
        if recovered_key == orig_key {
            return parity as u64;
        }
    }

    panic!(
        "failed to recover the parity bit from a signature; sig: {}, pubkey: {}",
        hex::encode(sig),
        hex::encode(pubkey)
    )
}

pub async fn send_raw_transaction(tx: String) -> SendRawTransactionStatus {
    let rpc_providers = read_state(|s| s.rpc_services.clone());
    let cycles = 10_000_000_000;

    ic_cdk::println!("[send_raw_transaction] rpc_providers = {:?}", rpc_providers);
    let arg2: Option<RpcConfig> = None;
    let res = ic_cdk::api::call::call_with_payment128(
        CANISTER_ID,
        "eth_sendRawTransaction",
        (rpc_providers, arg2, tx),
        cycles,
    )
    .await;
    match res {
        Ok((res,)) => match res {
            MultiSendRawTransactionResult::Consistent(status) => match status {
                SendRawTransactionResult::Ok(status) => status,
                SendRawTransactionResult::Err(e) => {
                    ic_cdk::trap(format!("Error: {:?}", e).as_str());
                }
            },
            MultiSendRawTransactionResult::Inconsistent(_) => {
                ic_cdk::trap("Status is inconsistent");
            }
        },
        Err(e) => ic_cdk::trap(format!("Error: {:?}", e).as_str()),
    }
}

impl RpcServices {
    pub fn chain_id(&self) -> U64 {
        match self {
            RpcServices::EthSepolia(_) => U64::from(11155111),
            RpcServices::Custom {
                chainId,
                services: _,
            } => U64::from(*chainId),
            RpcServices::EthMainnet(_) => U64::from(1),
        }
    }
}

pub async fn get_public_key() -> Vec<u8> {
    let key_id = read_state(|s| s.ecdsa_key_id.clone());

    let (key,) = ecdsa_public_key(EcdsaPublicKeyArgument {
        canister_id: None,
        derivation_path: [].to_vec(),
        key_id,
    })
    .await
    .expect("failed to get public key");
    key.public_key
}

pub fn pubkey_bytes_to_address(pubkey_bytes: &[u8]) -> String {
    use ethers_core::k256::elliptic_curve::sec1::ToEncodedPoint;
    use ethers_core::k256::PublicKey;

    let key =
        PublicKey::from_sec1_bytes(pubkey_bytes).expect("failed to parse the public key as SEC1");
    let point = key.to_encoded_point(false);
    // we re-encode the key to the decompressed representation.
    let point_bytes = point.as_bytes();
    assert_eq!(point_bytes[0], 0x04);

    let hash = keccak256(&point_bytes[1..]);

    ethers_core::utils::to_checksum(&Address::from_slice(&hash[12..32]), None)
}
