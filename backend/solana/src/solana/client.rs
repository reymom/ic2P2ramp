use sol_rpc_client::{IcRuntime, SolRpcClient};
use sol_rpc_types::{RpcConfig, RpcSource, RpcSources, SupportedRpcProviderId};

use crate::{memory::heap::config::read_state, model::helpers::solana_vote_quorum};

pub fn client() -> SolRpcClient<IcRuntime> {
    let state = read_state(|s| s.clone());

    SolRpcClient::builder(IcRuntime, state.sol_rpc_canister_id)
        .with_rpc_sources(RpcSources::Custom(vec![RpcSource::Supported(
            SupportedRpcProviderId::AlchemyDevnet,
        )]))
        .with_rpc_config(RpcConfig {
            response_consensus: Some(solana_vote_quorum()),
            ..Default::default()
        })
        .build()
}
