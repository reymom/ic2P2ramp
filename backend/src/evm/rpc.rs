#![allow(non_snake_case)]

use candid::{self, CandidType, Principal};
use ic_cdk::{self, api::call::CallResult};
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
    pub blockCount: u128,
    pub newestBlock: BlockTag,
    pub rewardPercentiles: Option<serde_bytes::ByteBuf>,
}

#[derive(CandidType, Deserialize)]
pub struct FeeHistory {
    pub reward: Vec<Vec<u128>>,
    pub gasUsedRatio: Vec<f64>,
    pub oldestBlock: u128,
    pub baseFeePerGas: Vec<u128>,
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

#[derive(CandidType, Deserialize)]
pub struct LogEntry {
    pub transactionHash: Option<String>,
    pub blockNumber: Option<u128>,
    pub data: String,
    pub blockHash: Option<String>,
    pub transactionIndex: Option<u128>,
    pub topics: Vec<String>,
    pub address: String,
    pub logIndex: Option<u128>,
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

#[derive(CandidType, Deserialize)]
pub struct TransactionReceipt {
    pub to: String,
    pub status: u128,
    pub transactionHash: String,
    pub blockNumber: u128,
    pub from: String,
    pub logs: Vec<LogEntry>,
    pub blockHash: String,
    pub r#type: String,
    pub transactionIndex: u128,
    pub effectiveGasPrice: u128,
    pub logsBloom: String,
    pub contractAddress: Option<String>,
    pub gasUsed: u128,
}

#[derive(CandidType, Deserialize)]
pub enum GetTransactionReceiptResult {
    Ok(Option<TransactionReceipt>),
    Err(RpcError),
}

#[derive(CandidType, Deserialize)]
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

pub struct EvmRpcCanister;
impl EvmRpcCanister {
    pub async fn eth_get_block_by_number(
        services: RpcServices,
        config: Option<RpcConfig>,
        block_tag: BlockTag,
        cycles: u128,
    ) -> CallResult<(MultiGetBlockByNumberResult,)> {
        ic_cdk::api::call::call_with_payment128(
            CANISTER_ID,
            "eth_getBlockByNumber",
            (services, config, block_tag),
            cycles,
        )
        .await
    }

    pub async fn create_loan(
        services: RpcServices,
        contract_address: String,
        loan_terms_factory_data: Vec<u8>,
        signature: Vec<u8>,
        loan_asset_permit: Vec<u8>,
        collateral_permit: Vec<u8>,
        cycles: u128,
    ) -> CallResult<(String,)> {
        let abi = r#"
            [
                {
                    "constant": false,
                    "inputs": [
                        {"name": "loanTermsFactoryContract", "type": "address"},
                        {"name": "loanTermsFactoryData", "type": "bytes"},
                        {"name": "signature", "type": "bytes"},
                        {"name": "loanAssetPermit", "type": "bytes"},
                        {"name": "collateralPermit", "type": "bytes"}
                    ],
                    "name": "createLOAN",
                    "outputs": [{"name": "loanId", "type": "uint256"}],
                    "type": "function"
                }
            ]
        "#;

        let contract = ethers_core::abi::Contract::load(abi.as_bytes()).unwrap();
        let function = contract.function("createLOAN").unwrap();

        let data = function
            .encode_input(&[
                ethers_core::abi::Token::Address(contract_address.parse().unwrap()),
                ethers_core::abi::Token::Bytes(loan_terms_factory_data),
                ethers_core::abi::Token::Bytes(signature),
                ethers_core::abi::Token::Bytes(loan_asset_permit),
                ethers_core::abi::Token::Bytes(collateral_permit),
            ])
            .unwrap();

        let payload = serde_json::to_string(&JsonRpcRequest {
            id: 1,
            jsonrpc: "2.0".to_string(),
            method: "eth_call".to_string(),
            params: (
                EthCallParams {
                    to: contract_address,
                    data: to_hex(&data),
                },
                "latest".to_string(),
            ),
        })
        .unwrap();

        ic_cdk::api::call::call_with_payment128(CANISTER_ID, "request", (services, payload), cycles)
            .await
    }

