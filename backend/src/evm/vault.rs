use ethers_core::types::{Address, U256};

use super::fees::{self, FeeEstimates};
use super::signer::{self, SignRequest};
use super::transaction;

use crate::errors::Result;
use crate::model::errors::RampError;
use crate::model::{
    helpers,
    types::{evm::chains, order::LockedOrder},
};

pub struct Ic2P2ramp;

impl Ic2P2ramp {
    pub(crate) const DEFAULT_GAS: u64 = 100_000;
    // Gas margin of 20%
    const GAS_MULTIPLIER_NUM: u64 = 12;
    const GAS_MULTIPLIER_DEN: u64 = 10;

    pub fn get_final_gas(estimated_gas: u64) -> u64 {
        estimated_gas * Self::GAS_MULTIPLIER_NUM / Self::GAS_MULTIPLIER_DEN
    }

    pub async fn commit_deposit(
        chain_id: u64,
        offramper_address: String,
        token_address: Option<String>,
        amount: u128,
        estimated_gas: Option<u64>,
    ) -> Result<String> {
        let gas = U256::from(Ic2P2ramp::get_final_gas(
            estimated_gas.unwrap_or(Self::DEFAULT_GAS),
        ));

        let fee_estimates = fees::get_fee_estimates(9, chain_id).await;
        ic_cdk::println!(
            "[commit_deposit] gas = {:?}, max_fee_per_gas = {:?}, max_priority_fee_per_gas = {:?}",
            gas,
            fee_estimates.max_fee_per_gas,
            fee_estimates.max_priority_fee_per_gas
        );

        let vault_manager_address = chains::get_vault_manager_address(chain_id)?;
        let token_address = token_address.unwrap_or_else(|| format!("{:#x}", Address::zero()));
        let request = Self::sign_request_commit_deposit(
            gas,
            fee_estimates,
            chain_id,
            offramper_address,
            token_address,
            amount,
            vault_manager_address,
        )
        .await?;

        transaction::send_signed_transaction(request, chain_id).await
    }

    async fn sign_request_commit_deposit(
        gas: U256,
        fee_estimates: FeeEstimates,
        chain_id: u64,
        offramper_address: String,
        token_address: String,
        amount: u128,
        vault_manager_address: String,
    ) -> Result<SignRequest> {
        let abi = r#"
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

        transaction::create_sign_request(
            abi,
            "commitDeposit",
            gas,
            fee_estimates,
            chain_id,
            U256::from(0),
            vault_manager_address,
            &[
                ethers_core::abi::Token::Address(helpers::parse_address(offramper_address)?),
                ethers_core::abi::Token::Address(helpers::parse_address(token_address)?),
                ethers_core::abi::Token::Uint(ethers_core::types::U256::from(amount)),
            ],
        )
        .await
    }

    pub async fn uncommit_deposit(
        chain_id: u64,
        offramper_address: String,
        token_address: Option<String>,
        amount: u128,
        estimated_gas: Option<u64>,
    ) -> Result<String> {
        let gas = U256::from(Ic2P2ramp::get_final_gas(
            estimated_gas.unwrap_or(Self::DEFAULT_GAS),
        ));

        let fee_estimates = fees::get_fee_estimates(9, chain_id).await;
        ic_cdk::println!(
            "[commit_deposit] gas = {:?}, max_fee_per_gas = {:?}, max_priority_fee_per_gas = {:?}",
            gas,
            fee_estimates.max_fee_per_gas,
            fee_estimates.max_priority_fee_per_gas
        );

        let vault_manager_address = chains::get_vault_manager_address(chain_id)?;
        let token_address = token_address.unwrap_or_else(|| format!("{:#x}", Address::zero()));
        let request = Self::sign_request_uncommit_deposit(
            gas,
            fee_estimates,
            chain_id,
            offramper_address,
            token_address,
            amount,
            vault_manager_address,
        )
        .await?;

        transaction::send_signed_transaction(request, chain_id).await
    }

