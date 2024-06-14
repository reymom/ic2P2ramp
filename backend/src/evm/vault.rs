use ethers_core::types::U256;

use crate::evm::rpc::{EvmRpcCanister, SendRawTransactionStatus};
use crate::evm::signer;
use crate::order::management;
use crate::state::mutate_state;
use crate::{RpcApi, RpcServices};

pub const VAULT_MANAGER_ADDRESS: &str = "0x8B1b90637F188541401DeeA100718ca618927E52";
pub const USDT_ADDRESS: &str = "0x0468880bE4970DBab8c9aBE52D9063050652b8db";
pub const RCP_SEPOLIA_MANTLE: &str = "https://rpc.sepolia.mantle.xyz";

pub async fn deposit_funds(chain_id: u64, amount: u64, token_type: String) -> Result<(), String> {
    let gas = U256::from(200_000);
    let fee_estimates = signer::get_fee_estimates();

    let abi = r#"
            [
                {
                    "constant": false,
                    "inputs": [
                        {"name": "token", "type": "address"},
                        {"name": "amount", "type": "uint256"}
                    ],
                    "name": "deposit",
                    "outputs": [{"name": "success", "type": "bool"}],
                    "type": "function"
                }
            ]
        "#;

    let contract = ethers_core::abi::Contract::load(abi.as_bytes()).unwrap();
    let function = contract.function("deposit").unwrap();
    let token_address = match token_type.as_str() {
        "USDT" => USDT_ADDRESS,
        _ => return Err("Unsupported token type".to_string()),
    };
    let data = function
        .encode_input(&[
            ethers_core::abi::Token::Address(token_address.parse().unwrap()),
            ethers_core::abi::Token::Uint(ethers_core::types::U256::from(amount)),
        ])
        .unwrap();

    let value = U256::from(0);
    let request = signer::create_sign_request(
        value,
        Some(VAULT_MANAGER_ADDRESS.to_string()),
        None,
        gas,
        Some(data),
        fee_estimates,
    )
    .await;

    ic_cdk::println!(
        "after request in [deposit_funds] request = {:?}",
        request.gas
    );
    let tx = signer::sign_transaction(request).await;
    ic_cdk::println!("after sign_transaction in [deposit_funds]");
    let status = signer::send_raw_transaction(tx.clone()).await;

    println!("Transaction sent: {:?}", tx);

    match status {
        SendRawTransactionStatus::Ok(transaction_hash) => {
            println!("Success {transaction_hash:?}");
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

pub async fn release_base_currency(order_id: String) -> Result<(), String> {
    let gas = U256::from(60_000);
    let fee_estimates = signer::get_fee_estimates();

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

    let order = management::get_order_by_id(order_id.clone()).await?;
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

    let value = U256::from(amount);
    let request = signer::create_sign_request(
        value,
        Some(VAULT_MANAGER_ADDRESS.to_string()),
        None,
        gas,
        Some(data),
        fee_estimates,
    )
    .await;

    let tx = signer::sign_transaction(request).await;
    ic_cdk::println!("after sign_transaction in [release funds]");
    let status = signer::send_raw_transaction(tx.clone()).await;

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

pub async fn withdraw(chain_id: u64, crypto_amount: u64) -> Result<String, String> {
    let rpc_providers = RpcServices::Custom {
        chainId: chain_id,
        services: vec![RpcApi {
            url: RCP_SEPOLIA_MANTLE.to_string(),
            headers: None,
        }],
    };
    let cycles = 10_000_000_000;

    let result = EvmRpcCanister::withdraw(
        rpc_providers,
        VAULT_MANAGER_ADDRESS.to_string(),
        USDT_ADDRESS.to_string(),
        crypto_amount,
        cycles,
    )
    .await
    .map_err(|e| format!("Call failed: {:?}", e))?;

    Ok(result.0)
}

pub async fn commit_order(chain_id: u64, offramper: String, amount: u64) -> Result<String, String> {
    let rpc_providers = RpcServices::Custom {
        chainId: chain_id,
        services: vec![RpcApi {
            url: RCP_SEPOLIA_MANTLE.to_string(),
            headers: None,
        }],
    };
    let cycles = 10_000_000_000;

    let result = EvmRpcCanister::commit_order(
        rpc_providers,
        VAULT_MANAGER_ADDRESS.to_string(),
        offramper,
        USDT_ADDRESS.to_string(),
        amount,
        cycles,
    )
    .await
    .map_err(|e| format!("Call failed: {:?}", e))?;

    Ok(result.0)
}

pub async fn uncommit_order(
    chain_id: u64,
    offramper: String,
    amount: u64,
) -> Result<String, String> {
    let rpc_providers = RpcServices::Custom {
        chainId: chain_id,
        services: vec![RpcApi {
            url: RCP_SEPOLIA_MANTLE.to_string(),
            headers: None,
        }],
    };
    let cycles = 10_000_000_000;

    let result = EvmRpcCanister::uncommit_order(
        rpc_providers,
        VAULT_MANAGER_ADDRESS.to_string(),
        offramper,
        USDT_ADDRESS.to_string(),
        amount,
        cycles,
    )
    .await
    .map_err(|e| format!("Call failed: {:?}", e))?;

    Ok(result.0)
}

pub async fn release_funds(chain_id: u64, onramper: String, amount: u64) -> Result<String, String> {
    let rpc_providers = RpcServices::Custom {
        chainId: chain_id,
        services: vec![RpcApi {
            url: RCP_SEPOLIA_MANTLE.to_string(),
            headers: None,
        }],
    };
    let cycles = 10_000_000_000;

    let result = EvmRpcCanister::release_funds(
        rpc_providers,
        VAULT_MANAGER_ADDRESS.to_string(),
        onramper,
        USDT_ADDRESS.to_string(),
        amount,
        cycles,
    )
    .await
    .map_err(|e| format!("Call failed: {:?}", e))?;

    Ok(result.0)
}