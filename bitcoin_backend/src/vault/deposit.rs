use crate::{
    memory::stable::vault::OFFRAMPER_VAULTS,
    model::types::{
        errors::{Result, VaultError}, runes::RuneID, vault::VaultEntry, Address
    },
};

pub fn deposit_to_vault(offramper_address: Address, amount: u64, rune: Option<RuneID>) -> Result<()> {
    OFFRAMPER_VAULTS.with_borrow_mut(|vaults| {
        let mut entry = vaults.get(&offramper_address).unwrap_or_else(VaultEntry::new);

        if let Some(rune_id) = rune {
            let rune_balance = entry.runes.entry(rune_id).or_insert(0);
            *rune_balance += amount;
        } else {
            entry.bitcoin_balance += amount;
        }

        vaults.insert(offramper_address, entry);
        Ok(())
    })
}

pub fn cancel_deposit(offramper_address: Address, amount: u64, rune: Option<RuneID>) -> Result<()> {
    OFFRAMPER_VAULTS.with_borrow_mut(|vaults| {
        let mut entry = vaults
            .get(&offramper_address)
            .ok_or(VaultError::AddressVaultNotFound)?;

        if let Some(rune_id) = rune {
            let rune_balance = entry.runes.get_mut(&rune_id).ok_or_else(|| {
                VaultError::InsufficientBalance
            })?;
            if *rune_balance < amount {
                return Err(VaultError::InsufficientBalance.into());
            }
            *rune_balance -= amount;

            if *rune_balance == 0 {
                entry.runes.remove(&rune_id);
            }
        } else {
            if entry.bitcoin_balance < amount {
                return Err(VaultError::InsufficientBalance.into());
            }
            entry.bitcoin_balance -= amount;
        }

        vaults.insert(offramper_address, entry);
        Ok(())
    })
}
