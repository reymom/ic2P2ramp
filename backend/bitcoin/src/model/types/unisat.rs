use candid::Deserialize;

use icramp_types::bitcoin::runes::RuneUTXOEntry;

#[derive(Deserialize, Debug)]
pub struct UnisatRune {
    pub runeid: String,
    pub amount: String,
}

#[derive(Deserialize, Debug)]
pub struct UnisatRuneData {
    // pub confirmations: u32,
    #[serde(rename = "scriptPk")]
    pub script_pk: String,
    pub txid: String,
    pub vout: u32,
    pub runes: Vec<UnisatRune>,
}

#[derive(Deserialize, Debug)]
pub struct UnisatRuneUTXOsData {
    pub utxo: Vec<UnisatRuneData>,
}

#[derive(Deserialize, Debug)]
pub struct UnisatRuneUTXOsResponse {
    pub data: UnisatRuneUTXOsData,
}

impl UnisatRuneData {
    pub fn into_rune_utxo_entry(&self, rune_id: &str) -> Option<RuneUTXOEntry> {
        // Find the specific rune we're looking for in the runes array
        let rune = self.runes.iter().find(|r| r.runeid == rune_id)?;

        // Parse the amount string into a u64
        let amount = rune.amount.parse::<u64>().ok()?;

        Some(RuneUTXOEntry {
            txid: self.txid.clone(),
            vout: self.vout,
            rune_amount: amount,
            script_pubkey: self.script_pk.clone(),
        })
    }
}

impl UnisatRuneUTXOsResponse {
    pub fn into_rune_utxo_entries(&self, rune_id: &str) -> Vec<RuneUTXOEntry> {
        self.data
            .utxo
            .iter()
            .filter_map(|utxo| utxo.into_rune_utxo_entry(rune_id))
            .collect()
    }
}
