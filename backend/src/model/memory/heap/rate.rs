use crate::{model::types::exchange_rate::ExchangeRateCache, outcalls::xrc_rates::Asset};

use super::storage::EXCHANGE_RATE_CACHE;

pub fn cache_exchange_rate(base_asset: Asset, quote_asset: Asset, rate: f64) {
    EXCHANGE_RATE_CACHE.with_borrow_mut(|rates| {
        rates.insert(
            (base_asset.symbol, quote_asset.symbol),
            ExchangeRateCache::new(rate),
        )
    });
}

pub fn get_cached_rate(base_asset: Asset, quote_asset: Asset) -> Option<f64> {
    EXCHANGE_RATE_CACHE.with_borrow(|rates| {
        rates
            .get(&(base_asset.symbol, quote_asset.symbol))
            .and_then(|rate| rate.get_cached_rate())
    })
}
