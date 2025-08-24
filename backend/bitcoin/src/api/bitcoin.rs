use candid::Principal;
use ic_btc_interface::{
    Address, GetBalanceRequest, GetBlockHeadersRequest, GetBlockHeadersResponse,
    GetCurrentFeePercentilesRequest, GetUtxosRequest, GetUtxosResponse, MillisatoshiPerByte,
    Network, Page, Satoshi, SendTransactionRequest, Utxo, UtxosFilterInRequest,
};
use ic_cdk::api::call::call_with_payment128;
use icramp_types::bitcoin::errors::{BitcoinError, Result};

const GET_UTXO_MAINNET: u128 = 10_000_000_000;
const GET_UTXO_TESTNET: u128 = 4_000_000_000;

const GET_CURRENT_FEE_PERCENTILES_MAINNET: u128 = 100_000_000;
const GET_CURRENT_FEE_PERCENTILES_TESTNET: u128 = 40_000_000;

const GET_BALANCE_MAINNET: u128 = 100_000_000;
const GET_BALANCE_TESTNET: u128 = 40_000_000;

const SEND_TRANSACTION_SUBMISSION_MAINNET: u128 = 5_000_000_000;
const SEND_TRANSACTION_SUBMISSION_TESTNET: u128 = 2_000_000_000;

const SEND_TRANSACTION_PAYLOAD_MAINNET: u128 = 20_000_000;
const SEND_TRANSACTION_PAYLOAD_TESTNET: u128 = 8_000_000;

const GET_BLOCK_HEADERS_MAINNET: u128 = 10_000_000_000;
const GET_BLOCK_HEADERS_TESTNET: u128 = 4_000_000_000;

/// Returns the balance of the given bitcoin address.
pub async fn get_balance(
    network: Network,
    btc_principal: Principal,
    address: String,
) -> Result<u64> {
    let cycles = match network {
        Network::Mainnet => GET_BALANCE_MAINNET,
        Network::Testnet => GET_BALANCE_TESTNET,
        Network::Regtest => 0,
    };

    let min_confirmations = None;
    match call_with_payment128::<(GetBalanceRequest,), (Satoshi,)>(
        btc_principal,
        "bitcoin_get_balance",
        (GetBalanceRequest {
            address,
            network: network.into(),
            min_confirmations,
        },),
        cycles,
    )
    .await
    {
        Ok((balance_res,)) => Ok(balance_res),
        Err((code, err)) => Err(BitcoinError::CallRejectionError(code, err)),
    }
}

/// Returns the UTXOs of the given bitcoin address.
pub async fn get_utxos(
    network: Network,
    btc_principal: Principal,
    address: Address,
) -> Result<Vec<Utxo>> {
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
            network: network.into(),
            filter: next_page.map(|page: Page| UtxosFilterInRequest::Page(page)),
        };
        let cycles = match network {
            Network::Mainnet => GET_UTXO_MAINNET,
            Network::Testnet => GET_UTXO_TESTNET,
            Network::Regtest => 0,
        };

        match call_with_payment128::<(GetUtxosRequest,), (GetUtxosResponse,)>(
            btc_principal,
            "bitcoin_get_utxos",
            (request,),
            cycles,
        )
        .await
        {
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
pub async fn get_current_fee_percentiles(
    network: Network,
    btc_principal: Principal,
) -> Result<Vec<MillisatoshiPerByte>> {
    let cycles = match network {
        Network::Mainnet => GET_CURRENT_FEE_PERCENTILES_MAINNET,
        Network::Testnet => GET_CURRENT_FEE_PERCENTILES_TESTNET,
        Network::Regtest => 0,
    };
    match call_with_payment128::<(GetCurrentFeePercentilesRequest,), (Vec<MillisatoshiPerByte>,)>(
        btc_principal,
        "bitcoin_get_current_fee_percentiles",
        (GetCurrentFeePercentilesRequest {
            network: network.into(),
        },),
        cycles,
    )
    .await
    {
        Ok((percentiles,)) => Ok(percentiles),
        Err((code, msg)) => Err(BitcoinError::CallRejectionError(code, msg)),
    }
}

fn send_transaction_fee(arg: &SendTransactionRequest, network: Network) -> u128 {
    let (submission, payload) = match network {
        Network::Mainnet => (
            SEND_TRANSACTION_SUBMISSION_MAINNET,
            SEND_TRANSACTION_PAYLOAD_MAINNET,
        ),
        Network::Testnet => (
            SEND_TRANSACTION_SUBMISSION_TESTNET,
            SEND_TRANSACTION_PAYLOAD_TESTNET,
        ),
        Network::Regtest => (0, 0),
    };
    submission + payload * arg.transaction.len() as u128
}

/// Sends a (signed) transaction to the bitcoin network.
pub async fn send_transaction(
    network: Network,
    btc_principal: Principal,
    transaction: Vec<u8>,
) -> Result<()> {
    let tx_request = SendTransactionRequest {
        transaction,
        network: network.into(),
    };
    let cycles = send_transaction_fee(&tx_request, network);
    call_with_payment128(
        btc_principal,
        "bitcoin_send_transaction",
        (tx_request,),
        cycles,
    )
    .await
    .map_err(|(code, msg)| BitcoinError::CallRejectionError(code, msg))
}

/// Returns the block headers within the specified height range
pub async fn get_block_headers(
    network: Network,
    btc_principal: Principal,
    start_height: u32,
    end_height: Option<u32>,
) -> Result<GetBlockHeadersResponse> {
    let cycles = match network {
        Network::Mainnet => GET_BLOCK_HEADERS_MAINNET,
        Network::Testnet => GET_BLOCK_HEADERS_TESTNET,
        Network::Regtest => 0,
    };
    match call_with_payment128::<(GetBlockHeadersRequest,), (GetBlockHeadersResponse,)>(
        btc_principal,
        "bitcoin_get_block_headers",
        (GetBlockHeadersRequest {
            start_height,
            end_height,
            network: network.into(),
        },),
        cycles,
    )
    .await
    {
        Ok(headers) => Ok(headers.0),
        Err((code, err)) => Err(BitcoinError::CallRejectionError(code, err)),
    }
}
