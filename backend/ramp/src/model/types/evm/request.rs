use candid::CandidType;
use ethers_core::abi::ethereum_types::{U256, U64};

#[derive(Clone, Debug, CandidType)]
pub struct SignRequestCandid {
    pub chain_id: u64,
    pub from: Option<String>,
    pub to: Option<String>,
    pub gas: u128,
    pub max_fee_per_gas: Option<u128>,
    pub max_priority_fee_per_gas: Option<u128>,
    pub value: Option<u128>,
    pub nonce: Option<u128>,
    pub data: Option<Vec<u8>>,
}

impl From<SignRequestCandid> for SignRequest {
    fn from(failed_request: SignRequestCandid) -> SignRequest {
        SignRequest {
            chain_id: Some(U64::from(failed_request.chain_id)),
            from: failed_request.from,
            to: failed_request.to,
            gas: U256::from(failed_request.gas),
            max_fee_per_gas: failed_request.max_fee_per_gas.map(U256::from),
            max_priority_fee_per_gas: failed_request.max_priority_fee_per_gas.map(U256::from),
            value: failed_request.value.map(U256::from),
            nonce: failed_request.nonce.map(U256::from),
            data: failed_request.data,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SignRequest {
    pub chain_id: Option<U64>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub gas: U256,
    pub max_fee_per_gas: Option<U256>,
    pub max_priority_fee_per_gas: Option<U256>,
    pub value: Option<U256>,
    pub nonce: Option<U256>,
    pub data: Option<Vec<u8>>,
}

impl SignRequest {
    pub fn add_nonce(&mut self, nonce: U256) {
        self.nonce = Some(nonce)
    }
}

impl From<SignRequest> for SignRequestCandid {
    fn from(sign_request: SignRequest) -> SignRequestCandid {
        SignRequestCandid {
            chain_id: sign_request.chain_id.map_or(0, |id| id.as_u64()),
            from: sign_request.from,
            to: sign_request.to,
            gas: sign_request.gas.as_u128(),
            max_fee_per_gas: sign_request.max_fee_per_gas.map(|fee| fee.as_u128()),
            max_priority_fee_per_gas: sign_request
                .max_priority_fee_per_gas
                .map(|fee| fee.as_u128()),
            value: sign_request.value.map(|val| val.as_u128()),
            nonce: sign_request.nonce.map(|nonce| nonce.as_u128()),
            data: sign_request.data,
        }
    }
}
