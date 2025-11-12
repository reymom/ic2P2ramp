use std::{fmt, hash::Hash};

use candid::{CandidType, Deserialize};

use crate::errors::{Result, SystemError};
use crate::helpers;
use crate::model::types::blockchain::BlockchainType;

// ---------
// Addresses
// ---------

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub enum LoginAddress {
    Email { email: String },
    ICP { principal_id: String },
    EVM { address: String },
    Bitcoin { address: String },
    Solana { address: String },
}

impl LoginAddress {
    pub fn validate(&self) -> Result<()> {
        match self {
            LoginAddress::Email { email } => {
                if email.is_empty() {
                    return Err(SystemError::InvalidInput("Email is empty".to_string()).into());
                }
                helpers::validate_email(email)?
            }
            LoginAddress::ICP { principal_id } => {
                if principal_id.is_empty() {
                    return Err(
                        SystemError::InvalidInput("ICP principal ID is empty".to_string()).into(),
                    );
                }
                helpers::validate_icp_address(principal_id)?;
            }
            LoginAddress::EVM { address } => {
                if address.is_empty() {
                    return Err(
                        SystemError::InvalidInput("EVM address is empty".to_string()).into(),
                    );
                }
                helpers::validate_evm_address(address)?;
            }
            LoginAddress::Bitcoin { address } => {
                if address.is_empty() {
                    return Err(
                        SystemError::InvalidInput("Bitcoin address is empty".to_string()).into(),
                    );
                }
                helpers::validate_bitcoin_address(address)?;
            }
            LoginAddress::Solana { address } => {
                if address.is_empty() {
                    return Err(
                        SystemError::InvalidInput("Solana address is empty".to_string()).into(),
                    );
                }
                helpers::validate_solana_address(address)?;
            }
        }
        Ok(())
    }

    pub fn to_transaction_address(&self) -> Result<TransactionAddress> {
        match self {
            LoginAddress::Email { .. } => Err(SystemError::InvalidInput(
                "Cannot convert Email to TransactionAddress".to_string(),
            )
            .into()),
            LoginAddress::ICP { principal_id } => Ok(TransactionAddress {
                address_type: AddressType::ICP,
                address: principal_id.clone(),
            }),
            LoginAddress::EVM { address } => Ok(TransactionAddress {
                address_type: AddressType::EVM,
                address: address.clone(),
            }),
            LoginAddress::Bitcoin { address } => Ok(TransactionAddress {
                address_type: AddressType::Bitcoin,
                address: address.clone(),
            }),
            LoginAddress::Solana { address } => Ok(TransactionAddress {
                address_type: AddressType::Solana,
                address: address.clone(),
            }),
        }
    }
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq, Hash, PartialOrd)]
pub enum AddressType {
    EVM,
    ICP,
    Bitcoin,
    Solana,
}

impl fmt::Display for AddressType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(CandidType, Deserialize, Clone, Debug, Eq)]
pub struct TransactionAddress {
    pub address_type: AddressType,
    pub address: String,
}

impl PartialEq for TransactionAddress {
    fn eq(&self, other: &Self) -> bool {
        self.address_type == other.address_type
    }
}

impl Hash for TransactionAddress {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.address_type.hash(state);
    }
}

impl TransactionAddress {
    pub fn validate(&self) -> Result<()> {
        if self.address.is_empty() {
            return Err(SystemError::InvalidInput("Address is empty".to_string()).into());
        }

        match self.address_type {
            AddressType::EVM => helpers::validate_evm_address(&self.address),
            AddressType::ICP => helpers::validate_icp_address(&self.address),
            AddressType::Bitcoin => helpers::validate_bitcoin_address(&self.address),
            AddressType::Solana => helpers::validate_solana_address(&self.address),
        }?;

        Ok(())
    }

    pub fn to_blockchain_type(&self) -> BlockchainType {
        match self.address_type {
            AddressType::EVM => BlockchainType::EVM,
            AddressType::ICP => BlockchainType::ICP,
            AddressType::Bitcoin => BlockchainType::Bitcoin,
            AddressType::Solana => BlockchainType::Solana,
        }
    }
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct AuthenticationData {
    pub password: Option<String>,  // For Email
    pub signature: Option<String>, // For EVM and Bitcoin
    pub pubkey: Option<String>,    // For Bitcoin
}
