use candid::CandidType;
use sol_rpc_types::RpcError;
use solana_pubkey::ParsePubkeyError;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, SolanaError>;

#[derive(Error, Debug, CandidType)]
pub enum SolanaError {
    #[error(transparent)]
    VaultError(#[from] VaultError),

    #[error(transparent)]
    SystemError(#[from] SystemError),

    #[error("Unsupported token")]
    UnsupportedToken(String),

    #[error("RPC error: {0}")]
    RpcError(String),

    #[error("Account not found")]
    AccountNotFound,

    #[error("Parse pubkey error: {0}")]
    ParsePubkeyError(String),

    #[error("Estimate blockhash errors: {0}")]
    BlockhashErrors(String),
}

#[derive(Error, Debug, CandidType)]
pub enum VaultError {
    #[error("Vault for address not found")]
    AddressVaultNotFound,
    #[error("Insufficient balance")]
    InsufficientBalance,
}

#[derive(Error, Debug)]
pub enum InternalError {}

#[derive(Error, Debug, CandidType)]
pub enum SystemError {
    #[error("HTTP request failed. RejectionCode: {0:?}, Error: {1}")]
    HttpRequestError(u64, String),

    #[error("Failed to parse response: {0}")]
    ParseError(String),

    #[error("Response is not UTF-8 encoded.")]
    Utf8Error,
}

impl From<RpcError> for SolanaError {
    fn from(error: RpcError) -> Self {
        SolanaError::RpcError(error.to_string())
    }
}

impl From<ParsePubkeyError> for SolanaError {
    fn from(error: ParsePubkeyError) -> Self {
        SolanaError::ParsePubkeyError(error.to_string())
    }
}
