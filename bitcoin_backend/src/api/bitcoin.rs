use ic_cdk::api::management_canister::bitcoin::{
    bitcoin_get_balance, bitcoin_get_current_fee_percentiles, bitcoin_get_utxos,
    bitcoin_send_transaction, BitcoinAddress, BitcoinNetwork, GetBalanceRequest,
    GetCurrentFeePercentilesRequest, GetUtxosRequest, MillisatoshiPerByte, SendTransactionRequest,
    Utxo, UtxoFilter,
};

use crate::model::types::errors::{BitcoinError, Result};

/// Returns the balance of the given bitcoin address.
///
/// Relies on the `bitcoin_get_balance` endpoint.
/// See https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-bitcoin_get_balance
pub async fn get_balance(network: BitcoinNetwork, address: String) -> Result<u64> {
    let min_confirmations = None;
    match bitcoin_get_balance(GetBalanceRequest {
        address,
        network,
        min_confirmations,
    })
    .await
    {
        Ok(balance_res) => Ok(balance_res.0),
        Err((code, err)) => Err(BitcoinError::CallRejectionError(code, err)),
    }
}

/// Returns the UTXOs of the given bitcoin address.
///
/// NOTE: Relies on the `bitcoin_get_utxos` endpoint.
/// See https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-bitcoin_get_utxos
pub async fn get_utxos(network: BitcoinNetwork, address: BitcoinAddress) -> Result<Vec<Utxo>> {
    const MAX_PAGES: usize = 20;
    let mut all_utxos = Vec::new();
    let mut next_page = None;
    let mut page_count = 0;
    let mut consecutive_empty_pages = 0;

    loop {
        page_count += 1;
        if page_count > MAX_PAGES {
            return Err(BitcoinError::InternalError(format!(
                "Exceeded maximum number of pages ({}) while fetching UTXOs.",
                MAX_PAGES
            )));
        }

        // Create the request, using `next_page` if it’s available.
        let request = GetUtxosRequest {
            address: address.clone(),
            network,
            filter: next_page.map(UtxoFilter::Page),
        };

        match bitcoin_get_utxos(request).await {
            Ok((utxos_response,)) => {
                if utxos_response.utxos.is_empty() {
                    consecutive_empty_pages += 1;
                } else {
                    all_utxos.extend(utxos_response.utxos);
                    consecutive_empty_pages = 0;
                }

                if consecutive_empty_pages >= 3 {
                    return Err(BitcoinError::InternalError(
                        "Exceeded maximum consecutive empty UTXO pages.".to_string(),
                    ));
                }

                next_page = utxos_response.next_page;
                if next_page.is_none() {
                    break;
                }
            }
            Err((code, msg)) => {
                return Err(BitcoinError::CallRejectionError(code, msg));
            }
        }
    }

    Ok(all_utxos)
}

/// Returns the 100 fee percentiles measured in millisatoshi/byte.
/// Percentiles are computed from the last 10,000 transactions (if available).
///
/// Relies on the `bitcoin_get_current_fee_percentiles` endpoint.
/// See https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-bitcoin_get_current_fee_percentiles
pub async fn get_current_fee_percentiles(
    network: BitcoinNetwork,
) -> Result<Vec<MillisatoshiPerByte>> {
    match bitcoin_get_current_fee_percentiles(GetCurrentFeePercentilesRequest { network }).await {
        Ok((percentiles,)) => Ok(percentiles),
        Err((code, msg)) => Err(BitcoinError::CallRejectionError(code, msg)),
    }
}

/// Sends a (signed) transaction to the bitcoin network.
///
/// Relies on the `bitcoin_send_transaction` endpoint.
/// See https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-bitcoin_send_transaction
pub async fn send_transaction(network: BitcoinNetwork, transaction: Vec<u8>) -> Result<()> {
    bitcoin_send_transaction(SendTransactionRequest {
        network,
        transaction,
    })
    .await
    .map_err(|(code, msg)| BitcoinError::CallRejectionError(code, msg))
}
