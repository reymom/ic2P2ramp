use crate::model::types::exchange_rate::ExchangeRateCache;

use super::storage::EXCHANGE_RATE_CACHE;

pub fn cache_exchange_rate(base_symbol: &str, quote_symbol: &str, rate: f64) {
    EXCHANGE_RATE_CACHE.with_borrow_mut(|rates| {
        rates.insert(
            (base_symbol.to_string(), quote_symbol.to_string()),
            ExchangeRateCache::new(rate),
        )
    });
}

pub fn get_cached_rate(base_symbol: &str, quote_symbol: &str) -> Option<f64> {
    EXCHANGE_RATE_CACHE.with_borrow(|rates| {
        rates
            .get(&(base_symbol.to_string(), quote_symbol.to_string()))
            .and_then(|rate| rate.get_cached_rate())
    })
}
