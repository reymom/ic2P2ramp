use crate::{
    memory::stable::vault::ONRAMPER_VAULTS,
    model::types::{Address, runes::RuneID},
};
use icramp_types::bitcoin::errors::{Result, VaultError};

pub fn complete_order(onramper: Address, amount: u64, rune: Option<RuneID>) -> Result<()> {
    ONRAMPER_VAULTS.with_borrow_mut(|vaults| {
        let mut entry = vaults
            .get(&onramper)
            .ok_or(VaultError::AddressVaultNotFound)?;

        if let Some(rune_id) = rune {
            let rune_balance = entry
                .runes
                .get_mut(&rune_id)
                .ok_or(VaultError::InsufficientBalance)?;
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

        vaults.insert(onramper, entry);

        Ok(())
    })
}
