use std::collections::HashMap;

use crate::model::errors::{RampError, Result};
use crate::model::state::{mutate_state, read_state};
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

    pub async fn get_canister_balances() -> Result<HashMap<String, f64>> {
        let mut balances: HashMap<String, f64> = HashMap::new();

        let icp_fees = read_state(|state| state.icp_fees.clone());
        for (ledger_canister, _) in icp_fees.iter() {
            let symbol = Ic2P2ramp::get_symbol(*ledger_canister).await?;
            let decimals = Ic2P2ramp::get_decimals(*ledger_canister).await?;
            let balance: u128 = Ic2P2ramp::get_balance(
                *ledger_canister,
                Account {
                    owner: ic_cdk::api::id(),
                    subaccount: None,
                },
            )
            .await?
            .0
            .try_into()
            .map_err(|e| {
                RampError::InternalError(format!("Cannot parse Nat into u128: e: {:?}", e))
            })?;

            let balance_float = balance as f64 / 10f64.powi(decimals as i32);
            balances.insert(symbol, balance_float);
        }

        Ok(balances)
    }

    pub async fn set_icp_fees(icp_canisters: Vec<String>) -> Result<()> {
        let mut icp_fees = HashMap::new();

        for ledger_canister in icp_canisters {
            let ledger_principal = Principal::from_text(&ledger_canister)
                .map_err(|_| RampError::InvalidInput("Invalid ledger principal".to_string()))?;

            let fee = Ic2P2ramp::get_icp_transaction_fee(&ledger_principal).await?;

            ic_cdk::println!("ledger: {:?}, fees: {:?}", ledger_canister, fee);
            icp_fees.insert(ledger_principal, fee);
        }

        mutate_state(|state| {
            state.icp_fees = icp_fees;
        });

        Ok(())
    }

    async fn get_balance(ledger_principal: Principal, account: Account) -> Result<Nat> {
        let (balance_response,): (Nat,) =
            ic_cdk::call::<(Account,), (Nat,)>(ledger_principal, "icrc1_balance_of", (account,))
                .await
                .map_err(|e| {
                    RampError::CanisterCallError(format!(
                        "Failed to call icrc1_balance_of: {:?}",
                        e
                    ))
                })?;

        Ok(balance_response)
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

    async fn get_symbol(ledger_principal: Principal) -> Result<String> {
        let (symbol_response,): (String,) =
            ic_cdk::call::<(), (String,)>(ledger_principal, "icrc1_symbol", ())
                .await
                .map_err(|e| {
                    RampError::CanisterCallError(format!("Failed to call icrc1_symbol: {:?}", e))
                })?;

        Ok(symbol_response)
    }

    async fn get_decimals(ledger_principal: Principal) -> Result<u8> {
        let (decimals_response,): (u8,) =
            ic_cdk::call::<(), (u8,)>(ledger_principal, "icrc1_decimals", ())
                .await
                .map_err(|e| {
                    RampError::CanisterCallError(format!("Failed to call icrc1_decimals: {:?}", e))
                })?;

        Ok(decimals_response)
    }
}
