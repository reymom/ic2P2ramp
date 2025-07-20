use sol_rpc_types::GetAccountInfoEncoding;
use solana_pubkey::Pubkey;
use std::str::FromStr;

use crate::solana::client::client;

pub async fn get_account_owner(account: &Pubkey) -> Pubkey {
    let owner = client()
        .get_account_info(*account)
        .with_encoding(GetAccountInfoEncoding::Base64)
        .send()
        .await
        .expect_consistent()
        .expect("Call to `getAccountInfo` failed")
        .unwrap_or_else(|| panic!("Account not found for pubkey `{account}`"))
        .owner;
    Pubkey::from_str(&owner).unwrap()
}
