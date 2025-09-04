use candid::{CandidType, Deserialize};
use serde::Serialize;

pub const CACHE_DURATION: u64 = 600 * 1_000_000_000; // 10 minutes

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub enum AssetClass {
    Cryptocurrency,
    FiatCurrency,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Asset {
    pub symbol: String,
    pub class: AssetClass,
}

#[derive(Debug, Clone, CandidType, Deserialize)]
pub struct ExchangeRateCache {
    pub rate: f64,
    pub timestamp: u64,
}

impl ExchangeRateCache {
    pub fn new(rate: f64) -> Self {
        ExchangeRateCache {
            rate,
            timestamp: ic_cdk::api::time(),
        }
    }

    pub fn get_cached_rate(&self) -> Option<f64> {
        let current_time = ic_cdk::api::time();

        let time_elapsed = current_time - self.timestamp;
        ic_cdk::println!(
            "[get_cached_rate] current_time ({}) - timestamp ({}) = {}",
            current_time / 1_000_000_000,
            self.timestamp / 1_000_000_000,
            time_elapsed / 1_000_000_000
        );
        if time_elapsed < CACHE_DURATION {
            return Some(self.rate);
        }

        None
    }
}

/// How to route price discovery for a given asset
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum RateAsset {
    /// Fiat currency priced via XRC (e.g., "USD", "EUR")
    Fiat { symbol: String },

    /// Generic crypto (BTC/ETH/USDC/… by **symbol**) priced via XRC
    Crypto { symbol: String },

    /// Bitcoin Rune by **name** (e.g., "DOG•GO•TO•THE•MOON")
    /// Priced via Ordiscan (USD), then converted to quote via XRC.
    Rune { name: String },

    /// Solana token by **symbol**, with optional **mint** for Jupiter fallback.
    /// Route: XRC by symbol first; if missing and mint is provided, fallback via Jupiter (mint→USD) then USD→quote via XRC.
    Solana {
        symbol: String,
        mint: Option<String>,
    },
}

impl RateAsset {
    /// Normalize in-place so cache/XRC keys are stable and consistent.
    /// - ckBTC/ckETH → BTC/ETH
    /// - Fiat-looking cryptos flipped to Fiat when symbol is USD/EUR
    /// - Rune names → [A–Z0–9] uppercase (strip separators like '•')
    pub fn normalize(&mut self) {
        *self = match core::mem::replace(
            self,
            RateAsset::Fiat {
                symbol: String::new(),
            },
        ) {
            RateAsset::Fiat { symbol } => RateAsset::Fiat {
                symbol: symbol.to_uppercase(),
            },

            RateAsset::Crypto { symbol } => {
                let mut s = symbol.to_uppercase();
                if s == "CKBTC" {
                    s = "BTC".into();
                } else if s == "CKETH" {
                    s = "ETH".into();
                }

                if s == "USD" || s == "EUR" {
                    RateAsset::Fiat { symbol: s }
                } else {
                    RateAsset::Crypto { symbol: s }
                }
            }

            RateAsset::Rune { name } => {
                let n = name
                    .chars()
                    .filter(|c| c.is_ascii_alphanumeric())
                    .collect::<String>()
                    .to_uppercase();
                RateAsset::Rune { name: n }
            }

            RateAsset::Solana { symbol, mint } => {
                let s = symbol.to_uppercase();
                let m = mint.map(|x| x.trim().to_string());

                if s == "USD" || s == "EUR" {
                    RateAsset::Fiat { symbol: s }
                } else if s == "CKBTC" {
                    RateAsset::Solana {
                        symbol: "BTC".into(),
                        mint: m,
                    }
                } else if s == "CKETH" {
                    RateAsset::Solana {
                        symbol: "ETH".into(),
                        mint: m,
                    }
                } else {
                    RateAsset::Solana { symbol: s, mint: m }
                }
            }
        };
    }

    /// get a display/cache key symbol after normalization.
    pub fn key_symbol(&self) -> String {
        match self {
            RateAsset::Fiat { symbol } => symbol.clone(),
            RateAsset::Crypto { symbol } => symbol.clone(),
            RateAsset::Rune { name } => name.clone(),
            RateAsset::Solana { symbol, .. } => symbol.clone(),
        }
    }

    /// Convert to internal `Asset` (for XRC paths only).
    /// Returns `None` for Rune (use Ordiscan).
    pub fn to_xrc_asset(&self) -> Asset {
        match self {
            RateAsset::Fiat { symbol } => Asset {
                symbol: symbol.clone(),
                class: AssetClass::FiatCurrency,
            },
            RateAsset::Crypto { symbol } => Asset {
                symbol: symbol.clone(),
                class: AssetClass::Cryptocurrency,
            },
            RateAsset::Solana { symbol, .. } => Asset {
                symbol: symbol.clone(),
                class: AssetClass::Cryptocurrency,
            },
            RateAsset::Rune { name } => Asset {
                symbol: name.to_string(),
                class: AssetClass::Cryptocurrency,
            },
        }
    }
}
