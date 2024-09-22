use candid::{CandidType, Deserialize, Principal};

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Blockchain {
    EVM { chain_id: u64 },
    ICP { ledger_principal: Principal },
    Solana,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Crypto {
    pub blockchain: Blockchain,
    pub token: Option<String>, // For EVM tokens, this will be the contract address
    pub amount: u128,
    pub fee: u128,
}

impl Crypto {
    pub fn new(blockchain: Blockchain, token: Option<String>, amount: u128, fee: u128) -> Self {
        Self {
            blockchain,
            token,
            amount,
            fee,
        }
    }
}
