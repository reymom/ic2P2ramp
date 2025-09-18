use candid::Principal;
use lazy_static::lazy_static;
use pocket_ic::PocketIc;
use std::{collections::HashMap, sync::Mutex};

use icramp_types::solana::{errors::Result, token::TokenInfo, vault::VaultEntry};

use crate::setup::setup_solana_backend;
use testkit::helpers::{query_call, update_call};

lazy_static! {
    pub static ref SOLANA_ENV: Mutex<Option<SolanaTestEnv>> = Mutex::new(None);
}

pub struct SolanaTestEnv {
    pub pic: PocketIc,
    pub canister_id: Principal,
    pub sol_account: Option<String>,
}

pub fn get_solana_env() -> std::sync::MutexGuard<'static, Option<SolanaTestEnv>> {
    let mut env = SOLANA_ENV.lock().unwrap_or_else(|p| p.into_inner());
    if env.is_none() {
        *env = Some(SolanaTestEnv::new());
    }
    env
}

impl SolanaTestEnv {
    pub fn new() -> Self {
        let (pic, can_id) = setup_solana_backend();
        Self {
            pic,
            canister_id: can_id,
            sol_account: None,
        }
    }

    #[allow(dead_code)]
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

    pub fn get_canister_account(&mut self) -> String {
        if let Some(ref acc) = self.sol_account {
            return acc.clone();
        }

        let account: String = update_call(
            &mut self.pic,
            self.canister_id,
            "canister_solana_account",
            (),
        )
        .expect("Failed to get canister solana account");

        self.sol_account = Some(account.clone());
        account
    }

    // Thin wrappers

    // --- TOKEN operations ---
    pub fn get_tokens(&mut self) -> HashMap<String, TokenInfo> {
        query_call(&mut self.pic, self.canister_id, "get_registered_tokens", ())
            .expect("get_tokens query failed")
    }

    pub fn get_token(&mut self, mint: String) -> Result<TokenInfo> {
        query_call(&mut self.pic, self.canister_id, "get_token_info", (mint,))
            .expect("get_token_info query failed")
    }

    pub fn supported_token(&mut self, mint: String) -> Result<()> {
        query_call(
            &mut self.pic,
            self.canister_id,
            "is_token_supported",
            (mint,),
        )
        .expect("is_token_supported query failed")
    }

    pub fn register_tokens(&mut self, tokens: Vec<(String, String, String)>) -> Result<()> {
        update_call(
            &mut self.pic,
            self.canister_id,
            "register_tokens",
            (tokens,),
        )
        .expect("register_tokens query failed")
    }

    // --- VAULT operations ---
    pub fn deposit(
        &mut self,
        offramper: String,
        amount: u64,
        spl_mint: Option<String>,
    ) -> Result<()> {
        update_call(
            &mut self.pic,
            self.canister_id,
            "deposit_to_vault_canister",
            (offramper, amount, spl_mint),
        )
        .expect("deposit_to_vault_canister query failed")
    }

    pub fn cancel(
        &mut self,
        offramper: String,
        amount: u64,
        spl_mint: Option<String>,
    ) -> Result<()> {
        update_call(
            &mut self.pic,
            self.canister_id,
            "cancel_deposit",
            (offramper, amount, spl_mint),
        )
        .expect("cancel_deposit query failed")
    }

    pub fn lock(
        &mut self,
        offramper: String,
        onramper: String,
        amount: u64,
        spl_mint: Option<String>,
    ) -> Result<()> {
        update_call(
            &mut self.pic,
            self.canister_id,
            "lock_funds",
            (offramper, onramper, amount, spl_mint),
        )
        .expect("lock_funds query failed")
    }

    pub fn unlock(
        &mut self,
        offramper: String,
        onramper: String,
        amount: u64,
        spl_mint: Option<String>,
    ) -> Result<()> {
        update_call(
            &mut self.pic,
            self.canister_id,
            "unlock_funds",
            (offramper, onramper, amount, spl_mint),
        )
        .expect("unlock_funds query failed")
    }

    pub fn complete(
        &mut self,
        onramper: String,
        amount: u64,
        spl_mint: Option<String>,
    ) -> Result<()> {
        update_call(
            &mut self.pic,
            self.canister_id,
            "complete_order",
            (onramper, amount, spl_mint),
        )
        .expect("complete_order query failed")
    }

    pub fn get_offramper_vault(&self, owner: String) -> Result<VaultEntry> {
        query_call(
            &self.pic,
            self.canister_id,
            "get_offramper_deposits",
            (owner,),
        )
        .expect("get_offramper_deposits failed")
    }

    pub fn get_onramper_vault(&self, owner: String) -> Result<VaultEntry> {
        query_call(
            &self.pic,
            self.canister_id,
            "get_onramper_deposits",
            (owner,),
        )
        .expect("get_onramper_deposits failed")
    }
}
