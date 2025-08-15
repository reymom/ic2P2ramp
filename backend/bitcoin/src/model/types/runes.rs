use std::borrow::Cow;
use std::str::FromStr;

use candid::{CandidType, Deserialize};
use ic_stable_structures::{storable::Bound, Storable};

use crate::model::types::errors::{BitcoinError, Result};

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct RuneMetadata {
    pub id: RuneID,       // Unique ID (BLOCK:TX)
    pub name: String,     // Rune name (e.g., DOG•TO•THE•MOON)
    pub symbol: String,   // Currency symbol (e.g., 🐕)
    pub divisibility: u8, // Number of decimals
    pub cap: u128,        // Maximum supply
    pub premine: u128,    // Pre-minted amount
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Etching {
    pub metadata: RuneMetadata,
    pub spacers: Option<u32>,
    pub amount: Option<u128>,
    pub height: Option<(u64, u64)>,
    pub offset: Option<(u64, u64)>,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct RuneID(String);

impl RuneID {
    /// Validate the RuneID during construction.
    pub fn new(block: u64, tx_index: u32) -> Result<Self> {
        if block == 0 || tx_index == 0 {
            return Err(BitcoinError::InvalidInput(
                "Block and transaction index must be greater than 0.".to_string(),
            ));
        }

        Ok(Self(format!("{}:{}", block, tx_index)))
    }

    /// Parse a RuneID from a string and validate it.
    pub fn validate(&self) -> Result<Self> {
        let parts: Vec<&str> = self.0.split(':').collect();

        if parts.len() != 2 {
            return Err(BitcoinError::InvalidRuneID(
                "Expected 'BLOCK:TX'.".to_string(),
            ));
        }

        let block = parts[0]
            .parse::<u64>()
            .map_err(|_| BitcoinError::InvalidRuneID("Invalid block number.".to_string()))?;
        parts[1]
            .parse::<u32>()
            .map_err(|_| BitcoinError::InvalidRuneID("Invalid transaction index.".to_string()))?;

        if block == 0 {
            return Err(BitcoinError::InvalidRuneID(
                "Block must be greater than 0".to_string(),
            ));
        }

        Ok(Self(self.to_string()))
    }

    /// Extract block and transaction index from the RuneID.
    pub fn parts(&self) -> (u64, u32) {
        let parts: Vec<&str> = self.0.split(':').collect();
        let block = parts[0].parse::<u64>().unwrap();
        let tx_index = parts[1].parse::<u32>().unwrap();
        (block, tx_index)
    }
}

impl Storable for RuneID {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(self.0.as_bytes().to_vec())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Self(String::from_utf8(bytes.to_vec()).unwrap())
    }

    const BOUND: Bound = Bound::Unbounded;
}

impl FromStr for RuneID {
    type Err = BitcoinError;

    fn from_str(s: &str) -> Result<Self> {
        let rune_id = Self(s.to_string());
        rune_id.validate()?;
        Ok(rune_id)
    }
}

impl std::fmt::Display for RuneID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for RuneMetadata {
    /// Serialize `RuneMetadata` to a string.
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}:{}:{}:{}:{}:{}",
            self.id, self.name, self.symbol, self.divisibility, self.cap, self.premine
        )
    }
}

impl RuneMetadata {
    pub fn new(
        block: u64,
        tx_index: u32,
        name: String,
        symbol: String,
        divisibility: u8,
        cap: u128,
        premine: u128,
    ) -> Result<Self> {
        let id = RuneID::new(block, tx_index)?;
        Ok(Self {
            id,
            name,
            symbol,
            divisibility,
            cap,
            premine,
        })
    }
}
