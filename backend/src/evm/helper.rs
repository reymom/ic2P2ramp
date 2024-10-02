use ethers_core::abi::{Contract, Token};

use crate::model::errors::{BlockchainError, Result};

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