    // EVM RPC functions remain mostly unchanged, except for PayPal proof verification
    pub async fn verify_payment_proof_on_evm(
        services: RpcServices,
        proof: Vec<u8>,
        contract_address: String,
        cycles: u128,
    ) -> CallResult<(bool,)> {
        let abi = r#"
            [
                {
                    "constant": true,
                    "inputs": [{"name": "proof", "type": "bytes"}],
                    "name": "verifyPaymentProof",
                    "outputs": [{"name": "valid", "type": "bool"}],
                    "type": "function"
                }
            ]
        "#;
        let contract = ethers_core::abi::Contract::load(abi.as_bytes()).unwrap();
        let function = contract.function("verifyPaymentProof").unwrap();

        let data = function
            .encode_input(&[ethers_core::abi::Token::Bytes(proof)])
            .unwrap();

        let payload = serde_json::to_string(&JsonRpcRequest {
            id: 1,
            jsonrpc: "2.0".to_string(),
            method: "eth_call".to_string(),
            params: (
                EthCallParams {
                    to: contract_address,
                    data: to_hex(&data),
                },
                "latest".to_string(),
            ),
        })
        .unwrap();

        ic_cdk::api::call::call_with_payment128(CANISTER_ID, "request", (services, payload), cycles)
            .await
    }

    pub async fn deposit(
        services: RpcServices,
        contract_address: String,
        token_address: String,
        amount: u64,
        cycles: u128,
    ) -> CallResult<()> {
        let abi = r#"
            [
                {
                    "constant": false,
                    "inputs": [
                        {"name": "token", "type": "address"},
                        {"name": "amount", "type": "uint256"}
                    ],
                    "name": "deposit",
                    "outputs": [{"name": "success", "type": "bool"}],
                    "type": "function"
                }
            ]
        "#;

        let contract = ethers_core::abi::Contract::load(abi.as_bytes()).unwrap();
        let function = contract.function("deposit").unwrap();

        let data = function
            .encode_input(&[
                ethers_core::abi::Token::Address(token_address.parse().unwrap()),
                ethers_core::abi::Token::Uint(ethers_core::types::U256::from(amount)),
            ])
            .unwrap();

        let payload = serde_json::to_string(&JsonRpcRequest {
            id: 1,
            jsonrpc: "2.0".to_string(),
            method: "eth_call".to_string(),
            params: (
                EthCallParams {
                    to: contract_address,
                    data: to_hex(&data),
                },
                "latest".to_string(),
            ),
        })
        .unwrap();

        let max_response_bytes = 2048;
        ic_cdk::api::call::call_with_payment128(
            CANISTER_ID,
            "eth_sendRawTransaction",
            (services, payload, max_response_bytes),
            cycles,
        )
        .await
    }

    pub async fn withdraw(
        services: RpcServices,
        contract_address: String,
        token_address: String,
        amount: u64,
        cycles: u128,
    ) -> CallResult<(String,)> {
        let abi = r#"
            [
                {
                    "constant": false,
                    "inputs": [
                        {"name": "token", "type": "address"},
                        {"name": "amount", "type": "uint256"}
                    ],
                    "name": "withdraw",
                    "outputs": [{"name": "success", "type": "bool"}],
                    "type": "function"
                }
            ]
        "#;

        let contract = ethers_core::abi::Contract::load(abi.as_bytes()).unwrap();
        let function = contract.function("withdraw").unwrap();

        let data = function
            .encode_input(&[
                ethers_core::abi::Token::Address(token_address.parse().unwrap()),
                ethers_core::abi::Token::Uint(ethers_core::types::U256::from(amount)),
            ])
            .unwrap();

        let payload = serde_json::to_string(&JsonRpcRequest {
            id: 1,
            jsonrpc: "2.0".to_string(),
            method: "eth_sendTransaction".to_string(),
            params: (
                EthCallParams {
                    to: contract_address,
                    data: to_hex(&data),
                },
                "latest".to_string(),
            ),
        })
        .unwrap();

        ic_cdk::api::call::call_with_payment128(CANISTER_ID, "request", (services, payload), cycles)
            .await
    }

