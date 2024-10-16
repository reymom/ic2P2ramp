use ethers_core::abi::Token;
use ethers_core::types::{Address, U256};
use evm_rpc_canister_types::BlockTag;

use super::fees::{self, eth_get_latest_block};
use super::helper::{self, load_contract_data};
use super::transaction::{broadcast_transaction, create_vault_sign_request};
use super::{estimate_gas, EstimateGasParams};

use crate::errors::{BlockchainError, Result};
use crate::model::{
    helpers,
    memory::heap::{logs, read_state},
};
use crate::types::{
    evm::{
        gas::get_average_gas,
        request::SignRequest,
        transaction::{TransactionAction, TransactionVariant},
    },
    orders::{LockInput, LockedOrder},
};

pub struct Ic2P2ramp;

impl Ic2P2ramp {
    // Gas margin of 20%
    const GAS_MULTIPLIER_NUM: u64 = 12;
    const GAS_MULTIPLIER_DEN: u64 = 10;

    pub fn get_final_gas(estimated_gas: u64) -> u64 {
        estimated_gas * Self::GAS_MULTIPLIER_NUM / Self::GAS_MULTIPLIER_DEN
    }

    pub async fn get_average_gas_price(chain_id: u64, method: &TransactionAction) -> Result<u64> {
        let block = eth_get_latest_block(chain_id, BlockTag::Latest)
            .await
            .map(|block| block.number)?;
        let average_gas = get_average_gas(chain_id, block, None, method)?
            .map(|(gas, _)| Ok(gas))
            .unwrap_or_else(|| Ok(method.default_gas(chain_id)));
        if chain_id == 5003 || chain_id == 5000 {
            return average_gas.map(|gas| gas * 2);
        }

        average_gas
    }

    pub async fn estimate_gas(
        chain_id: u64,
        to_address: String,
        data: Vec<u8>,
        value: Option<String>,
    ) -> Result<Option<u64>> {
        let params = EstimateGasParams::new(
            read_state(|s| s.evm_address.clone()),
            to_address,
            value,
            data,
        );

        estimate_gas(chain_id, params).await
    }

    pub fn commit_inputs(
        offramper: String,
        token_address: Option<String>,
        amount: u128,
    ) -> Result<[Token; 3]> {
        let token_address = token_address.unwrap_or_else(|| format!("{:#x}", Address::zero()));
        Ok([
            Token::Address(helpers::parse_address(offramper)?),
            Token::Address(helpers::parse_address(token_address)?),
            Token::Uint(U256::from(amount)),
        ])
    }

    pub async fn commit_deposit(
        chain_id: u64,
        order_id: u64,
        offramper: String,
        token_address: Option<String>,
        amount: u128,
        estimated_gas: Option<u64>,
        lock_input: LockInput,
    ) -> Result<()> {
        let commit_inputs = Self::commit_inputs(offramper, token_address, amount)?;

        let transaction_type = TransactionAction::Commit;
        let (smart_contract, data) =
            helper::get_vault_and_data(chain_id, &transaction_type, &commit_inputs)?;

        let sign_request = create_vault_sign_request(
            chain_id,
            &transaction_type,
            smart_contract,
            data,
            estimated_gas,
        )
        .await?;

        logs::new_transaction_log(order_id, transaction_type.clone());
        broadcast_transaction(
            order_id,
            chain_id,
            transaction_type,
            sign_request,
            Some(lock_input),
            0,
            false,
        );

        Ok(())
    }

    pub async fn uncommit_deposit(
        chain_id: u64,
        order_id: u64,
        offramper: String,
        token_address: Option<String>,
        amount: u128,
        estimated_gas: Option<u64>,
    ) -> Result<()> {
        let uncommit_inputs = Self::commit_inputs(offramper, token_address, amount)?;

        let transaction_type = TransactionAction::Uncommit;
        let (smart_contract, data) =
            helper::get_vault_and_data(chain_id, &transaction_type, &uncommit_inputs)?;

        let sign_request = create_vault_sign_request(
            chain_id,
            &transaction_type,
            smart_contract,
            data,
            estimated_gas,
        )
        .await?;

        logs::new_transaction_log(order_id, transaction_type.clone());
        broadcast_transaction(
            order_id,
            chain_id,
            transaction_type,
            sign_request,
            None,
            0,
            false,
        );

        Ok(())
    }

    pub fn release_inputs(
        offramper: String,
        onramper: String,
        token: Option<String>,
        amount: u128,
        fee: u128,
    ) -> Result<(Vec<Token>, TransactionAction)> {
        let mut inputs: Vec<Token> = vec![
            Token::Address(helpers::parse_address(offramper)?),
            Token::Address(helpers::parse_address(onramper)?),
            Token::Uint(U256::from(amount)),
            Token::Uint(U256::from(fee)),
        ];

        let mut transaction_variant = TransactionVariant::Native;
        if let Some(token) = token {
            inputs.insert(2, Token::Address(helpers::parse_address(token)?));
            transaction_variant = TransactionVariant::Token;
        }

        Ok((inputs, TransactionAction::Release(transaction_variant)))
    }

