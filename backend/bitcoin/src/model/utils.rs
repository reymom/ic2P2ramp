use ic_cdk::api::call::RejectionCode;
use icramp_types::bitcoin::errors::{BitcoinError, Result};

pub async fn handle_ic_call<T>(
    result: std::result::Result<(T,), (RejectionCode, String)>,
) -> Result<T> {
    result
        .map(|response| response.0)
        .map_err(|(code, err)| BitcoinError::CallRejectionError(code, err))
}
