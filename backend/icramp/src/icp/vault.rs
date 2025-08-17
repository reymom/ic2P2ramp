use std::collections::HashMap;

use candid::{Nat, Principal};
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer::{BlockIndex, NumTokens, TransferArg, TransferError};

use crate::errors::{Result, SystemError};
use crate::model::memory::heap::{mutate_state, read_state};
use crate::types::icp::IcpToken;

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
        .map_err(|e| SystemError::CanisterCallError(format!("Failed to call transfer: {:?}", e)))?
        .0
        .map_err(|e| SystemError::CanisterCallError(e.to_string()).into())
    }

    pub async fn get_canister_balances() -> Result<HashMap<String, f64>> {
        let mut balances: HashMap<String, f64> = HashMap::new();

        let icp_tokens = read_state(|state| state.icp_tokens.clone());
        for (ledger_canister, token) in icp_tokens.iter() {
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
                SystemError::InternalError(format!("Cannot parse Nat into u128: e: {:?}", e))
            })?;

            let balance_float = balance as f64 / 10f64.powi(token.decimals as i32);
            balances.insert(token.symbol.clone(), balance_float);
        }

        Ok(balances)
    }

    pub async fn register_icp_token(icp_canisters: Vec<String>) -> Result<()> {
        let mut icp_tokens = HashMap::new();

        for ledger_canister in icp_canisters {
            let ledger_principal = Principal::from_text(&ledger_canister)
                .map_err(|_| SystemError::InvalidInput("Invalid ledger principal".to_string()))?;

            let symbol = Ic2P2ramp::get_symbol(&ledger_principal).await?;
            let decimals = Ic2P2ramp::get_decimals(&ledger_principal).await?;
            let fee = Ic2P2ramp::get_icp_transaction_fee(&ledger_principal).await?;

            ic_cdk::println!("ledger: {:?}, fees: {:?}", ledger_canister, fee);
            icp_tokens.insert(ledger_principal, IcpToken::new(&symbol, decimals, fee));
        }

        mutate_state(|state| {
            state.icp_tokens = icp_tokens;
        });

        Ok(())
    }

    async fn get_balance(ledger_principal: Principal, account: Account) -> Result<Nat> {
        let (balance_response,): (Nat,) =
            ic_cdk::call::<(Account,), (Nat,)>(ledger_principal, "icrc1_balance_of", (account,))
                .await
                .map_err(|e| {
                    SystemError::CanisterCallError(format!(
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
        .map_err(|e| SystemError::CanisterCallError(format!("Failed to call icrc1_fee: {:?}", e)))?
        .0,);

        Ok(fee_response)
    }

    async fn get_symbol(ledger_principal: &Principal) -> Result<String> {
        let (symbol_response,): (String,) =
            ic_cdk::call::<(), (String,)>(*ledger_principal, "icrc1_symbol", ())
                .await
                .map_err(|e| {
                    SystemError::CanisterCallError(format!("Failed to call icrc1_symbol: {:?}", e))
                })?;

        Ok(symbol_response)
    }

    async fn get_decimals(ledger_principal: &Principal) -> Result<u8> {
        let (decimals_response,): (u8,) =
            ic_cdk::call::<(), (u8,)>(*ledger_principal, "icrc1_decimals", ())
                .await
                .map_err(|e| {
                    SystemError::CanisterCallError(format!(
                        "Failed to call icrc1_decimals: {:?}",
                        e
                    ))
                })?;

        Ok(decimals_response)
    }
}
