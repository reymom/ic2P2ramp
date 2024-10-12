use ethers_core::{
    abi::{Contract, Token},
    types::U256,
};
use evm_rpc_canister_types::TransactionReceipt;

use crate::{
    errors::{BlockchainError, Result},
    types::evm::{chains::get_vault_manager_address, transaction::TransactionAction},
};

pub fn get_vault_and_data(
    chain_id: u64,
    transaction_type: &TransactionAction,
    inputs: &[Token],
) -> Result<(String, Vec<u8>)> {
    Ok((
        get_vault_manager_address(chain_id)?,
        load_contract_data(
            transaction_type.abi(),
            transaction_type.function_name(),
            &inputs,
        )?,
    ))
}

pub fn load_contract_data(abi: &str, function: &str, inputs: &[Token]) -> Result<Vec<u8>> {
    let contract = Contract::load(abi.as_bytes())
        .map_err(|e| BlockchainError::EthersAbiError(format!("Contract load error: {:?}", e)))?;
    let function = contract.function(function).map_err(|e| {
        BlockchainError::EthersAbiError(format!("Function not found error: {:?}", e))
    })?;

    function
        .encode_input(inputs)
        .map_err(|e| BlockchainError::EthersAbiError(format!("Encode input error: {:?}", e)).into())
}

pub fn nat_to_u256(n: &candid::Nat) -> U256 {
    let be_bytes = n.0.to_bytes_be();
    U256::from_big_endian(&be_bytes)
}

pub fn empty_transaction_receipt() -> TransactionReceipt {
    TransactionReceipt {
        to: String::new(),
        status: candid::Nat::default(),
        transactionHash: String::new(),
        blockNumber: candid::Nat::default(),
        from: String::new(),
        logs: Vec::new(),
        blockHash: String::new(),
        r#type: String::new(),
        transactionIndex: candid::Nat::default(),
        effectiveGasPrice: candid::Nat::default(),
        logsBloom: String::new(),
        contractAddress: None,
        gasUsed: candid::Nat::default(),
    }
}
