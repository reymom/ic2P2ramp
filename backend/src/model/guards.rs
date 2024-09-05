use super::errors::{RampError, Result};

pub fn only_controller() -> Result<()> {
    if ic_cdk::api::is_controller(&ic_cdk::caller()) {
        Ok(())
    } else {
        Err(RampError::OnlyController)
    }
}
