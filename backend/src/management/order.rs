use std::collections::HashSet;

use crate::{
    errors::{RampError, Result},
    management::user as user_management,
    state::storage::{self, Order, OrderState, PaymentProvider},
};

pub fn create_order(
    fiat_amount: u64,
    currency_symbol: String,
    crypto_amount: u64,
    offramper_providers: HashSet<PaymentProvider>,
    offramper_address: String,
    chain_id: u64,
    token_address: Option<String>,
) -> Result<u64> {
    let order = Order::new(
        fiat_amount,
        currency_symbol,
        crypto_amount,
        offramper_providers,
        offramper_address,
        chain_id,
        token_address,
    )?;

    match storage::insert_order(&order) {
        Some(OrderState::Created(order)) => Ok(order.id),
        Some(order_state) => Err(RampError::InvalidOrderState(order_state.to_string())),
        None => Err(RampError::OrderCreateFailed),
    }
}

pub fn get_orders() -> Vec<OrderState> {
    storage::ORDERS.with(|p| p.borrow().iter().map(|(_, v)| v.clone()).collect())
}

pub fn update_order_state(order_id: u64, new_state: OrderState) -> Result<()> {
    storage::ORDERS.with(|orders| {
        let mut orders = orders.borrow_mut();
        if orders.contains_key(&order_id) {
            orders.insert(order_id, new_state);
            Ok(())
        } else {
            Err(RampError::OrderNotFound)
        }
    })
}

pub fn lock_order(
    order_id: u64,
    onramper_provider: PaymentProvider,
    onramper_address: String,
) -> Result<()> {
    storage::ORDERS.with(|orders| {
        let mut orders = orders.borrow_mut();
        if let Some(order_state) = orders.get(&order_id) {
            match order_state {
                OrderState::Created(order) => {
                    orders.remove(&order_id).unwrap();
                    orders.insert(
                        order_id,
                        OrderState::Locked(order.lock(onramper_provider, onramper_address)),
                    );
                    Ok(())
                }
                _ => Err(RampError::InvalidOrderState(order_state.to_string())),
            }
        } else {
            Err(RampError::OrderNotFound)
        }
    })
}

pub fn unlock_order(order_id: u64) -> Result<()> {
    storage::ORDERS.with(|orders| {
        let mut orders = orders.borrow_mut();
        if let Some(order_state) = orders.get(&order_id) {
            match order_state {
                OrderState::Locked(order) => {
                    let score =
                        user_management::decrease_user_score(&order.clone().onramper_address)?;
                    ic_cdk::println!("[mark_order_as_paid] user score decreased = {:?}", score);

                    orders.remove(&order_id);
                    orders.insert(order_id, OrderState::Created(order.base));
                    Ok(())
                }
                _ => Err(RampError::InvalidOrderState(order_state.to_string())),
            }
        } else {
            Err(RampError::OrderNotFound)
        }
    })
}

pub fn mark_order_as_paid(order_id: u64) -> Result<()> {
    storage::ORDERS.with(|orders| {
        let mut orders = orders.borrow_mut();
        if let Some(order_state) = orders.remove(&order_id) {
            match order_state {
                OrderState::Locked(mut locked_order) => {
                    let score = user_management::increase_user_score(
                        &locked_order.clone().onramper_address,
                        locked_order.base.fiat_amount,
                    )?;
                    ic_cdk::println!("[mark_order_as_paid] user score increased = {:?}", score);

                    locked_order.payment_done = true;
                    orders.insert(order_id, OrderState::Locked(locked_order));
                    Ok(())
                }
                _ => Err(RampError::InvalidOrderState(order_state.to_string())),
            }
        } else {
            Err(RampError::OrderNotFound)
        }
    })
}

pub fn cancel_order(order_id: u64) -> Result<String> {
    storage::ORDERS.with(|orders| {
        let mut orders = orders.borrow_mut();
        if let Some(order_state) = orders.remove(&order_id) {
            match order_state {
                OrderState::Created(_) => {
                    orders.insert(order_id, OrderState::Cancelled(order_id));
                    Ok(order_id.to_string())
                }
                _ => Err(RampError::InvalidOrderState(order_state.to_string())),
            }
        } else {
            Err(RampError::OrderNotFound)
        }
    })
}
