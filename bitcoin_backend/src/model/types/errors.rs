use bitcoin::{address::ParseError, amount::ParseAmountError};
use candid::{CandidType, Deserialize};
use ic_cdk::api::call::RejectionCode;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, BitcoinError>;

#[derive(Error, Debug, Clone, CandidType, Deserialize)]
pub enum BitcoinError {
    #[error(transparent)]
    VaultError(#[from] VaultError),

    #[error(transparent)]
    SystemError(#[from] SystemError),

    #[error("Rejection Code: {0:?}, Error: {1}")]
    CallRejectionError(RejectionCode, String),

    #[error("Address parsing error: {0}")]
    ParsingError(String),

    #[error("Parse amount error: {0}")]
    ParseAmountError(String),

    #[error("Internal Error: {0}")]
    InternalError(String),

    #[error("Invalid Input: {0}")]
    InvalidInput(String),

    #[error("Cryptographic Error: {0}")]
    CryptoError(String),

    #[error("Taproot Builder Error: {0}")]
    TaprootError(String),

    #[error("Taproot Builder could not be finalized.")]
    TaprootNotFinalizable,

    #[error(transparent)]
    InsufficientBalance(#[from] InsufficientBalanceError),

    #[error("Unsupported Address Type: {0}")]
    UnsupportedAddressType(String),

    #[error("Unsupported Rune: {0}")]
    UnsupportedRune(String),

    #[error("Invalid Rune ID: {0}")]
    InvalidRuneID(String),

    #[error("Invalid Runestone: {0}")]
    InvalidRunestone(String),

    #[error("Unsupported Transaction")]
    UnsupportedTransaction,
}

#[derive(Error, Debug, Clone, CandidType, Deserialize)]
pub enum VaultError {
    #[error("Address Vault not found")]
    AddressVaultNotFound,

    #[error("Insufficient Balance to Lock")]
    InsufficientBalance,

    #[error("Insufficient Locked Balance")]
    InsufficientLockedBalance,
}

#[derive(Error, Debug, Clone, CandidType, Deserialize)]
pub enum SystemError {
    #[error("HTTP request failed. RejectionCode: {0:?}, Error: {1}")]
    HttpRequestError(u64, String),

    #[error("Failed to parse response: {0}")]
    ParseError(String),

    #[error("Response is not UTF-8 encoded.")]
    Utf8Error,
}

impl From<ParseError> for BitcoinError {
    fn from(error: ParseError) -> Self {
        // Convert the `ParseError` to a string to retain compatibility with `CandidType`
        BitcoinError::ParsingError(error.to_string())
    }
}

impl From<ParseAmountError> for BitcoinError {
    fn from(error: ParseAmountError) -> Self {
        BitcoinError::ParsingError(error.to_string())
    }
}

impl From<serde_json::Error> for SystemError {
    fn from(err: serde_json::Error) -> Self {
        SystemError::ParseError(err.to_string())
    }
}

// Mapping from `bitcoin::secp256k1::Error` to `BitcoinError`
impl From<bitcoin::secp256k1::Error> for BitcoinError {
    fn from(error: bitcoin::secp256k1::Error) -> Self {
        BitcoinError::CryptoError(error.to_string())
    }
}

impl From<bitcoin::key::FromSliceError> for BitcoinError {
    fn from(error: bitcoin::key::FromSliceError) -> Self {
        BitcoinError::CryptoError(error.to_string())
    }
}

impl From<bitcoin::taproot::TaprootBuilderError> for BitcoinError {
    fn from(error: bitcoin::taproot::TaprootBuilderError) -> Self {
        BitcoinError::TaprootError(error.to_string())
    }
}

impl From<bitcoin::sighash::TaprootError> for BitcoinError {
    fn from(error: bitcoin::sighash::TaprootError) -> Self {
        BitcoinError::TaprootError(error.to_string())
    }
}

// Conversion for `TaprootBuilder` into `BitcoinError`
impl From<bitcoin::taproot::TaprootBuilder> for BitcoinError {
    fn from(_: bitcoin::taproot::TaprootBuilder) -> Self {
        BitcoinError::TaprootNotFinalizable
    }
}

impl From<bitcoin::consensus::encode::Error> for BitcoinError {
    fn from(error: bitcoin::consensus::encode::Error) -> Self {
        BitcoinError::InternalError(error.to_string())
    }
}

#[derive(Debug, Error, Clone, CandidType, Deserialize)]
#[error("Insufficient balance: {current_balance} satoshi, trying to transfer {transfer_amount} satoshi with fee {fee}")]
pub struct InsufficientBalanceError {
    pub current_balance: u64,
    pub transfer_amount: u64,
    pub fee: u64,
}
