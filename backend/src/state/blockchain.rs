use candid::{CandidType, Deserialize};

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Blockchain {
    EVM { chain_id: u64 },
    ICP { subnet_id: String },
    Solana,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Crypto {
    pub blockchain: Blockchain,
    pub token: Option<String>, // For EVM tokens, this will be the contract address
    pub amount: u64,
    pub fee: u64,
}

impl Crypto {
    pub fn new(blockchain: Blockchain, token: Option<String>, amount: u64, fee: u64) -> Self {
        Self {
            blockchain,
            token,
            amount,
            fee,
        }
    }
}
