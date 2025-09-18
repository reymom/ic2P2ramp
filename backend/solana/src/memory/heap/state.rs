use candid::{CandidType, Deserialize, Principal};
use ic_ed25519::PublicKey;
use icramp_types::solana::ed25519::Ed25519KeyName;
use sol_rpc_client::{
    IcRuntime,
    ed25519::{DerivationPath, Ed25519KeyId, get_pubkey},
};
use sol_rpc_types::SolanaCluster;

use crate::{
    memory::heap::config::{mutate_state, read_state},
    solana::ed25519::Ed25519ExtendedPublicKey,
};

#[derive(Clone, CandidType, Deserialize)]
pub struct State {
    /// SOL-RPC canister we call for outcalls
    pub sol_rpc_canister_id: Principal,

    /// Which Solana cluster to hit: Devnet / Mainnet
    pub network: SolanaCluster,

    /// Name of the ICP Ed25519 key (threshold key on every node)
    pub ed25519_key_name: Ed25519KeyName,

    /// Cached root public key **as raw bytes** (32 bytes).  Optional.
    /// Keep in bytes so the struct stays Candid-friendly.
    pub ed25519_root_pk: Option<(Vec<u8>, Vec<u8>)>, // (pk, chain_code)

    /// Optional proxy URL for IPv4 HTTPS outcalls
    pub proxy_url: String,
}

pub async fn lazy_root_public_key() -> Ed25519ExtendedPublicKey {
    // in-memory cache?
    if let Some((pk_bytes, cc_bytes)) = read_state(|s| s.ed25519_root_pk.clone()) {
        let public_key = PublicKey::deserialize_raw(&pk_bytes).expect("cached pub-key corrupt");
        let chain_code: [u8; 32] = cc_bytes[..].try_into().expect("cached chain-code corrupt");
        return Ed25519ExtendedPublicKey {
            public_key,
            chain_code,
        };
    }

    // fetch from management canister
    let key_name: Ed25519KeyName = read_state(|s| s.ed25519_key_name);
    let key_id: Ed25519KeyId = key_name.into();
    let deriv_path = DerivationPath::default();
    let (pk_raw, chain_code) = get_pubkey(
        &IcRuntime,
        None, // root key, no owner-specific derivation
        Some(&deriv_path),
        key_id,
    )
    .await
    .expect("get_pubkey failed");

    let public_key =
        PublicKey::deserialize_raw(&pk_raw.to_bytes()).expect("management returned bad pub-key");

    let root = Ed25519ExtendedPublicKey {
        public_key,
        chain_code,
    };

    mutate_state(|s| {
        s.ed25519_root_pk = Some((
            root.public_key.serialize_raw().to_vec(),
            root.chain_code.to_vec(),
        ));
    });

    root
}

impl core::fmt::Debug for State {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("State")
            .field("sol_rpc_canister_id", &self.sol_rpc_canister_id.to_text())
            .field("network", &self.network)
            .field("ed25519_key_name", &self.ed25519_key_name)
            .field("ed25519_root_pk", &self.ed25519_root_pk)
            .field("proxy_url", &self.proxy_url)
            .finish()
    }
}
