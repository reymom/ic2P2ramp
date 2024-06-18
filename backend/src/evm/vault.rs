use std::str::FromStr;

use ethers_core::types::U256;

use super::fees::{self, FeeEstimates};
use super::rpc::SendRawTransactionStatus;
use super::signer::{self, SignRequest};
use crate::management;
use crate::state::{mutate_state, read_state};

pub struct Ic2P2ramp;

impl Ic2P2ramp {
    pub fn get_vault_manager_address(&self, chain_id: u64) -> Result<String, String> {
        read_state(|s| {
            s.vault_manager_addresses
                .get(&chain_id)
                .cloned()
                .ok_or_else(|| format!("Vault manager address not found for chain_id {}", chain_id))
        })
    }

    pub async fn deposit_funds(
        chain_id: u64,
        amount: u64,
        token_address: Option<String>,
        gas: Option<String>,
    ) -> Result<(), String> {
        let gas = U256::from_str(gas.unwrap_or("21000".to_string()).as_str())
            .unwrap_or(U256::from(21_000));

        let fee_estimates = fees::get_fee_estimates(9, chain_id).await;
        println!(
            "gas = {:?}, max_fee_per_gas = {:?}, max_priority_fee_per_gas = {:?}",
            gas, fee_estimates.max_fee_per_gas, fee_estimates.max_priority_fee_per_gas
        );

        let vault_manager_address = Self.get_vault_manager_address(chain_id)?;

        let request: SignRequest;
        if let Some(token_address) = token_address {
            request = Self::sign_request_deposit_token(
                gas,
                fee_estimates,
                chain_id,
                amount,
                token_address,
                &vault_manager_address,
            )
            .await?;
        } else {
            request = Self::sign_request_deposit_base_currency(
                gas,
                fee_estimates,
                chain_id,
                amount,
                &vault_manager_address,
            )
            .await?;
        }

        let tx = signer::sign_transaction(request).await;
        println!("Transaction sent: {:?}", tx);

        let status = signer::send_raw_transaction(tx.clone(), chain_id).await;
        match status {
            SendRawTransactionStatus::Ok(transaction_hash) => {
                println!("Success {transaction_hash:?}");
                mutate_state(|s| {
                    s.nonce += U256::from(1);
                });
                Ok(())
            }
            SendRawTransactionStatus::NonceTooLow => {
                let msg = "Nonce too low".to_string();
                println!("{}", msg);
                Err(msg)
            }
            SendRawTransactionStatus::NonceTooHigh => {
                let msg = "Nonce too high".to_string();
                println!("{}", msg);
                Err(msg)
            }
            SendRawTransactionStatus::InsufficientFunds => {
                let msg = "Insufficient funds".to_string();
                println!("{}", msg);
                Err(msg)
            }
        }
    }

    async fn sign_request_deposit_token(
        gas: U256,
        fee_estimates: FeeEstimates,
        chain_id: u64,
        amount: u64,
        token_address: String,
        vault_manager_address: &String,
    ) -> Result<SignRequest, String> {
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

        let contract = ethers_core::abi::Contract::load(abi.as_bytes()).unwrap();
        let function = contract.function("depositToken").unwrap();
        let data = function
            .encode_input(&[
                ethers_core::abi::Token::Address(token_address.parse().unwrap()),
                ethers_core::abi::Token::Uint(ethers_core::types::U256::from(amount)),
            ])
            .unwrap();

        let value = U256::from(0);
        Ok(signer::create_sign_request(
            value,
            chain_id.into(),
            Some(vault_manager_address.clone()),
            None,
            gas,
            Some(data),
            fee_estimates,
        )
        .await)
    }

    async fn sign_request_deposit_base_currency(
        gas: U256,
        fee_estimates: FeeEstimates,
        chain_id: u64,
        amount: u64,
        vault_manager_address: &String,
    ) -> Result<SignRequest, String> {
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

        let contract = ethers_core::abi::Contract::load(abi.as_bytes()).unwrap();
        let function = contract.function("depositBaseCurrency").unwrap();
        let data = function.encode_input(&[]).unwrap();

        Ok(signer::create_sign_request(
            U256::from(amount),
            chain_id.into(),
            Some(vault_manager_address.clone()),
            None,
            gas,
            Some(data),
            fee_estimates,
        )
        .await)
    }

