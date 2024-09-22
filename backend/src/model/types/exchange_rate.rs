const CACHE_DURATION: u64 = 600; // 10 minutes

pub struct ExchangeRateCache {
    pub rate: f64,
    pub timestamp: u64,
}
