use std::num::ParseFloatError;

use candid::CandidType;
use ic_cdk::api::call::RejectionCode;
use thiserror::Error;

use crate::{outcalls::xrc_rates::ExchangeRateError, types::PaymentProviderType};

pub type Result<T> = std::result::Result<T, RampError>;

#[derive(Error, Debug, Clone, CandidType)]
pub enum RampError {
    #[error(transparent)]
    UserError(#[from] UserError),

    #[error(transparent)]
    OrderError(#[from] OrderError),

    #[error(transparent)]
    BlockchainError(#[from] BlockchainError),

    #[error(transparent)]
    SystemError(#[from] SystemError),
}

#[derive(Error, Debug, CandidType, Clone)]
pub enum UserError {
    #[error("Only controller is allowed")]
    OnlyController,

    #[error("Password is Invalid")]
    InvalidPassword,

    #[error("Password is Required")]
    PasswordRequired,

    #[error("User Principal is not authorized")]
    UnauthorizedPrincipal,

    #[error("User is not authorized")]
    Unauthorized,

    #[error("Signature is required")]
    SignatureRequired,

    #[error("Signature is not valid")]
    InvalidSignature,

    #[error("Token is Invalid")]
    TokenInvalid,

    #[error("Token is Expired")]
    TokenExpired,

    #[error("Session not Found")]
    SessionNotFound,

    #[error("User Not Found")]
    UserNotFound,

    #[error("User Not Offramper")]
    UserNotOfframper,

    #[error("User Not Onramper")]
    UserNotOnramper,

    #[error("User score below zero")]
    UserBanned,

    #[error("Provider is Not Defined for User {:?}", .0)]
    ProviderNotInUser(PaymentProviderType),
}

#[derive(Error, Debug, CandidType, Clone)]
pub enum OrderError {
    #[error("Order Not Found")]
    OrderNotFound,

    #[error("Order is already being processed")]
    OrderProcessing,

    #[error("Order is not ready to be processed")]
    OrderNotProcessing,

    #[error("Order Timer Not Found")]
    OrderTimerNotFound,

    #[error("Invalid Order State: {0}")]
    InvalidOrderState(String),

    #[error("Order is Uncommitted in the EVM vault")]
    OrderUncommitted,

    #[error("Order is still in Locked time")]
    OrderInLockTime,

    #[error("Payment is already done")]
    PaymentDone,

    #[error("Invalid onramper provider")]
    InvalidOnramperProvider,

    #[error("Invalid offramper provider")]
    InvalidOfframperProvider,

    #[error("Missing Debtor Account")]
    MissingDebtorAccount,

    #[error("Missing Revolut's Access Token")]
    MissingAccessToken,

    #[error("Payment Verification Failed")]
    PaymentVerificationFailed,
}

#[derive(Error, Debug, CandidType, Clone)]
pub enum BlockchainError {
    #[error("Invalid Ethereum address")]
    InvalidAddress,

    #[error("Chain ID not found: {0}")]
    ChainIdNotFound(u64),

    #[error("Vault manager address not found for chain ID: {0}")]
    VaultManagerAddressNotFound(u64),

    #[error("Timeout when waiting for nonce for chain ID: {0}")]
    NonceLockTimeout(u64),

    #[error("Token is unregistered")]
    UnregisteredEvmToken,

    #[error("Transaction timeout")]
    TransactionTimeout,

    #[error("Inconsistent transaction status")]
    InconsistentStatus,

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

    #[error("Replacement transaction underpriced")]
    ReplacementUnderpriced,

    #[error("Fees exceed the funds amount")]
    FundsBelowFees,

    #[error("Funds are too low")]
    FundsTooLow,

    #[error("Ledger principal {0} not supported")]
    LedgerPrincipalNotSupported(String),

    #[error("Blockchain is not supported")]
    UnsupportedBlockchain,

    #[error("EVM Log Error: {0}")]
    EvmLogError(String),

    #[error("Gas Log error: {0}")]
    GasLogError(String),

    #[error("Gas estimation failed")]
    GasEstimationFailed,

    #[error("RPC Provider not found")]
    RpcProviderNotFound,

    #[error("Evm Execution Reverted. Code: {0}, Message: {1}")]
    EvmExecutionReverted(i64, String),
}

#[derive(Error, Debug, CandidType, Clone)]
pub enum SystemError {
    #[error("Invalid Input: {0}")]
    InvalidInput(String),

    #[error("Internal Error: {0}")]
    InternalError(String),

    #[error("Currency Symbol not found")]
    CurrencySymbolNotFound(),

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

    #[error("Rpc Error: {0}")]
    RpcError(String),

    #[error("IC Rejection Code: {0:?}, Error: {1}")]
    ICRejectionError(RejectionCode, String),
}

impl From<ParseFloatError> for SystemError {
    fn from(err: ParseFloatError) -> Self {
        SystemError::ParseFloatError(err.to_string())
    }
}

impl From<serde_json::Error> for SystemError {
    fn from(err: serde_json::Error) -> Self {
        SystemError::ParseError(err.to_string())
    }
}

impl From<rsa::errors::Error> for SystemError {
    fn from(err: rsa::errors::Error) -> Self {
        SystemError::RsaError(err.to_string())
    }
}

impl From<rsa::pkcs8::Error> for SystemError {
    fn from(err: rsa::pkcs8::Error) -> Self {
        SystemError::Pkcs8Error(err.to_string())
    }
}
