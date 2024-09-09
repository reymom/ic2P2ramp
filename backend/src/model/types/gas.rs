use candid::{CandidType, Deserialize};

#[derive(Deserialize, CandidType, Debug)]
pub enum MethodGasUsage {
    Commit,
    ReleaseToken,
    ReleaseNative,
}

#[derive(Clone, Debug, Default)]
pub struct GasRecord {
    gas: u128,
    gas_price: u128,
    block_number: u128,
}

#[derive(Clone, Debug, Default)]
pub struct GasUsage {
    records: Vec<GasRecord>,
}

#[derive(Clone, Debug, Default)]
pub struct ChainGasTracking {
    pub commit_gas: GasUsage,
    pub release_token_gas: GasUsage,
    pub release_native_gas: GasUsage,
}

impl GasUsage {
    // Record gas usage with the block number
    pub fn record_gas_usage(&mut self, gas: u128, gas_price: u128, block_number: u128) {
        self.records.push(GasRecord {
            gas,
            gas_price,
            block_number,
        });
    }

    // Calculate average gas usage for records within the last blocks
    pub fn average_gas(
        &self,
        current_block: u128,
        max_blocks_in_past: u64,
    ) -> Option<(u128, u128)> {
        let relevant_records: Vec<_> = self
            .records
            .iter()
            .filter(|record| {
                current_block.saturating_sub(record.block_number) <= max_blocks_in_past as u128
            })
            .collect();

        if relevant_records.is_empty() {
            return None;
        }

        let length = relevant_records.len() as u128;

        let (total_gas, total_gas_price) =
            relevant_records
                .iter()
                .fold((0u128, 0u128), |(sum_gas, sum_gas_price), record| {
                    (sum_gas + record.gas, sum_gas_price + record.gas_price)
                });

        Some((total_gas / length, total_gas_price / length))
    }
}
