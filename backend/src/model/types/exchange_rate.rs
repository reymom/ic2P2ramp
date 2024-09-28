use candid::{CandidType, Deserialize};

pub const CACHE_DURATION: u64 = 600 * 1_000_000_000; // 10 minutes

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
