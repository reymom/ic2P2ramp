use crate::{
    model::{
        errors::Result,
        memory::heap,
        types::exchange_rate::{Asset, AssetClass},
    },
    outcalls::pricing::{ordiscan::fetch_rune_price, xrc_rates::get_xrc_exchange_rate},
};

pub async fn get_cached_exchange_rate(
    mut base_asset: Asset,
    mut quote_asset: Asset,
) -> Result<f64> {
    if let Some(predefined_rate) =
        get_predefined_rate_if_stablecoin(&base_asset.symbol, &quote_asset.symbol)
    {
        Ok(predefined_rate)
    } else {
        match heap::get_cached_rate(base_asset.clone(), quote_asset.clone()) {
            Some(cache) => Ok(cache),
            None => {
                ic_cdk::println!("[get_cached_exchange_rate] Recalculating cache.");

                let rate = match (base_asset.clone().class, quote_asset.clone().class) {
                    (AssetClass::Rune, _) => {
                        let rune_price_in_usd = fetch_rune_price(&base_asset.symbol)
                            .await?
                            .data
                            .price_in_usd;
                        if quote_asset.symbol == "USD" {
                            rune_price_in_usd
                        } else {
                            let conversion_rate = get_xrc_exchange_rate(
                                Asset {
                                    symbol: "USD".to_string(),
                                    class: AssetClass::FiatCurrency,
                                },
                                quote_asset.clone(),
                            )
                            .await?;
                            rune_price_in_usd * conversion_rate
                        }
                    }
                    (_, AssetClass::Rune) => {
                        let rune_price_in_usd = fetch_rune_price(&base_asset.symbol)
                            .await?
                            .data
                            .price_in_usd;
                        if base_asset.symbol == "USD" {
                            1.0 / rune_price_in_usd
                        } else {
                            let conversion_rate = get_xrc_exchange_rate(
                                Asset {
                                    symbol: "USD".to_string(),
                                    class: AssetClass::FiatCurrency,
                                },
                                base_asset.clone(),
                            )
                            .await?;
                            conversion_rate / rune_price_in_usd
                        }
                    }
                    _ => {
                        base_asset.normalize();
                        quote_asset.normalize();
                        get_xrc_exchange_rate(base_asset.clone(), quote_asset.clone()).await?
                    }
                };

                heap::cache_exchange_rate(base_asset, quote_asset, rate);
                Ok(rate)
            }
        }
    }
}

fn get_predefined_rate_if_stablecoin(base_symbol: &str, quote_symbol: &str) -> Option<f64> {
    match (base_symbol, quote_symbol) {
        ("USD", "USD") => Some(1.0),
        ("EUR", "EUR") => Some(1.0),
        _ => None,
    }
}
