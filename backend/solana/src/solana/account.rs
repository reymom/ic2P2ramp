use ramp_types::solana::errors::{Result, SolanaError};
use sol_rpc_types::GetAccountInfoEncoding;
use solana_pubkey::Pubkey;
use std::str::FromStr;

use crate::solana::client::client;

pub async fn get_account_owner(account: &Pubkey) -> Result<Pubkey> {
    let owner = client()
        .get_account_info(*account)
        .with_encoding(GetAccountInfoEncoding::Base64)
        .with_cycles(3_000_000_000)
        .send()
        .await
        .expect_consistent()
        .map_err(SolanaError::from)?
        .ok_or_else(|| SolanaError::AccountNotFound)?
        .owner;
    Pubkey::from_str(&owner).map_err(SolanaError::from)
}
