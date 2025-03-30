use candid::{CandidType, Deserialize};

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct UnisatState {
    pub api_url: String,
    pub api_key: String,
}

#[derive(Deserialize, Debug)]
pub struct UnisatTxStatusData {
    pub confirmations: u32,
}

#[derive(Deserialize, Debug)]
pub struct UnisatTxStatusResponse {
    // pub code: i32,
    // pub msg: String,
    pub data: UnisatTxStatusData,
}

#[derive(Deserialize, Debug)]
pub struct UnisatTxOut {
    pub txid: String,
    pub vout: u32,
    pub address: String,
    // pub codeType: i32,
    // pub satoshi: u64,
    // pub scriptType: String,
    #[serde(rename = "scriptPk")]
    pub script_pk: String,
    // pub height: u64,
    // pub idx: u32,
    // pub txidSpent: String,
    // pub heightSpent: u64,
}

#[derive(Deserialize, Debug)]
pub struct UnisatTxOutResponse {
    // pub code: i32,
    // pub msg: String,
    pub data: Vec<UnisatTxOut>,
}

#[derive(Deserialize, Debug)]
pub struct UnisatRuneBalance {
    // pub rune: String,
    // pub runeid: String,
    // pub spacedRune: String,
    pub amount: String,
    // pub symbol: String,
    // pub divisibility: u32,
}

#[derive(Deserialize, Debug)]
pub struct UnisatRuneBalanceResponse {
    // pub code: i32,
    pub data: Vec<UnisatRuneBalance>,
}
