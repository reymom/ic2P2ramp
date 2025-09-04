use crate::{
    model::{
        errors::Result,
        memory::heap,
        types::exchange_rate::{Asset, AssetClass, RateAsset},
    },
    outcalls::{
        jupiter::price::get_usd_price_by_mint, ordiscan::price::fetch_rune_price,
        pricing::xrc_rates::get_xrc_exchange_rate,
    },
};

pub async fn get_exchange_rate(
    mut base_asset: RateAsset,
    mut quote_asset: RateAsset,
) -> Result<f64> {
    base_asset.normalize();
    quote_asset.normalize();

    let base_symbol = base_asset.key_symbol();
    let quote_symbol = quote_asset.key_symbol();

    // Trivial stable pairs
    if let Some(pre) = predefined_stablecoin(&base_symbol, &quote_symbol) {
        return Ok(pre);
    }

    // Cache
    if let Some(cached) = heap::get_cached_rate(&base_symbol, &quote_symbol) {
        return Ok(cached);
    }

    ic_cdk::println!(
        "[get_exchange_rate] miss -> computing {} / {}",
        base_symbol,
        quote_symbol
    );

    // Helper: USD → quote via XRC (works for fiat/crypto quotes known to XRC)
    let usd = || Asset {
        symbol: "USD".into(),
        class: AssetClass::FiatCurrency,
    };
    let usd_to_quote = |q: Asset| async {
        if q.symbol == "USD" {
            Ok(1.0)
        } else {
            get_xrc_exchange_rate(usd(), q).await
        }
    };

    let rate = match (&base_asset, &quote_asset) {
        // Rune → *
        (RateAsset::Rune { name }, _) => {
            let rune_usd = fetch_rune_price(name).await?.data.price_in_usd;
            if quote_symbol == "USD" {
                rune_usd
            } else {
                rune_usd * usd_to_quote(quote_asset.to_xrc_asset()).await?
            }
        }

        // --- * → Rune (quote) ---
        (_, RateAsset::Rune { name }) => {
            let rune_usd = fetch_rune_price(name).await?.data.price_in_usd;
            let base_usd = if base_symbol == "USD" {
                1.0
            } else {
                get_xrc_exchange_rate(base_asset.to_xrc_asset(), usd()).await?
            };
            base_usd / rune_usd
        }

        // --- Everything else: XRC first; SPL-mint (Solana) fallback to Jupiter ---
        _ => {
            match get_xrc_exchange_rate(base_asset.to_xrc_asset(), quote_asset.to_xrc_asset()).await
            {
                Ok(v) => v,
                Err(e) => {
                    ic_cdk::println!("[get_exchange_rate] get_xrc_exchange_rate failed: {}", e);

                    // Fallback in Solana asset with mint present
                    match (&base_asset, &quote_asset) {
                        (
                            RateAsset::Solana {
                                mint: Some(mint), ..
                            },
                            _,
                        ) => {
                            let base_usd = get_usd_price_by_mint(mint).await?;
                            if quote_symbol == "USD" {
                                base_usd
                            } else {
                                base_usd * usd_to_quote(quote_asset.to_xrc_asset()).await?
                            }
                        }
                        (
                            _,
                            RateAsset::Solana {
                                mint: Some(mint), ..
                            },
                        ) => {
                            let quote_usd = get_usd_price_by_mint(mint).await?;
                            let base_usd = if base_symbol == "USD" {
                                1.0
                            } else {
                                get_xrc_exchange_rate(base_asset.to_xrc_asset(), usd()).await?
                            };
                            base_usd / quote_usd
                        }
                        _ => return Err(e.into()),
                    }
                }
            }
        }
    };

    heap::cache_exchange_rate(&base_symbol, &quote_symbol, rate);
    Ok(rate)
}

fn predefined_stablecoin(base: &str, quote: &str) -> Option<f64> {
    match (base, quote) {
        ("USD", "USD") | ("EUR", "EUR") => Some(1.0),
        _ => None,
    }
}