    pub async fn commit_order(
        services: RpcServices,
        contract_address: String,
        offramper: String,
        token_address: String,
        amount: u64,
        cycles: u128,
    ) -> CallResult<(String,)> {
        let abi = r#"
            [
                {
                    "constant": false,
                    "inputs": [
                        {"name": "offramper", "type": "address"},
                        {"name": "token", "type": "address"},
                        {"name": "amount", "type": "uint256"}
                    ],
                    "name": "commitDeposit",
                    "outputs": [{"name": "success", "type": "bool"}],
                    "type": "function"
                }
            ]
        "#;

        let contract = ethers_core::abi::Contract::load(abi.as_bytes()).unwrap();
        let function = contract.function("commitDeposit").unwrap();

        let data = function
            .encode_input(&[
                ethers_core::abi::Token::Address(offramper.parse().unwrap()),
                ethers_core::abi::Token::Address(token_address.parse().unwrap()),
                ethers_core::abi::Token::Uint(ethers_core::types::U256::from(amount)),
            ])
            .unwrap();

        let payload = serde_json::to_string(&JsonRpcRequest {
            id: 1,
            jsonrpc: "2.0".to_string(),
            method: "eth_sendTransaction".to_string(),
            params: (
                EthCallParams {
                    to: contract_address,
                    data: to_hex(&data),
                },
                "latest".to_string(),
            ),
        })
        .unwrap();

        ic_cdk::api::call::call_with_payment128(CANISTER_ID, "request", (services, payload), cycles)
            .await
    }

    pub async fn uncommit_order(
        services: RpcServices,
        contract_address: String,
        offramper: String,
        token_address: String,
        amount: u64,
        cycles: u128,
    ) -> CallResult<(String,)> {
        let abi = r#"
            [
                {
                    "constant": false,
                    "inputs": [
                        {"name": "offramper", "type": "address"},
                        {"name": "token", "type": "address"},
                        {"name": "amount", "type": "uint256"}
                    ],
                    "name": "uncommitDeposit",
                    "outputs": [{"name": "success", "type": "bool"}],
                    "type": "function"
                }
            ]
        "#;

        let contract = ethers_core::abi::Contract::load(abi.as_bytes()).unwrap();
        let function = contract.function("uncommitDeposit").unwrap();

        let data = function
            .encode_input(&[
                ethers_core::abi::Token::Address(offramper.parse().unwrap()),
                ethers_core::abi::Token::Address(token_address.parse().unwrap()),
                ethers_core::abi::Token::Uint(ethers_core::types::U256::from(amount)),
            ])
            .unwrap();

        let payload = serde_json::to_string(&JsonRpcRequest {
            id: 1,
            jsonrpc: "2.0".to_string(),
            method: "eth_sendTransaction".to_string(),
            params: (
                EthCallParams {
                    to: contract_address,
                    data: to_hex(&data),
                },
                "latest".to_string(),
            ),
        })
        .unwrap();

        ic_cdk::api::call::call_with_payment128(CANISTER_ID, "request", (services, payload), cycles)
            .await
    }

    pub async fn release_funds(
        services: RpcServices,
        contract_address: String,
        onramper: String,
        token_address: String,
        amount: u64,
        cycles: u128,
    ) -> CallResult<(String,)> {
        let abi = r#"
            [
                {
                    "constant": false,
                    "inputs": [
                        {"name": "onramper", "type": "address"},
                        {"name": "token", "type": "address"},
                        {"name": "amount", "type": "uint256"}
                    ],
                    "name": "releaseFunds",
                    "outputs": [{"name": "success", "type": "bool"}],
                    "type": "function"
                }
            ]
        "#;

        let contract = ethers_core::abi::Contract::load(abi.as_bytes()).unwrap();
        let function = contract.function("releaseFunds").unwrap();

        let data = function
            .encode_input(&[
                ethers_core::abi::Token::Address(onramper.parse().unwrap()),
                ethers_core::abi::Token::Address(token_address.parse().unwrap()),
                ethers_core::abi::Token::Uint(ethers_core::types::U256::from(amount)),
            ])
            .unwrap();

        let payload = serde_json::to_string(&JsonRpcRequest {
            id: 1,
            jsonrpc: "2.0".to_string(),
            method: "eth_call".to_string(),
            params: (
                EthCallParams {
                    to: contract_address,
                    data: to_hex(&data),
                },
                "latest".to_string(),
            ),
        })
        .unwrap();

        ic_cdk::api::call::call_with_payment128(
            CANISTER_ID,
            "eth_sendRawTransaction",
            (services, payload),
            cycles,
        )
        .await
    }
}

fn to_hex(data: &[u8]) -> String {
    format!("0x{}", hex::encode(data))
}

