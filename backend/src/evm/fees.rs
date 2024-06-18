use ethers_core::abi::ethereum_types::U256;

pub struct FeeEstimates {
    pub max_fee_per_gas: U256,
    pub max_priority_fee_per_gas: U256,
}

pub fn get_fee_estimates() -> FeeEstimates {
    let max_fee_per_gas = U256::from(100_000_000_000u64);
    let max_priority_fee_per_gas = U256::from(2_000_000_000);

    FeeEstimates {
        max_fee_per_gas,
        max_priority_fee_per_gas,
    }
}
