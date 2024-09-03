use super::{
    errors::{RampError, Result},
    state::read_state,
};

pub fn only_controller() -> Result<()> {
    if ic_cdk::api::is_controller(&ic_cdk::caller()) {
        Ok(())
    } else {
        Err(RampError::OnlyController)
    }
}

pub fn only_frontend() -> Result<()> {
    read_state(|state| {
        if let Some(frontend_principal) = state.frontend_canister {
            if ic_cdk::caller() == frontend_principal {
                return Ok(());
            }
        }
        Err(RampError::OnlyFrontend)
    })
}