    async fn sign_request_uncommit_deposit(
        gas: U256,
        fee_estimates: FeeEstimates,
        chain_id: u64,
        offramper_address: String,
        token_address: String,
        amount: u128,
        vault_manager_address: String,
    ) -> Result<SignRequest> {
        let abi = r#"
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

        transaction::create_sign_request(
            abi,
            "uncommitDeposit",
            gas,
            fee_estimates,
            chain_id,
            U256::from(0),
            vault_manager_address,
            &[
                ethers_core::abi::Token::Address(helpers::parse_address(offramper_address)?),
                ethers_core::abi::Token::Address(helpers::parse_address(token_address)?),
                ethers_core::abi::Token::Uint(ethers_core::types::U256::from(amount)),
            ],
        )
        .await
    }

    pub async fn release_funds(
        order: LockedOrder,
        chain_id: u64,
        estimated_gas: Option<u64>,
    ) -> Result<String> {
        let gas = U256::from(Ic2P2ramp::get_final_gas(
            estimated_gas.unwrap_or(Self::DEFAULT_GAS),
        ));
        let fee_estimates = fees::get_fee_estimates(9, chain_id).await;
        ic_cdk::println!(
            "[release_funds] gas = {:?}, max_fee_per_gas = {:?}, max_priority_fee_per_gas = {:?}",
            gas,
            fee_estimates.max_fee_per_gas,
            fee_estimates.max_priority_fee_per_gas
        );
        let vault_manager_address = chains::get_vault_manager_address(chain_id)?;

        ic_cdk::println!(
            "[release_funds] Releasing base currency with the following details: offramper_address = {}, onramper_address = {}, amount = {}, fees = {}, vault_manager_address = {}",
            order.base.offramper_address.address,
            order.onramper_address.address,
            order.base.crypto.amount,
            order.base.crypto.fee,
            vault_manager_address
        );
        let request: SignRequest;
        if let Some(token_address) = order.base.crypto.token {
            request = Self::sign_request_release_token(
                gas,
                fee_estimates,
                chain_id,
                order.base.offramper_address.address,
                order.onramper_address.address,
                token_address,
                order.base.crypto.amount,
                order.base.crypto.fee,
                vault_manager_address,
            )
            .await?;
        } else {
            request = Self::sign_request_release_base_currency(
                gas,
                fee_estimates,
                chain_id,
                order.base.crypto.amount,
                order.base.crypto.fee,
                order.base.offramper_address.address,
                order.onramper_address.address,
                vault_manager_address,
            )
            .await?;
        }

        transaction::send_signed_transaction(request, chain_id).await
    }

    async fn sign_request_release_token(
        gas: U256,
        fee_estimates: FeeEstimates,
        chain_id: u64,
        offramper_address: String,
        onramper_address: String,
        token_address: String,
        amount: u128,
        fees: u128,
        vault_manager_address: String,
    ) -> Result<SignRequest> {
        let abi = r#"
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

        transaction::create_sign_request(
            abi,
            "releaseToken",
            gas,
            fee_estimates,
            chain_id,
            U256::from(0),
            vault_manager_address,
            &[
                ethers_core::abi::Token::Address(helpers::parse_address(offramper_address)?),
                ethers_core::abi::Token::Address(helpers::parse_address(onramper_address)?),
                ethers_core::abi::Token::Address(helpers::parse_address(token_address)?),
                ethers_core::abi::Token::Uint(ethers_core::types::U256::from(amount)),
                ethers_core::abi::Token::Uint(ethers_core::types::U256::from(fees)),
            ],
        )
        .await
    }

    async fn sign_request_release_base_currency(
        gas: U256,
        fee_estimates: FeeEstimates,
        chain_id: u64,
        amount: u128,
        fees: u128,
        offramper_address: String,
        onramper_address: String,
        vault_manager_address: String,
    ) -> Result<SignRequest> {
        let abi = r#"
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

        transaction::create_sign_request(
            abi,
            "releaseBaseCurrency",
            gas,
            fee_estimates,
            chain_id,
            U256::from(0),
            vault_manager_address,
            &[
                ethers_core::abi::Token::Address(helpers::parse_address(offramper_address)?),
                ethers_core::abi::Token::Address(helpers::parse_address(onramper_address)?),
                ethers_core::abi::Token::Uint(ethers_core::types::U256::from(amount)),
                ethers_core::abi::Token::Uint(ethers_core::types::U256::from(fees)),
            ],
        )
        .await
    }

