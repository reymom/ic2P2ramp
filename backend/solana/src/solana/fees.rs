use core::num::NonZeroU8;
use icramp_types::solana::fees::SolanaFeeEstimates;
use serde_json::json;
use solana_pubkey::Pubkey;
use solana_sdk_ids::system_program;
use std::str::FromStr;

use crate::solana::account::get_account_owner;
use crate::solana::client::client;
use crate::solana::wallet::SolanaWallet;
use crate::{Result, SolanaError};

// Conservative constants (tune as needed)
const BASE_SIG_LAMPORTS: u64 = 5_000; // per signature (fallback)
const CU_SOL_TRANSFER: u64 = 600; // ~450 observed; pad a bit
const CU_SPL_TRANSFER: u64 = 10_000;
const CU_CREATE_ATA: u64 = 40_000;
const ATA_LEN_BYTES: u64 = 165;
const ATA_RENT_FALLBACK: u64 = 2_039_280; // typical rent-exempt for 165 bytes

pub async fn estimate_fees(token_mint: Option<String>) -> Result<SolanaFeeEstimates> {
    // 1. Build the address set for getRecentPrioritizationFees
    let mut addrs: Vec<Pubkey> = vec![
        Pubkey::from_str(&system_program::ID.to_string()).expect("valid system program id"),
        Pubkey::from_str("ComputeBudget111111111111111111111111111111")
            .expect("valid compute budget id"),
    ];

    // If SPL, include the actual token program that owns the mint
    if let Some(mint) = token_mint.as_ref() {
        let mint_pk = Pubkey::from_str(mint).map_err(SolanaError::from)?;
        let token_program = get_account_owner(&mint_pk).await?;
        addrs.push(token_program);
    }

    // Optional: include vault (canister) pubkey to bias fees to our address set
    let vault = SolanaWallet::new_canister().await.solana_account();
    addrs.push(vault.ed25519_public_key);

    // 2. Fetch recent priority fees (subset; median in µ-lamports/CU)
    let fees = client()
        .get_recent_prioritization_fees(addrs.iter().collect::<Vec<_>>())
        .map_err(SolanaError::from)?
        .with_max_length(NonZeroU8::new(8).unwrap())
        .send()
        .await
        .expect_consistent()
        .map_err(SolanaError::from)?;

    let micro_lamports_per_cu = median_u64(fees.iter().map(|f| f.prioritization_fee));
    // If provider returns empty, treat it as 0 priority fee.
    let micro_lamports_per_cu = micro_lamports_per_cu.unwrap_or(0);

    // 3. Decide compute-units budget
    let (lock_cu, withdraw_cu, ata_rent) = if token_mint.is_some() {
        // SPL: payout may need ATA creation; withdraw assumes ATA exists
        let rent = get_ata_rent_exemption().await.unwrap_or(ATA_RENT_FALLBACK);
        (CU_SPL_TRANSFER + CU_CREATE_ATA, CU_SPL_TRANSFER, rent)
    } else {
        (CU_SOL_TRANSFER, CU_SOL_TRANSFER, 0)
    };

    // 4. Convert µ-lamports/CU -> lamports for each path (ceil)
    let pri_lock_lamports =
        ceil_div_u128(micro_lamports_per_cu as u128 * lock_cu as u128, 1_000_000) as u64;
    let pri_withdraw_lamports = ceil_div_u128(
        micro_lamports_per_cu as u128 * withdraw_cu as u128,
        1_000_000,
    ) as u64;

    // 5. Base signatures (usually 1; keep simple & conservative)
    let base_lock = BASE_SIG_LAMPORTS;
    let base_withdraw = BASE_SIG_LAMPORTS;

    Ok(SolanaFeeEstimates {
        lock_lamports: base_lock + pri_lock_lamports + ata_rent, // SPL may add rent
        withdraw_lamports: base_withdraw + pri_withdraw_lamports, // ATA presumed existing
    })
}

fn median_u64<I: Iterator<Item = u64>>(mut it: I) -> Option<u64> {
    let mut v: Vec<u64> = Vec::new();
    // avoid allocations on IC; bound to something small
    while let Some(x) = it.next() {
        let _ = v.push(x);
    }
    if v.is_empty() {
        return None;
    }
    v.sort_unstable();
    let mid = v.len() / 2;
    Some(if v.len() % 2 == 0 {
        (v[mid - 1] + v[mid]) / 2
    } else {
        v[mid]
    })
}

// Purpose: ceil(a / b) for u128
fn ceil_div_u128(a: u128, b: u128) -> u128 {
    if a == 0 { 0 } else { (a + b - 1) / b }
}

// Purpose: try to get rent-exempt lamports for a 165-byte ATA; fallback if unavailable.
async fn get_ata_rent_exemption() -> Result<u64> {
    let s = client()
        .json_request(json!({
            "jsonrpc":"2.0","id":1,
            "method":"getMinimumBalanceForRentExemption",
            "params":[ATA_LEN_BYTES]
        }))
        .send()
        .await
        .expect_consistent()
        .map_err(SolanaError::from)?;
    let v: serde_json::Value = serde_json::from_str(&s)
        .map_err(|e| SolanaError::SystemError(crate::SystemError::ParseError(e.to_string())))?;
    Ok(v.get("result")
        .and_then(|n| n.as_u64())
        .unwrap_or(ATA_RENT_FALLBACK))
}
