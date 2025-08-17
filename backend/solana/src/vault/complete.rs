use crate::{memory::stable::vault::ONRAMPER_VAULTS, model::types::Address};
use icramp_types::solana::errors::{Result, VaultError};

pub fn complete_order(onramper: Address, amount: u64, token_mint: Option<String>) -> Result<()> {
    ONRAMPER_VAULTS.with_borrow_mut(|vaults| {
        let mut entry = vaults
            .get(&onramper)
            .ok_or(VaultError::AddressVaultNotFound)?;

        if let Some(mint) = token_mint {
            let tokens = &mut entry.tokens;
            let current = tokens
                .get_mut(&mint)
                .ok_or(VaultError::InsufficientBalance)?;
            if *current < amount {
                return Err(VaultError::InsufficientBalance);
            }
            *current -= amount;
            if *current == 0 {
                tokens.remove(&mint);
            }
        } else {
            if entry.lamports < amount {
                return Err(VaultError::InsufficientBalance);
            }
            entry.lamports -= amount;
        }

        vaults.insert(onramper, entry);

        Ok(())
    })?;
    Ok(())
}
