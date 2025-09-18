use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;
use icramp_types::solana::errors::{Result, SolanaError, SystemError};
use sol_rpc_types::{AccountData, AccountEncoding, DataSlice, GetAccountInfoEncoding};
use solana_pubkey::Pubkey;
use std::str::FromStr;

use crate::solana::{account::get_account_owner, client::client};

const TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
const TOKEN_2022_PROGRAM_ID: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";

pub fn validate_token_mint(mint: String) -> Result<()> {
    Pubkey::from_str(&mint).map_err(SolanaError::from)?;
    Ok(())
}

// SPL Mint layout keeps `decimals` at byte 44 for both Token + Token-2022.
pub async fn fetch_mint_decimals(mint: &Pubkey) -> Result<u8> {
    let owner = get_account_owner(mint).await?;
    let owner_str = owner.to_string();
    if owner_str != TOKEN_PROGRAM_ID && owner_str != TOKEN_2022_PROGRAM_ID {
        return Err(SolanaError::UnsupportedToken(mint.to_string()));
    }

    let ui = client()
        .get_account_info(*mint)
        .with_encoding(GetAccountInfoEncoding::Base64)
        .with_data_slice(DataSlice {
            length: 1,
            offset: 44,
        })
        .send()
        .await
        .expect_consistent()
        .map_err(SolanaError::from)?
        .ok_or_else(|| SolanaError::RpcError("Mint account not found".to_string()))?;

    let b64 = match ui.data.into() {
        AccountData::Binary(s, AccountEncoding::Base64) => s,
        AccountData::LegacyBinary(s) => s, // unlikely if we set Base64, but tolerate
        AccountData::Binary(_, other) => {
            return Err(SolanaError::RpcError(format!(
                "Unexpected encoding {:?}",
                other
            )));
        }
        AccountData::Json(_) => {
            return Err(SolanaError::RpcError("Unexpected JSON account data".into()));
        }
    };

    let bytes = B64
        .decode(b64.as_bytes())
        .map_err(|e| SolanaError::SystemError(SystemError::ParseError(e.to_string())))?;

    if bytes.len() != 1 {
        return Err(SolanaError::RpcError(format!(
            "Expected 1 byte, got {}",
            bytes.len()
        )));
    }
    Ok(bytes[0])
}
