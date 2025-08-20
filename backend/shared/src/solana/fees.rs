use candid::{CandidType, Deserialize};
use serde::Serialize;

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct SolanaFeeEstimates {
    /// Estimated lamports for transfer on "lock" (payout path)
    pub lock_lamports: u64,
    /// Estimated lamports for transfer on "withdraw/refund" path
    pub withdraw_lamports: u64,
}
