use std::time::Duration;

use ethers_core::types::{Address, U256};

use super::fees::{self, FeeEstimates};
use super::signer::{self, SignRequest};
use super::{helpers, transaction};

use crate::{
    errors::{RampError, Result},
    state::{mutate_state, storage},
    types::{self, order::OrderState},
};

pub struct Ic2P2ramp;

impl Ic2P2ramp {
    pub async fn approve_token_allowance(
        chain_id: u64,
        token_address: &str,
        gas: u32,
    ) -> Result<()> {
        helpers::validate_evm_address(&token_address)?;
        if types::token_is_approved(chain_id, token_address)? {
            return Err(RampError::TokenAlreadyRegistered);
        };

        let fee_estimates = fees::get_fee_estimates(9, chain_id).await;
        let tx_hash = Self::approve_infinite_allowance(
            chain_id,
            token_address,
            U256::from(gas),
            fee_estimates,
        )
        .await?;

        match transaction::wait_for_transaction_confirmation(
            tx_hash.clone(),
            chain_id,
            60,
            Duration::from_secs(4),
        )
        .await
        {
            Ok(_) => {
                mutate_state(|state| {
                    if let Some(chain_state) = state.chains.get_mut(&chain_id) {
                        chain_state
                            .approved_tokens
                            .insert(token_address.to_string(), true);
                    }
                });
                ic_cdk::println!(
                    "[approve_token_allowance] Added token: {} for chain_id: {}",
                    token_address,
                    chain_id
                );
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    pub async fn deposit_funds(
        chain_id: u64,
        amount: u128,
        token_address: Option<String>,
        gas: Option<u32>,
    ) -> Result<String> {
        let gas = U256::from(gas.unwrap_or(21_000));

        let fee_estimates = fees::get_fee_estimates(9, chain_id).await;
        ic_cdk::println!(
            "[deposit_funds] gas = {:?}, max_fee_per_gas = {:?}, max_priority_fee_per_gas = {:?}",
            gas,
            fee_estimates.max_fee_per_gas,
            fee_estimates.max_priority_fee_per_gas
        );

        let vault_manager_address = types::get_vault_manager_address(chain_id)?;

        let request: SignRequest;
        if let Some(token_address) = token_address {
            if !types::token_is_approved(chain_id, &token_address)? {
                return Err(RampError::TokenUnregistered);
            }

            request = Self::sign_request_deposit_token(
                gas,
                fee_estimates,
                chain_id,
                amount,
                token_address,
                vault_manager_address,
            )
            .await?;
        } else {
            request = Self::sign_request_deposit_base_currency(
                gas,
                fee_estimates,
                chain_id,
                amount,
                vault_manager_address,
            )
            .await?;
        }

        transaction::send_signed_transaction(request, chain_id).await
    }

    async fn sign_request_deposit_token(
        gas: U256,
        fee_estimates: FeeEstimates,
        chain_id: u64,
        amount: u128,
        token_address: String,
        vault_manager_address: String,
    ) -> Result<SignRequest> {
        let abi = r#"
            [
                {
                    "inputs": [
                        {"internalType": "address", "name": "_token", "type": "address"},
                        {"internalType": "uint256", "name": "_amount", "type": "uint256"}
                    ],
                    "name": "depositToken",
                    "outputs": [],
                    "stateMutability": "nonpayable",
                    "type": "function"
                }
            ]
        "#;

        transaction::create_sign_request(
            abi,
            "depositToken",
            gas,
            fee_estimates,
            chain_id,
            U256::from(0),
            vault_manager_address,
            &[
                ethers_core::abi::Token::Address(helpers::parse_address(token_address)?),
                ethers_core::abi::Token::Uint(ethers_core::types::U256::from(amount)),
            ],
        )
        .await
    }

    async fn sign_request_deposit_base_currency(
        gas: U256,
        fee_estimates: FeeEstimates,
        chain_id: u64,
        amount: u128,
        vault_manager_address: String,
    ) -> Result<SignRequest> {
        let abi = r#"
            [
                {
                    "inputs": [],
                    "name": "depositBaseCurrency",
                    "outputs": [],
                    "stateMutability": "payable",
                    "type": "function"
                }
            ]
        "#;

        transaction::create_sign_request(
            abi,
            "depositBaseCurrency",
            gas,
            fee_estimates,
            chain_id,
            U256::from(amount),
            vault_manager_address,
            &[],
        )
        .await
    }

    pub async fn commit_deposit(
        chain_id: u64,
        offramper_address: String,
        token_address: Option<String>,
        amount: u128,
        gas: Option<u32>,
    ) -> Result<String> {
        let gas = U256::from(gas.unwrap_or(21_000));

        let fee_estimates = fees::get_fee_estimates(9, chain_id).await;
        ic_cdk::println!(
            "[commit_deposit] gas = {:?}, max_fee_per_gas = {:?}, max_priority_fee_per_gas = {:?}",
            gas,
            fee_estimates.max_fee_per_gas,
            fee_estimates.max_priority_fee_per_gas
        );

        let vault_manager_address = types::get_vault_manager_address(chain_id)?;
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
        gas: Option<u32>,
    ) -> Result<String> {
        let gas = U256::from(gas.unwrap_or(21_000));

        let fee_estimates = fees::get_fee_estimates(9, chain_id).await;
        ic_cdk::println!(
            "[commit_deposit] gas = {:?}, max_fee_per_gas = {:?}, max_priority_fee_per_gas = {:?}",
            gas,
            fee_estimates.max_fee_per_gas,
            fee_estimates.max_priority_fee_per_gas
        );

        let vault_manager_address = types::get_vault_manager_address(chain_id)?;
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

    pub async fn release_funds(order_id: u64, chain_id: u64, gas: Option<u32>) -> Result<String> {
        let order_state = storage::get_order(&order_id)?;
        let order = match order_state {
            OrderState::Locked(locked_order) => locked_order,
            _ => return Err(RampError::InvalidOrderState(order_state.to_string())),
        };

        let gas = U256::from(gas.unwrap_or(21_000));
        let fee_estimates = fees::get_fee_estimates(9, chain_id).await;
        ic_cdk::println!(
            "[release_funds] gas = {:?}, max_fee_per_gas = {:?}, max_priority_fee_per_gas = {:?}",
            gas,
            fee_estimates.max_fee_per_gas,
            fee_estimates.max_priority_fee_per_gas
        );
        let vault_manager_address = types::get_vault_manager_address(chain_id)?;

        // todo: substract admin_fee to fund the icp evm canister
        let request: SignRequest;
        if let Some(token_address) = order.base.crypto.token {
            if types::token_is_approved(chain_id, &token_address)? {
                return Err(RampError::TokenAlreadyRegistered);
            };
            request = Self::sign_request_release_token(
                gas,
                fee_estimates,
                chain_id,
                order.base.offramper_address.address,
                order.onramper_address.address,
                token_address,
                order.base.crypto.amount,
                vault_manager_address,
            )
            .await?;
        } else {
            request = Self::sign_request_release_base_currency(
                gas,
                fee_estimates,
                chain_id,
                order.base.crypto.amount,
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
        vault_manager_address: String,
    ) -> Result<SignRequest> {
        let abi = r#"
            [
                {
                    "inputs": [
                        {"internalType": "address", "name": "_offramper", "type": "address"},
                        {"internalType": "address", "name": "_onramper", "type": "address"},
                        {"internalType": "address", "name": "_token", "type": "address"},
                        {"internalType": "uint256", "name": "_amount", "type": "uint256"}
                    ],
                    "name": "releaseFunds",
                    "outputs": [],
                    "stateMutability": "nonpayable",
                    "type": "function"
                }
            ]
        "#;

        transaction::create_sign_request(
            abi,
            "releaseFunds",
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
            ],
        )
        .await
    }

    async fn sign_request_release_base_currency(
        gas: U256,
        fee_estimates: FeeEstimates,
        chain_id: u64,
        amount: u128,
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
                        {"internalType": "uint256", "name": "_amount", "type": "uint256"}
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
            ],
        )
        .await
    }

    async fn approve_infinite_allowance(
        chain_id: u64,
        token_address: &str,
        gas: U256,
        fee_estimates: FeeEstimates,
    ) -> Result<String> {
        let abi = r#"
            [
                {
                    "constant": false,
                    "inputs": [
                        {"name": "spender", "type": "address"},
                        {"name": "value", "type": "uint256"}
                    ],
                    "name": "approve",
                    "outputs": [{"name": "success", "type": "bool"}],
                    "type": "function"
                }
            ]
        "#;

        let vault_manager_address = types::get_vault_manager_address(chain_id)?
            .parse()
            .map_err(|e| RampError::EthersAbiError(format!("Invalid address error: {:?}", e)))?;

        let request = transaction::create_sign_request(
            abi,
            "approve",
            gas,
            fee_estimates,
            chain_id,
            U256::from(0),
            token_address.to_string(),
            &[
                ethers_core::abi::Token::Address(vault_manager_address),
                ethers_core::abi::Token::Uint(U256::max_value()),
            ],
        )
        .await?;

        transaction::send_signed_transaction(request, chain_id).await
    }

    pub async fn transfer_eth(
        chain_id: u64,
        to: String,
        value: u128,
        gas: Option<i32>,
    ) -> Result<String> {
        let fee_estimates = fees::get_fee_estimates(9, chain_id).await;
        let gas = U256::from(gas.unwrap_or(30_000));

        let gas_cost = fee_estimates.max_fee_per_gas * gas;
        if U256::from(value) <= gas_cost {
            return Err(RampError::InsufficientFunds);
        }
        let transfer_value = U256::from(value) - gas_cost;

        let request = signer::create_sign_request(
            transfer_value,
            chain_id.into(),
            Some(to),
            None,
            gas,
            None,
            fee_estimates,
        )
        .await;

        transaction::send_signed_transaction(request, chain_id).await
    }
}