    pub async fn release_funds(order: LockedOrder, chain_id: u64) -> Result<()> {
        let crypto = order.base.crypto.clone();
        if crypto.amount < crypto.fee {
            return Err(BlockchainError::FundsBelowFees)?;
        }

        ic_cdk::println!(
            "[release_funds] Releasing base currency with the following details: 
                offramper_address = {}, onramper_address = {}, amount = {}, fees = {}",
            order.base.offramper_address.address,
            order.onramper.address.address,
            order.base.crypto.amount,
            order.base.crypto.fee,
        );

        let (release_inputs, transaction_type) = Self::release_inputs(
            order.base.offramper_address.address,
            order.onramper.address.address,
            crypto.token,
            crypto.amount,
            crypto.fee,
        )?;
        let (smart_contract, data) =
            helper::get_vault_and_data(chain_id, &transaction_type, &release_inputs)?;

        let estimated_gas = Self::get_average_gas_price(chain_id, &transaction_type).await?;
        let sign_request = create_vault_sign_request(
            chain_id,
            &transaction_type,
            smart_contract,
            data,
            Some(estimated_gas),
        )
        .await?;

        logs::new_transaction_log(order.base.id, transaction_type.clone());
        broadcast_transaction(
            order.base.id,
            chain_id,
            transaction_type,
            sign_request,
            None,
            0,
            false,
        );

        Ok(())
    }

    pub fn withdraw_inputs(
        offramper: String,
        amount: u128,
        fees: u128,
        token: Option<String>,
    ) -> Result<(Vec<Token>, TransactionAction)> {
        let mut inputs: Vec<Token> = vec![
            Token::Address(helpers::parse_address(offramper)?),
            Token::Uint(U256::from(amount)),
            Token::Uint(U256::from(fees)),
        ];

        let mut transaction_variant = TransactionVariant::Native;
        if let Some(token) = token {
            inputs.insert(1, Token::Address(helpers::parse_address(token)?));
            transaction_variant = TransactionVariant::Token;
        }

        Ok((inputs, TransactionAction::Cancel(transaction_variant)))
    }

    pub async fn withdraw_deposit(
        chain_id: u64,
        order_id: u64,
        offramper: String,
        token_address: Option<String>,
        amount: u128,
        fees: u128,
    ) -> Result<()> {
        if amount < fees {
            return Err(BlockchainError::FundsBelowFees)?;
        }

        let (withdraw_inputs, transaction_type) =
            Self::withdraw_inputs(offramper, amount, fees, token_address)?;
        let (smart_contract, data) =
            helper::get_vault_and_data(chain_id, &transaction_type, &withdraw_inputs)?;

        let estimated_gas = Self::get_average_gas_price(chain_id, &transaction_type).await?;

        let sign_request = create_vault_sign_request(
            chain_id,
            &transaction_type,
            smart_contract,
            data,
            Some(estimated_gas),
        )
        .await?;

        logs::new_transaction_log(order_id, transaction_type.clone());
        broadcast_transaction(
            order_id,
            chain_id,
            transaction_type,
            sign_request,
            None,
            0,
            false,
        );

        Ok(())
    }

    pub async fn transfer(
        chain_id: u64,
        to: &str,
        value: u128,
        token_address: Option<String>,
        estimated_gas: Option<u64>,
    ) -> Result<()> {
        let fee_estimates = fees::get_fee_estimates(9, chain_id).await?;

        let (request, transaction_type) = match token_address {
            Some(token) => {
                let transaction_type = TransactionAction::Transfer(TransactionVariant::Token);

                let inputs: [Token; 2] = [
                    Token::Address(helpers::parse_address(to.to_string())?),
                    Token::Uint(U256::from(value)),
                ];
                let data = load_contract_data(
                    transaction_type.abi(),
                    transaction_type.function_name(),
                    &inputs,
                )?;
                let rpc_gas =
                    match Self::estimate_gas(chain_id, to.to_string(), data.clone(), None).await {
                        Ok(Some(gas)) => {
                            ic_cdk::println!("[transfer] estimate_gas = {}", gas);
                            U256::from(gas * 120 / 100)
                        }
                        _ => U256::zero(),
                    };
                let backend_gas = U256::from(Ic2P2ramp::get_final_gas(
                    estimated_gas.unwrap_or(transaction_type.default_gas(chain_id)),
                ));
                let gas = std::cmp::max(rpc_gas, backend_gas);

                (
                    SignRequest {
                        chain_id: Some(chain_id.into()),
                        from: None,
                        to: Some(token.to_string()),
                        gas,
                        max_fee_per_gas: Some(fee_estimates.max_fee_per_gas),
                        max_priority_fee_per_gas: Some(fee_estimates.max_priority_fee_per_gas),
                        data: Some(data),
                        value: None,
                        nonce: None,
                    },
                    transaction_type,
                )
            }
            None => {
                let transaction_type = TransactionAction::Transfer(TransactionVariant::Native);

                let value = U256::from(value);
                let rpc_gas = match Self::estimate_gas(
                    chain_id,
                    to.to_string(),
                    Vec::new(),
                    Some(value.to_string()),
                )
                .await
                {
                    Ok(Some(gas)) => {
                        ic_cdk::println!("[transfer] estimate_gas = {}", gas);
                        U256::from(gas * 120 / 100)
                    }
                    _ => U256::zero(),
                };
                let backend_gas = U256::from(Ic2P2ramp::get_final_gas(
                    estimated_gas.unwrap_or(transaction_type.default_gas(chain_id)),
                ));
                let gas = std::cmp::max(rpc_gas, backend_gas);

                let gas_cost = fee_estimates.max_fee_per_gas * gas;
                if value < gas_cost {
                    return Err(BlockchainError::FundsBelowFees)?;
                }
                (
                    SignRequest {
                        chain_id: Some(chain_id.into()),
                        from: None,
                        to: Some(to.to_string()),
                        gas,
                        max_fee_per_gas: Some(fee_estimates.max_fee_per_gas),
                        max_priority_fee_per_gas: Some(fee_estimates.max_priority_fee_per_gas),
                        data: None,
                        value: Some(value - gas_cost),
                        nonce: None,
                    },
                    transaction_type,
                )
            }
        };

        logs::new_transaction_log(0, transaction_type.clone());

        broadcast_transaction(0, chain_id, transaction_type, request, None, 0, false);
        Ok(())
    }
}
