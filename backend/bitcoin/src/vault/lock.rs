use crate::{
    memory::stable::vault::ONRAMPER_VAULTS,
    model::types::{Address, runes::RuneID, vault::VaultEntry},
};
use icramp_types::bitcoin::errors::Result;

pub fn lock_funds(
    offramper_address: Address,
    onramper: Address,
    amount: u64,
    rune: Option<RuneID>,
) -> Result<()> {
    super::deposit::cancel_deposit(offramper_address, amount, rune.clone())?;

    ONRAMPER_VAULTS.with_borrow_mut(|vaults| {
        let mut entry = vaults.get(&onramper).unwrap_or_else(VaultEntry::new);

        if let Some(rune_id) = rune {
            let rune_balance = entry.runes.entry(rune_id).or_insert(0);
            *rune_balance += amount;
        } else {
            entry.bitcoin_balance += amount;
        }

        vaults.insert(onramper, entry);
    });

    Ok(())
}

pub fn unlock_funds(
    offramper: Address,
    onramper: Address,
    amount: u64,
    rune: Option<RuneID>,
) -> Result<()> {
    super::complete::complete_order(onramper, amount, rune.clone())?;

    super::deposit::deposit_to_vault(offramper, amount, rune)?;

    Ok(())
}
