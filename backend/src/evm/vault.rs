use ethers_core::abi::Token;
use ethers_core::types::{Address, U256};

use super::fees::{self, eth_get_latest_block};
use super::helper::load_contract_data;
use super::rpc::BlockTag;
use super::transaction::create_vault_sign_request;

use crate::errors::{BlockchainError, Result};
use crate::evm::transaction::broadcast_transaction;
use crate::model::types::evm::gas::get_average_gas;
use crate::model::{helpers, memory::heap::logs};
use crate::types::{
    evm::{
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
        let block = eth_get_latest_block(chain_id, BlockTag::Latest).await?;
        get_average_gas(chain_id, block.number, None, &method)?
            .map(|(gas, _)| Ok(gas))
            .unwrap_or_else(|| Ok(method.default_gas(chain_id)))
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
        let token_address = token_address.unwrap_or_else(|| format!("{:#x}", Address::zero()));
        let inputs: [Token; 3] = [
            Token::Address(helpers::parse_address(offramper)?),
            Token::Address(helpers::parse_address(token_address)?),
            Token::Uint(U256::from(amount)),
        ];

        let transaction_type = TransactionAction::Commit;
        let sign_request =
            create_vault_sign_request(chain_id, &transaction_type, &inputs, estimated_gas).await?;

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
        offramper_address: String,
        token_address: Option<String>,
        amount: u128,
        estimated_gas: Option<u64>,
    ) -> Result<()> {
        let token_address = token_address.unwrap_or_else(|| format!("{:#x}", Address::zero()));
        let inputs: [Token; 3] = [
            Token::Address(helpers::parse_address(offramper_address)?),
            Token::Address(helpers::parse_address(token_address)?),
            Token::Uint(U256::from(amount)),
        ];

        let transaction_type = TransactionAction::Uncommit;
        let sign_request =
            create_vault_sign_request(chain_id, &transaction_type, &inputs, estimated_gas).await?;

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

        let mut inputs: Vec<Token> = Vec::new();
        inputs.push(Token::Address(helpers::parse_address(
            order.base.offramper_address.address,
        )?));
        inputs.push(Token::Address(helpers::parse_address(
            order.onramper.address.address,
        )?));
        inputs.push(Token::Uint(U256::from(crypto.amount)));
        inputs.push(Token::Uint(U256::from(crypto.fee)));

        let mut transaction_variant = TransactionVariant::Native;
        if let Some(token) = crypto.token {
            inputs.insert(2, Token::Address(helpers::parse_address(token)?));
            transaction_variant = TransactionVariant::Token;
        }
        let transaction_type = TransactionAction::Release(transaction_variant);
        let estimated_gas = Self::get_average_gas_price(chain_id, &transaction_type).await?;

        let sign_request =
            create_vault_sign_request(chain_id, &transaction_type, &inputs, Some(estimated_gas))
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

        let mut inputs: Vec<Token> = Vec::new();
        inputs.push(Token::Address(helpers::parse_address(offramper)?));
        inputs.push(Token::Uint(U256::from(amount)));
        inputs.push(Token::Uint(U256::from(fees)));

        let mut transaction_variant = TransactionVariant::Native;
        if let Some(token) = token_address {
            inputs.insert(1, Token::Address(helpers::parse_address(token)?));
            transaction_variant = TransactionVariant::Token;
        }
        let transaction_type = TransactionAction::Cancel(transaction_variant);
        let estimated_gas = Self::get_average_gas_price(chain_id, &transaction_type).await?;

        let sign_request =
            create_vault_sign_request(chain_id, &transaction_type, &inputs, Some(estimated_gas))
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
                let gas = U256::from(Ic2P2ramp::get_final_gas(
                    estimated_gas.unwrap_or(transaction_type.default_gas(chain_id)),
                ));
                let inputs: [Token; 2] = [
                    Token::Address(helpers::parse_address(to.to_string())?),
                    Token::Uint(U256::from(value)),
                ];
                let data = load_contract_data(
                    transaction_type.abi(),
                    transaction_type.function_name(),
                    &inputs,
                )?;

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
                let gas = U256::from(Ic2P2ramp::get_final_gas(
                    estimated_gas.unwrap_or(transaction_type.default_gas(chain_id)),
                ));
                let gas_cost = fee_estimates.max_fee_per_gas * gas;
                let value = U256::from(value);
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