    pub async fn withdraw_token(
        chain_id: u64,
        offramper: String,
        token_address: String,
        amount: u128,
        fees: u128,
    ) -> Result<String> {
        if amount < fees {
            return Err(RampError::FundsBelowFees);
        }

        let abi = r#"
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

        let fee_estimates = fees::get_fee_estimates(9, chain_id).await;
        let vault_manager_address = chains::get_vault_manager_address(chain_id)?;

        let request = transaction::create_sign_request(
            abi,
            "withdrawToken",
            U256::from(Ic2P2ramp::DEFAULT_GAS),
            fee_estimates,
            chain_id,
            U256::from(0),
            vault_manager_address,
            &[
                ethers_core::abi::Token::Address(helpers::parse_address(offramper)?),
                ethers_core::abi::Token::Address(helpers::parse_address(token_address)?),
                ethers_core::abi::Token::Uint(ethers_core::types::U256::from(amount)),
                ethers_core::abi::Token::Uint(ethers_core::types::U256::from(fees)),
            ],
        )
        .await?;

        transaction::send_signed_transaction(request, chain_id).await
    }

    pub async fn withdraw_base_currency(
        chain_id: u64,
        offramper: String,
        amount: u128,
        fees: u128,
    ) -> Result<String> {
        // ) -> Result<(String, SignRequest)> {
        if amount < fees {
            return Err(RampError::FundsBelowFees);
        }

        let abi = r#"
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

        let fee_estimates = fees::get_fee_estimates(9, chain_id).await;
        let vault_manager_address = chains::get_vault_manager_address(chain_id)?;

        let request = transaction::create_sign_request(
            abi,
            "withdrawBaseCurrency",
            U256::from(Ic2P2ramp::DEFAULT_GAS),
            fee_estimates,
            chain_id,
            U256::from(0),
            vault_manager_address,
            &[
                ethers_core::abi::Token::Address(helpers::parse_address(offramper)?),
                ethers_core::abi::Token::Uint(ethers_core::types::U256::from(amount)),
                ethers_core::abi::Token::Uint(ethers_core::types::U256::from(fees)),
            ],
        )
        .await?;

        let tx_hash = transaction::send_signed_transaction(request, chain_id).await?;

        // return Ok(tx_hash, request)
        return Ok(tx_hash);
    }

    pub async fn transfer(
        chain_id: u64,
        to: &str,
        value: u128,
        token_address: Option<String>,
        estimated_gas: Option<u64>,
    ) -> Result<String> {
        let gas = U256::from(Ic2P2ramp::get_final_gas(
            estimated_gas.unwrap_or(Self::DEFAULT_GAS),
        ));

        let fee_estimates = fees::get_fee_estimates(9, chain_id).await;

        let request: SignRequest;
        if let Some(token_address) = token_address {
            request = Ic2P2ramp::sign_request_transfer_token(
                chain_id,
                gas,
                fee_estimates,
                to,
                &token_address,
                value,
            )
            .await?;
        } else {
            let gas_cost = fee_estimates.max_fee_per_gas * gas;

            if U256::from(value) < gas_cost {
                return Err(RampError::FundsBelowFees);
            }

            let transfer_value = U256::from(value) - gas_cost;

            request = signer::create_sign_request(
                transfer_value,
                chain_id.into(),
                Some(to.to_string()),
                None,
                gas,
                None,
                fee_estimates,
            )
            .await;
        }

        transaction::send_signed_transaction(request, chain_id).await
    }

    async fn sign_request_transfer_token(
        chain_id: u64,
        gas: U256,
        fee_estimates: FeeEstimates,
        to: &str,
        token_address: &str,
        value: u128,
    ) -> Result<SignRequest> {
        let abi = r#"
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

        transaction::create_sign_request(
            abi,
            "transfer",
            gas,
            fee_estimates,
            chain_id,
            U256::from(0),
            token_address.to_string(),
            &[
                ethers_core::abi::Token::Address(helpers::parse_address(to.to_string())?),
                ethers_core::abi::Token::Uint(ethers_core::types::U256::from(value)),
            ],
        )
        .await
    }
}
