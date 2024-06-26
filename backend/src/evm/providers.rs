use super::rpc::{ProviderView, EVM_RPC};

pub async fn get_providers() -> Vec<ProviderView> {
    match EVM_RPC.get_providers().await {
        Ok((res,)) => res,
        Err(e) => ic_cdk::trap(format!("Error: {:?}", e).as_str()),
    }
}
