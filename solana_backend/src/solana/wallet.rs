use candid::Principal;
use sol_rpc_client::{
    IcRuntime,
    ed25519::{DerivationPath, sign_message},
};

use crate::{
    memory::heap::{config::read_state, state::lazy_root_public_key},
    solana::ed25519::Ed25519ExtendedPublicKey,
};
use solana_message::Message;
use solana_pubkey::Pubkey;
use solana_signature::Signature;
use std::fmt::Display;

#[derive(Clone)]
pub struct SolanaAccount {
    pub ed25519_public_key: Pubkey,
    pub derivation_path: DerivationPath,
}

impl SolanaAccount {
    pub fn new_derived_account(
        root_public_key: &Ed25519ExtendedPublicKey,
        derivation_path: DerivationPath,
    ) -> Self {
        let ed25519_public_key = root_public_key
            .derive_public_key(derivation_path.clone())
            .public_key
            .serialize_raw()
            .into();
        Self {
            ed25519_public_key,
            derivation_path,
        }
    }

    pub async fn sign_message(&self, message: &Message) -> Signature {
        sign_message(
            &IcRuntime,
            message,
            read_state(|s| s.ed25519_key_name).into(),
            Some(&self.derivation_path),
        )
        .await
        .expect("Failed to sign transaction")
    }
}

impl AsRef<Pubkey> for SolanaAccount {
    fn as_ref(&self) -> &Pubkey {
        &self.ed25519_public_key
    }
}

impl Display for SolanaAccount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            bs58::encode(&self.ed25519_public_key).into_string()
        )
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SolanaWallet {
    owner: Principal,
    root_public_key: Ed25519ExtendedPublicKey,
}

impl SolanaWallet {
    pub async fn new(owner: Principal) -> Self {
        let root_public_key = lazy_root_public_key().await;
        Self {
            owner,
            root_public_key,
        }
    }

    pub async fn new_canister() -> Self {
        Self::new(ic_cdk::id()).await
    }

    pub fn derive_account(&self, derivation_path: DerivationPath) -> SolanaAccount {
        SolanaAccount::new_derived_account(&self.root_public_key, derivation_path)
    }

    pub fn solana_account(&self) -> SolanaAccount {
        self.derive_account(self.owner.as_slice().into())
    }

    pub fn derived_nonce_account(&self) -> SolanaAccount {
        self.derive_account(
            [self.owner.as_slice(), "nonce-account".as_bytes()]
                .concat()
                .as_slice()
                .into(),
        )
    }
}
