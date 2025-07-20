use crate::memory::stable::vault::OFFRAMPER_VAULTS;
use crate::model::types::{
    Address,
    errors::{Result, VaultError},
    vault::VaultEntry,
};

/// Deposit lamports or a specific SPL token amount into an offramper’s vault.
pub fn deposit_to_vault(
    offramper_address: Address,
    amount: u64,
    token_mint: Option<String>,
) -> Result<()> {
    OFFRAMPER_VAULTS.with(|vaults_cell| {
        let mut vaults = vaults_cell.borrow_mut();
        let mut entry = vaults
            .get(&offramper_address)
            .unwrap_or_else(VaultEntry::new);

        if let Some(mint) = token_mint {
            // Increment token balance
            let mut tokens = entry.tokens.clone();
            let bal = tokens.entry(mint.clone()).or_insert(0);
            *bal += amount;
            entry.tokens = tokens;
        } else {
            // Increment lamports
            entry.lamports = entry.lamports.saturating_add(amount);
        }

        vaults.insert(offramper_address, entry);
        Ok(())
    })
}

/// Cancel (roll back) a deposit—subtract lamports or token amount
pub fn cancel_deposit(
    offramper_address: Address,
    amount: u64,
    token_mint: Option<String>,
) -> Result<()> {
    OFFRAMPER_VAULTS.with(|vaults_cell| {
        let mut vaults = vaults_cell.borrow_mut();
        let mut entry: VaultEntry = vaults
            .get(&offramper_address)
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

        vaults.insert(offramper_address, entry);
        Ok(())
    })?;
    Ok(())
}

/// Read one vault entry for a given address
pub fn get_vault_entry(offramper_address: &Address) -> Option<VaultEntry> {
    OFFRAMPER_VAULTS.with(|vaults_cell| vaults_cell.borrow().get(offramper_address))
}
