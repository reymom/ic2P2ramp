use candid::Principal;

/// (Helper) Which consensus strategy to use—here, we just ask for Equality.
pub fn solana_vote_quorum() -> sol_rpc_types::ConsensusStrategy {
    sol_rpc_types::ConsensusStrategy::Equality
}

pub fn validate_caller_not_anonymous() -> Principal {
    let principal = ic_cdk::caller();
    if principal == Principal::anonymous() {
        panic!("anonymous principal is not allowed");
    }
    principal
}
