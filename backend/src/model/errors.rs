use std::num::ParseFloatError;

use candid::CandidType;
use ic_cdk::api::call::RejectionCode;
use thiserror::Error;

use crate::{outcalls::xrc_rates::ExchangeRateError, types::PaymentProviderType};

pub type Result<T> = std::result::Result<T, RampError>;

#[derive(Error, Debug, CandidType)]
pub enum RampError {
    #[error("Order Not Found")]
    OrderNotFound,

    #[error("Order Timer Not Found")]
    OrderTimerNotFound,

    #[error("Invalid Order State: {0}")]
    InvalidOrderState(String),

    #[error("Invalid Ethereum address")]
    InvalidAddress,

    #[error("Provider is Not Defined for User {:?}", .0)]
    ProviderNotInUser(PaymentProviderType),

    #[error("Invalid onramper provider")]
    InvalidOnramperProvider,

    #[error("Invalid offramper provider")]
    InvalidOfframperProvider,

    #[error("User Not Found")]
    UserNotFound,

    #[error("User Not Offramper")]
    UserNotOfframper,

    #[error("User Not Onramper")]
    UserNotOnramper,

    #[error("User score below zero")]
    UserBanned,

    #[error("Invalid Input: {0}")]
    InvalidInput(String),

    #[error("Missing Debtor Account")]
    MissingDebtorAccount,

    #[error("Missing Revolut's Access Token")]
    MissingAccessToken,

    #[error("Chain ID not found: {0}")]
    ChainIdNotFound(u64),

    #[error("Vault manager address not found for chain ID: {0}")]
    VaultManagerAddressNotFound(u64),

    #[error("Token already registered")]
    TokenAlreadyRegistered,

    #[error("Token is unregistered")]
    TokenUnregistered,

    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    #[error("Transaction timeout")]
    TransactionTimeout,

    #[error("Payment Verification Failed")]
    PaymentVerificationFailed,

    #[error("Ethers ABI error: {0}")]
    EthersAbiError(String),

    #[error("Transaction hash is empty")]
    EmptyTransactionHash,

    #[error("Nonce too low")]
    NonceTooLow,

    #[error("Nonce too high")]
    NonceTooHigh,

    #[error("Insufficient funds")]
    InsufficientFunds,

    #[error("Failed to parse response: {0}")]
    ParseError(String),

    #[error("HTTP request failed. RejectionCode: {0:?}, Error: {1}")]
    HttpRequestError(u64, String),

    #[error("Response is not UTF-8 encoded.")]
    Utf8Error,

    #[error("Exchange rate error: {0:?}")]
    ExchangeRateError(ExchangeRateError),

    #[error("Failed to call exchange rate canister: {0}")]
    CanisterCallError(String),

    #[error("Failed to parse float amount: {0}")]
    ParseFloatError(String),

    #[error("pkcs8 error: {0}")]
    Pkcs8Error(String),

    #[error("Rsa Error: {0}")]
    RsaError(String),

    #[error("IC Rejection Code: {0:?}, Error: {1}")]
    ICRejectionError(RejectionCode, String),
}

impl From<ParseFloatError> for RampError {
    fn from(err: ParseFloatError) -> Self {
        RampError::ParseFloatError(err.to_string())
    }
}

impl From<serde_json::Error> for RampError {
    fn from(err: serde_json::Error) -> Self {
        RampError::ParseError(err.to_string())
    }
}

impl From<rsa::errors::Error> for RampError {
    fn from(err: rsa::errors::Error) -> Self {
        RampError::RsaError(err.to_string())
    }
}

impl From<rsa::pkcs8::Error> for RampError {
    fn from(err: rsa::pkcs8::Error) -> Self {
        RampError::Pkcs8Error(err.to_string())
    }
}