    pub async fn release_base_currency(chain_id: u64, order_id: String) -> Result<(), String> {
        let gas = U256::from(60_000);
        let fee_estimates = fees::get_fee_estimates(9, chain_id).await;

        let abi = r#"
            [
                {
                    "constant": false,
                    "inputs": [
                        {"name": "_onramper", "type": "address"},
                        {"name": "_amount", "type": "uint256"}
                    ],
                    "name": "releaseBaseCurrency",
                    "outputs": [],
                    "stateMutability": "payable",
                    "type": "function"
                }
            ]
        "#;

        let contract = ethers_core::abi::Contract::load(abi.as_bytes()).unwrap();
        let function = contract.function("releaseBaseCurrency").unwrap();

        let order = management::order::get_order_by_id(order_id.clone())?;
        let onramper_address = order
            .onramper_address
            .expect("onramper address should be setup");
        let amount = order.crypto_amount;

        let data = function
            .encode_input(&[
                ethers_core::abi::Token::Address(onramper_address.parse().unwrap()),
                ethers_core::abi::Token::Uint(ethers_core::types::U256::from(amount)),
            ])
            .unwrap();

        let vault_manager_address = Self.get_vault_manager_address(chain_id)?;
        let value = U256::from(amount);
        let request = signer::create_sign_request(
            value,
            chain_id.into(),
            Some(vault_manager_address.clone()),
            None,
            gas,
            Some(data),
            fee_estimates,
        )
        .await;

        let tx = signer::sign_transaction(request).await;
        ic_cdk::println!("after sign_transaction in [release funds]");
        let status = signer::send_raw_transaction(tx.clone(), chain_id).await;

        match status {
            SendRawTransactionStatus::Ok(transaction_hash) => {
                ic_cdk::println!("Success {transaction_hash:?}");
                mutate_state(|s| {
                    s.nonce += U256::from(1);
                });
            }
            SendRawTransactionStatus::NonceTooLow => {
                println!("Nonce too low");
            }
            SendRawTransactionStatus::NonceTooHigh => {
                println!("Nonce too high");
            }
            SendRawTransactionStatus::InsufficientFunds => {
                println!("Insufficient funds");
            }
        }

        Ok(())
    }
}

// pub async fn withdraw(chain_id: u64, crypto_amount: u64) -> Result<String, String> {
//     let rpc_providers = RpcServices::Custom {
//         chainId: chain_id,
//         services: vec![RpcApi {
//             url: RCP_SEPOLIA_MANTLE.to_string(),
//             headers: None,
//         }],
//     };
//     let cycles = 10_000_000_000;

//     let result = EvmRpcCanister::withdraw(
//         rpc_providers,
//         VAULT_MANAGER_ADDRESS.to_string(),
//         USDT_ADDRESS.to_string(),
//         crypto_amount,
//         cycles,
//     )
//     .await
//     .map_err(|e| format!("Call failed: {:?}", e))?;

//     Ok(result.0)
// }

// pub async fn commit_order(chain_id: u64, offramper: String, amount: u64) -> Result<String, String> {
//     let rpc_providers = RpcServices::Custom {
//         chainId: chain_id,
//         services: vec![RpcApi {
//             url: RCP_SEPOLIA_MANTLE.to_string(),
//             headers: None,
//         }],
//     };
//     let cycles = 10_000_000_000;

//     let result = EvmRpcCanister::commit_order(
//         rpc_providers,
//         VAULT_MANAGER_ADDRESS.to_string(),
//         offramper,
//         USDT_ADDRESS.to_string(),
//         amount,
//         cycles,
//     )
//     .await
//     .map_err(|e| format!("Call failed: {:?}", e))?;

//     Ok(result.0)
// }

// pub async fn uncommit_order(
//     chain_id: u64,
//     offramper: String,
//     amount: u64,
// ) -> Result<String, String> {
//     let rpc_providers = RpcServices::Custom {
//         chainId: chain_id,
//         services: vec![RpcApi {
//             url: RCP_SEPOLIA_MANTLE.to_string(),
//             headers: None,
//         }],
//     };
//     let cycles = 10_000_000_000;

//     let result = EvmRpcCanister::uncommit_order(
//         rpc_providers,
//         VAULT_MANAGER_ADDRESS.to_string(),
//         offramper,
//         USDT_ADDRESS.to_string(),
//         amount,
//         cycles,
//     )
//     .await
//     .map_err(|e| format!("Call failed: {:?}", e))?;

//     Ok(result.0)
// }

// pub async fn release_funds(chain_id: u64, onramper: String, amount: u64) -> Result<String, String> {
//     let rpc_providers = RpcServices::Custom {
//         chainId: chain_id,
//         services: vec![RpcApi {
//             url: RCP_SEPOLIA_MANTLE.to_string(),
//             headers: None,
//         }],
//     };
//     let cycles = 10_000_000_000;

//     let result = EvmRpcCanister::release_funds(
//         rpc_providers,
//         VAULT_MANAGER_ADDRESS.to_string(),
//         onramper,
//         USDT_ADDRESS.to_string(),
//         amount,
//         cycles,
//     )
//     .await
//     .map_err(|e| format!("Call failed: {:?}", e))?;

//     Ok(result.0)
// }
