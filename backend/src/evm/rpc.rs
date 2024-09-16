#![allow(non_snake_case)]

use candid::{self, CandidType, Principal};
use ic_cdk::api::call::CallResult as Result;
use serde::{Deserialize, Serialize};

pub const CANISTER_ID: Principal =
    Principal::from_slice(b"\x00\x00\x00\x00\x02\x30\x00\xCC\x01\x01"); // 7hfb6-caaaa-aaaar-qadga-cai

#[derive(CandidType, Deserialize)]
pub enum Auth {
    RegisterProvider,
    FreeRpc,
    PriorityRpc,
    Manage,
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub enum EthSepoliaService {
    Alchemy,
    BlockPi,
    PublicNode,
    Ankr,
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct HttpHeader {
    pub value: String,
    pub name: String,
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct RpcApi {
    pub url: String,
    pub headers: Option<Vec<HttpHeader>>,
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub enum EthMainnetService {
    Alchemy,
    BlockPi,
    Cloudflare,
    PublicNode,
    Ankr,
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub enum RpcServices {
    EthSepolia(Option<Vec<EthSepoliaService>>),
    Custom { chainId: u64, services: Vec<RpcApi> },
    EthMainnet(Option<Vec<EthMainnetService>>),
}

#[derive(CandidType, Deserialize)]
pub struct RpcConfig {
    pub responseSizeEstimate: Option<u64>,
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub enum BlockTag {
    Earliest,
    Safe,
    Finalized,
    Latest,
    Number(u128),
    Pending,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EthCallParams {
    pub to: String,
    pub data: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub id: u64,
    pub jsonrpc: String,
    pub method: String,
    pub params: (EthCallParams, String),
}

#[derive(CandidType, Deserialize)]
pub struct FeeHistoryArgs {
    pub blockCount: candid::Nat,
    pub newestBlock: BlockTag,
    pub rewardPercentiles: Option<serde_bytes::ByteBuf>,
}

#[derive(CandidType, Deserialize)]
pub struct FeeHistory {
    pub reward: Vec<Vec<candid::Nat>>,
    pub gasUsedRatio: Vec<f64>,
    pub oldestBlock: candid::Nat,
    pub baseFeePerGas: Vec<candid::Nat>,
}

#[derive(CandidType, Debug, Deserialize)]
pub struct JsonRpcResult {
    result: Option<String>,
    error: Option<JsonRpcError>,
}

#[derive(CandidType, Debug, Deserialize)]
pub struct JsonRpcError {
    code: isize,
    message: String,
}

#[derive(CandidType, Deserialize, Debug)]
pub enum ProviderError {
    TooFewCycles { expected: u128, received: u128 },
    MissingRequiredProvider,
    ProviderNotFound,
    NoPermission,
}

#[derive(CandidType, Deserialize, Debug)]
pub enum ValidationError {
    CredentialPathNotAllowed,
    HostNotAllowed(String),
    CredentialHeaderNotAllowed,
    UrlParseError(String),
    Custom(String),
    InvalidHex(String),
}

#[derive(CandidType, Deserialize, Debug)]
pub enum RejectionCode {
    NoError,
    CanisterError,
    SysTransient,
    DestinationInvalid,
    Unknown,
    SysFatal,
    CanisterReject,
}

#[derive(CandidType, Deserialize, Debug)]
pub enum HttpOutcallError {
    IcError {
        code: RejectionCode,
        message: String,
    },
    InvalidHttpJsonRpcResponse {
        status: u16,
        body: String,
        parsingError: Option<String>,
    },
}

#[derive(CandidType, Deserialize, Debug)]
pub enum RpcError {
    JsonRpcError(JsonRpcError),
    ProviderError(ProviderError),
    ValidationError(ValidationError),
    HttpOutcallError(HttpOutcallError),
}

#[derive(CandidType, Deserialize)]
pub enum FeeHistoryResult {
    Ok(Option<FeeHistory>),
    Err(RpcError),
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub enum RpcService {
    EthSepolia(EthSepoliaService),
    Custom(RpcApi),
    EthMainnet(EthMainnetService),
    Chain(u64),
    Provider(u64),
}

#[derive(CandidType, Deserialize)]
pub enum MultiFeeHistoryResult {
    Consistent(FeeHistoryResult),
    Inconsistent(Vec<(RpcService, FeeHistoryResult)>),
}

#[derive(CandidType, Deserialize)]
pub struct Block {
    pub miner: String,
    pub totalDifficulty: u128,
    pub receiptsRoot: String,
    pub stateRoot: String,
    pub hash: String,
    pub difficulty: u128,
    pub size: u128,
    pub uncles: Vec<String>,
    pub baseFeePerGas: u128,
    pub extraData: String,
    pub transactionsRoot: Option<String>,
    pub sha3Uncles: String,
    pub nonce: u128,
    pub number: u128,
    pub timestamp: u128,
    pub transactions: Vec<String>,
    pub gasLimit: u128,
    pub logsBloom: String,
    pub parentHash: String,
    pub gasUsed: u128,
    pub mixHash: String,
}

#[derive(CandidType, Deserialize)]
pub enum GetBlockByNumberResult {
    Ok(Block),
    Err(RpcError),
}

#[derive(CandidType, Deserialize)]
pub enum MultiGetBlockByNumberResult {
    Consistent(GetBlockByNumberResult),
    Inconsistent(Vec<(RpcService, GetBlockByNumberResult)>),
}

#[derive(CandidType, Deserialize)]
pub struct GetLogsArgs {
    pub fromBlock: Option<BlockTag>,
    pub toBlock: Option<BlockTag>,
    pub addresses: Vec<String>,
    pub topics: Option<Vec<Vec<String>>>,
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct LogEntry {
    pub transactionHash: Option<String>,
    pub blockNumber: Option<candid::Nat>,
    pub data: String,
    pub blockHash: Option<String>,
    pub transactionIndex: Option<candid::Nat>,
    pub topics: Vec<String>,
    pub address: String,
    pub logIndex: Option<candid::Nat>,
    pub removed: bool,
}

#[derive(CandidType, Deserialize)]
pub enum GetLogsResult {
    Ok(Vec<LogEntry>),
    Err(RpcError),
}

#[derive(CandidType, Deserialize)]
pub enum MultiGetLogsResult {
    Consistent(GetLogsResult),
    Inconsistent(Vec<(RpcService, GetLogsResult)>),
}

#[derive(CandidType, Deserialize)]
pub struct GetTransactionCountArgs {
    pub address: String,
    pub block: BlockTag,
}

#[derive(CandidType, Deserialize)]
pub enum GetTransactionCountResult {
    Ok(u128),
    Err(RpcError),
}

#[derive(CandidType, Deserialize)]
pub enum MultiGetTransactionCountResult {
    Consistent(GetTransactionCountResult),
    Inconsistent(Vec<(RpcService, GetTransactionCountResult)>),
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct TransactionReceipt {
    pub to: String,
    pub status: candid::Nat,
    pub transactionHash: String,
    pub blockNumber: candid::Nat,
    pub from: String,
    pub logs: Vec<LogEntry>,
    pub blockHash: String,
    pub r#type: String,
    pub transactionIndex: candid::Nat,
    pub effectiveGasPrice: candid::Nat,
    pub logsBloom: String,
    pub contractAddress: Option<String>,
    pub gasUsed: candid::Nat,
}

#[derive(CandidType, Deserialize, Debug)]
pub enum GetTransactionReceiptResult {
    Ok(Option<TransactionReceipt>),
    Err(RpcError),
}

#[derive(CandidType, Deserialize, Debug)]
pub enum MultiGetTransactionReceiptResult {
    Consistent(GetTransactionReceiptResult),
    Inconsistent(Vec<(RpcService, GetTransactionReceiptResult)>),
}

#[derive(CandidType, Deserialize)]
pub enum SendRawTransactionStatus {
    Ok(Option<String>),
    NonceTooLow,
    NonceTooHigh,
    InsufficientFunds,
}

#[derive(CandidType, Deserialize)]
pub enum SendRawTransactionResult {
    Ok(SendRawTransactionStatus),
    Err(RpcError),
}

#[derive(CandidType, Deserialize)]
pub enum MultiSendRawTransactionResult {
    Consistent(SendRawTransactionResult),
    Inconsistent(Vec<(RpcService, SendRawTransactionResult)>),
}

#[derive(CandidType, Deserialize)]
pub struct Metrics {
    pub cyclesWithdrawn: u128,
    pub responses: Vec<((String, String, String), u64)>,
    pub errNoPermission: u64,
    pub inconsistentResponses: Vec<((String, String), u64)>,
    pub cyclesCharged: Vec<((String, String), u128)>,
    pub requests: Vec<((String, String), u64)>,
    pub errHttpOutcall: Vec<((String, String), u64)>,
    pub errHostNotAllowed: Vec<(String, u64)>,
}

#[derive(CandidType, Deserialize)]
pub struct ProviderView {
    pub cyclesPerCall: u64,
    pub owner: Principal,
    pub hostname: String,
    pub primary: bool,
    pub chainId: u64,
    pub cyclesPerMessageByte: u64,
    pub providerId: u64,
}

#[derive(CandidType, Deserialize)]
pub struct ManageProviderArgs {
    pub service: Option<RpcService>,
    pub primary: Option<bool>,
    pub providerId: u64,
}

#[derive(CandidType, Deserialize)]
pub struct RegisterProviderArgs {
    pub cyclesPerCall: u64,
    pub credentialPath: String,
    pub hostname: String,
    pub credentialHeaders: Option<Vec<HttpHeader>>,
    pub chainId: u64,
    pub cyclesPerMessageByte: u64,
}

#[derive(CandidType, Deserialize)]
pub enum RequestResult {
    Ok(String),
    Err(RpcError),
}

#[derive(CandidType, Deserialize)]
pub enum RequestCostResult {
    Ok(u128),
    Err(RpcError),
}

#[derive(CandidType, Deserialize)]
pub struct UpdateProviderArgs {
    pub cyclesPerCall: Option<u64>,
    pub credentialPath: Option<String>,
    pub hostname: Option<String>,
    pub credentialHeaders: Option<Vec<HttpHeader>>,
    pub primary: Option<bool>,
    pub cyclesPerMessageByte: Option<u64>,
    pub providerId: u64,
}

pub struct Service(pub Principal);
impl Service {
    pub async fn eth_fee_history(
        &self,
        arg0: RpcServices,
        arg1: Option<RpcConfig>,
        arg2: FeeHistoryArgs,
        cycles: u128,
    ) -> Result<(MultiFeeHistoryResult,)> {
        ic_cdk::api::call::call_with_payment128(
            self.0,
            "eth_feeHistory",
            (arg0, arg1, arg2),
            cycles,
        )
        .await
    }
    pub async fn eth_get_block_by_number(
        &self,
        arg0: RpcServices,
        arg1: Option<RpcConfig>,
        arg2: BlockTag,
        cycles: u128,
    ) -> Result<(MultiGetBlockByNumberResult,)> {
        ic_cdk::api::call::call_with_payment128(
            self.0,
            "eth_getBlockByNumber",
            (arg0, arg1, arg2),
            cycles,
        )
        .await
    }
    pub async fn eth_get_transaction_receipt(
        &self,
        arg0: RpcServices,
        arg1: Option<RpcConfig>,
        arg2: String,
        cycles: u128,
    ) -> Result<(MultiGetTransactionReceiptResult,)> {
        ic_cdk::api::call::call_with_payment128(
            self.0,
            "eth_getTransactionReceipt",
            (arg0, arg1, arg2),
            cycles,
        )
        .await
    }
    pub async fn send_raw_transaction(
        &self,
        arg0: RpcServices,
        arg1: Option<RpcConfig>,
        arg2: String,
        cycles: u128,
    ) -> Result<(MultiSendRawTransactionResult,)> {
        ic_cdk::api::call::call_with_payment128(
            self.0,
            "eth_sendRawTransaction",
            (arg0, arg1, arg2),
            cycles,
        )
        .await
    }
}

pub const EVM_RPC: Service = Service(CANISTER_ID);
