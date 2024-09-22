pub const CACHE_DURATION: u64 = 600; // 10 minutes

#[derive(Clone)]
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

        if current_time - self.timestamp < CACHE_DURATION {
            return Some(self.rate);
        }

        None
    }
}
