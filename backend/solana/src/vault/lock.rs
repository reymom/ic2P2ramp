use ramp_types::solana::errors::Result;

use crate::{
    memory::stable::vault::ONRAMPER_VAULTS,
    model::types::{Address, vault::VaultEntry},
};

pub fn lock_funds(
    offramper_address: Address,
    onramper: Address,
    amount: u64,
    token_mint: Option<String>,
) -> Result<()> {
    super::deposit::cancel_deposit(offramper_address, amount, token_mint.clone())?;

    ONRAMPER_VAULTS.with_borrow_mut(|vaults| {
        let mut entry = vaults.get(&onramper).unwrap_or_else(VaultEntry::new);

        if let Some(mint) = token_mint {
            let mut tokens = entry.tokens.clone();
            let bal = tokens.entry(mint.clone()).or_insert(0);

            *bal += amount;
            entry.tokens = tokens;
        } else {
            entry.lamports = entry.lamports.saturating_add(amount);
        }

        vaults.insert(onramper, entry);
    });

    Ok(())
}

pub fn unlock_funds(
    offramper: Address,
    onramper: Address,
    amount: u64,
    token_mint: Option<String>,
) -> Result<()> {
    super::complete::complete_order(onramper, amount, token_mint.clone())?;

    super::deposit::deposit_to_vault(offramper, amount, token_mint)?;

    Ok(())
}
