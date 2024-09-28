use candid::{CandidType, Deserialize};

#[derive(CandidType, Debug, Clone, Deserialize)]
pub enum TransactionVariant {
    Native,
    Token,
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub enum TransactionAction {
    Commit,
    Uncommit,
    Cancel(TransactionVariant),
    Release(TransactionVariant),
    Transfer(TransactionVariant),
}

impl TransactionAction {
    pub fn abi(&self) -> &'static str {
        match self {
            &TransactionAction::Commit => COMMIT_ABI,
            &TransactionAction::Uncommit => UNCOMMIT_ABI,
            &TransactionAction::Cancel(TransactionVariant::Native) => CANCEL_NATIVE_ABI,
            &TransactionAction::Cancel(TransactionVariant::Token) => CANCEL_TOKEN_ABI,
            &TransactionAction::Release(TransactionVariant::Native) => RELEASE_NATIVE_ABI,
            &TransactionAction::Release(TransactionVariant::Token) => RELEASE_TOKEN_ABI,
            &TransactionAction::Transfer(TransactionVariant::Token) => TRANSFER_TOKEN_ABI,
            _ => "",
        }
    }

    pub fn function_name(&self) -> &'static str {
        match self {
            &TransactionAction::Commit => "commitDeposit",
            &TransactionAction::Uncommit => "uncommitDeposit",
            &TransactionAction::Cancel(TransactionVariant::Native) => "withdrawBaseCurrency",
            &TransactionAction::Cancel(TransactionVariant::Token) => "withdrawToken",
            &TransactionAction::Release(TransactionVariant::Native) => "releaseBaseCurrency",
            &TransactionAction::Release(TransactionVariant::Token) => "releaseToken",
            &TransactionAction::Transfer(TransactionVariant::Token) => "transfer",
            _ => "",
        }
    }
}

const COMMIT_ABI: &str = r#"
    [
        {
            "inputs": [
                {"internalType": "address", "name": "_offramper", "type": "address"},
                {"internalType": "address", "name": "_token", "type": "address"},
                {"internalType": "uint256", "name": "_amount", "type": "uint256"}
            ],
            "name": "commitDeposit",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        }
    ]
"#;

const UNCOMMIT_ABI: &str = r#"
    [
        {
            "inputs": [
                {"internalType": "address", "name": "_offramper", "type": "address"},
                {"internalType": "address", "name": "_token", "type": "address"},
                {"internalType": "uint256", "name": "_amount", "type": "uint256"}
            ],
            "name": "uncommitDeposit",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        }
    ]
"#;

const CANCEL_NATIVE_ABI: &str = r#"
    [
        {
            "inputs": [
                {"internalType": "address", "name": "_offramper", "type": "address"},
                {"internalType": "uint256", "name": "_amount", "type": "uint256"},
                {"internalType": "uint256", "name": "_fees", "type": "uint256"}
            ],
            "name": "withdrawBaseCurrency",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        }
    ]
"#;

const CANCEL_TOKEN_ABI: &str = r#"
    [
        {
            "inputs": [
                {"internalType": "address", "name": "_offramper", "type": "address"},
                {"internalType": "address", "name": "_token", "type": "address"},
                {"internalType": "uint256", "name": "_amount", "type": "uint256"},
                {"internalType": "uint256", "name": "_fees", "type": "uint256"}
            ],
            "name": "withdrawToken",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        }
    ]
"#;

const RELEASE_NATIVE_ABI: &str = r#"
    [
        {
            "inputs": [
                {"internalType": "address", "name": "_offramper", "type": "address"},
                {"internalType": "address", "name": "_onramper", "type": "address"},
                {"internalType": "uint256", "name": "_amount", "type": "uint256"},
                {"internalType": "uint256", "name": "_fees", "type": "uint256"}
            ],
            "name": "releaseBaseCurrency",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        }
    ]
"#;

const RELEASE_TOKEN_ABI: &str = r#"
    [
        {
            "inputs": [
                {"internalType": "address", "name": "_offramper", "type": "address"},
                {"internalType": "address", "name": "_onramper", "type": "address"},
                {"internalType": "address", "name": "_token", "type": "address"},
                {"internalType": "uint256", "name": "_amount", "type": "uint256"},
                {"internalType": "uint256", "name": "_fees", "type": "uint256"}
            ],
            "name": "releaseToken",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        }
    ]
"#;

const TRANSFER_TOKEN_ABI: &str = r#"
    [
        {
            "inputs": [
                {"internalType": "address", "name": "recipient", "type": "address"},
                {"internalType": "uint256", "name": "amount", "type": "uint256"}
            ],
            "name": "transfer",
            "outputs": [{"internalType": "bool", "name": "", "type": "bool"}],
            "stateMutability": "nonpayable",
            "type": "function"
        }
    ]
"#;
