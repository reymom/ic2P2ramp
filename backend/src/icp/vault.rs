use std::collections::HashMap;

use crate::model::errors::{RampError, Result};
use crate::model::state::mutate_state;
use candid::{Nat, Principal};
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer::{BlockIndex, NumTokens, TransferArg, TransferError};

pub struct Ic2P2ramp;

impl Ic2P2ramp {
    pub async fn transfer(
        ledger_canister: Principal,
        to: Account,
        amount: NumTokens,
        fee: Option<NumTokens>,
    ) -> Result<BlockIndex> {
        let args = TransferArg {
            memo: None,
            amount,
            fee,
            from_subaccount: None,
            to,
            created_at_time: None,
        };

        ic_cdk::call::<(TransferArg,), (std::result::Result<BlockIndex, TransferError>,)>(
            ledger_canister,
            "icrc1_transfer",
            (args,),
        )
        .await
        .map_err(|e| RampError::CanisterCallError(format!("Failed to call transfer: {:?}", e)))?
        .0
        .map_err(|e| RampError::CanisterCallError(e.to_string()))
    }

    pub async fn set_icp_fees(icp_canisters: Vec<String>) -> Result<()> {
        let mut icp_fees = HashMap::new();

        for ledger_canister in icp_canisters {
            let ledger_principal = Principal::from_text(&ledger_canister)
                .map_err(|_| RampError::InvalidInput("Invalid ledger principal".to_string()))?;

            let fee = Ic2P2ramp::get_icp_transaction_fee(&ledger_principal).await?;

            icp_fees.insert(ledger_principal, fee);
        }

        mutate_state(|state| {
            state.icp_fees = icp_fees;
        });

        Ok(())
    }

    async fn get_icp_transaction_fee(ledger_principal: &Principal) -> Result<Nat> {
        let (fee_response,): (Nat,) = (ic_cdk::call::<(), (Nat,)>(
            *ledger_principal,
            "icrc1_fee",
            (),
        )
        .await
        .map_err(|e| RampError::CanisterCallError(format!("Failed to call icrc1_fee: {:?}", e)))?
        .0,);

        Ok(fee_response)
    }
}
