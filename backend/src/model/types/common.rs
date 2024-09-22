use candid::{CandidType, Deserialize};
use std::hash::Hash;

use crate::errors::{RampError, Result};
use crate::helpers;

// ---------
// Addresses
// ---------

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub enum LoginAddress {
    Email { email: String },
    EVM { address: String },
    ICP { principal_id: String },
    Solana { address: String },
}

impl LoginAddress {
    pub fn validate(&self) -> Result<()> {
        match self {
            LoginAddress::Email { email } => {
                if email.is_empty() {
                    return Err(RampError::InvalidInput("Email is empty".to_string()));
                }
                helpers::validate_email(email)?
            }
            LoginAddress::EVM { address } => {
                if address.is_empty() {
                    return Err(RampError::InvalidInput("EVM address is empty".to_string()));
                }
                helpers::validate_evm_address(address)?;
            }
            LoginAddress::ICP { principal_id } => {
                if principal_id.is_empty() {
                    return Err(RampError::InvalidInput(
                        "ICP principal ID is empty".to_string(),
                    ));
                }
                helpers::validate_icp_address(principal_id)?;
            }
            LoginAddress::Solana { address } => {
                if address.is_empty() {
                    return Err(RampError::InvalidInput(
                        "Solana address is empty".to_string(),
                    ));
                }
                helpers::validate_solana_address(address)?;
            }
        }
        Ok(())
    }

    pub fn to_transaction_address(&self) -> Result<TransactionAddress> {
        match self {
            LoginAddress::Email { .. } => Err(RampError::InvalidInput(
                "Cannot convert Email to TransactionAddress".to_string(),
            )),
            LoginAddress::EVM { address } => Ok(TransactionAddress {
                address_type: AddressType::EVM,
                address: address.clone(),
            }),
            LoginAddress::ICP { principal_id } => Ok(TransactionAddress {
                address_type: AddressType::ICP,
                address: principal_id.clone(),
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
    Solana,
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
            return Err(RampError::InvalidInput("Address is empty".to_string()));
        }

        match self.address_type {
            AddressType::EVM => helpers::validate_evm_address(&self.address),
            AddressType::ICP => helpers::validate_icp_address(&self.address),
            AddressType::Solana => helpers::validate_solana_address(&self.address),
        }?;

        Ok(())
    }
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct AuthenticationData {
    pub password: Option<String>,  // For Email
    pub signature: Option<String>, // For EVM
}
