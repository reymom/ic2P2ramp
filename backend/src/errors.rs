use std::num::ParseFloatError;

use candid::CandidType;
use thiserror::Error;

use crate::outcalls::xrc_rates::ExchangeRateError;

pub type Result<T> = std::result::Result<T, RampError>;

#[derive(Error, Debug, CandidType)]
pub enum RampError {
    // #[error("Unauthorized")]
    // Unauthorized,
    #[error("Order Not Found")]
    OrderNotFound,

    #[error("Order Could Not be Created")]
    OrderCreateFailed,

    #[error("Invalid Order State: {0}")]
    InvalidOrderState(String),

    #[error("Invalid Ethereum address")]
    InvalidAddress,

    #[error("Provider is Not Defined for User {0}")]
    ProviderNotInUser(String),

    #[error("Invalid onramper provider")]
    InvalidOnramperProvider,

    #[error("User Not Found")]
    UserNotFound,

    #[error("User Could Not be Created")]
    UserCreateFailed,

    #[error("User score below zero")]
    UserBanned,

    #[error("Invalid Input: {0}")]
    InvalidInput(String),

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
}

impl From<ParseFloatError> for RampError {
    fn from(err: ParseFloatError) -> Self {
        RampError::ParseFloatError(err.to_string())
    }
}
