use ic_cdk::api::time;

use crate::{
    errors::{RampError, Result},
    management::user as user_management,
    state::storage::{self, Order, OrderState, PaymentProvider},
};

pub fn create_order(
    fiat_amount: u64,
    currency_symbol: String,
    crypto_amount: u64,
    offramper_providers: Vec<PaymentProvider>,
    offramper_address: String,
    chain_id: u64,
    token_address: Option<String>,
) -> Option<String> {
    let order_id = storage::generate_order_id();
    let order = Order {
        id: order_id.clone(),
        originator: ic_cdk::caller(),
        created_at: time(),
        fiat_amount,
        currency_symbol,
        crypto_amount,
        offramper_providers,
        offramper_address,
        chain_id,
        token_address,
    };

    storage::ORDERS.with(|p| {
        p.borrow_mut()
            .insert(order_id.clone(), OrderState::Created(order))
    });
    Some(order_id)
}

pub fn get_orders() -> Vec<OrderState> {
    storage::ORDERS.with(|p| p.borrow().iter().map(|(_, v)| v.clone()).collect())
}

pub fn get_order_by_id(order_id: &str) -> Result<OrderState> {
    storage::ORDERS.with(|orders| {
        let orders = orders.borrow();
        if let Some(order_state) = orders.get(&order_id.to_string()) {
            Ok(order_state.clone())
        } else {
            Err(RampError::OrderNotFound)
        }
    })
}

pub fn update_order_state(order_id: &str, new_state: OrderState) -> Result<()> {
    storage::ORDERS.with(|orders| {
        let mut orders = orders.borrow_mut();
        if orders.contains_key(&order_id.to_string()) {
            orders.insert(order_id.to_string(), new_state);
            Ok(())
        } else {
            Err(RampError::OrderNotFound)
        }
    })
}

pub fn lock_order(
    order_id: &str,
    onramper_provider: PaymentProvider,
    onramper_address: String,
) -> Result<()> {
    storage::ORDERS.with(|orders| {
        let mut orders = orders.borrow_mut();
        if let Some(order_state) = orders.get(&order_id.to_string()) {
            match order_state {
                OrderState::Created(order) => {
                    orders.remove(&order_id.to_string()).unwrap();
                    orders.insert(
                        order_id.to_string(),
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

pub fn unlock_order(order_id: &str) -> Result<()> {
    storage::ORDERS.with(|orders| {
        let mut orders = orders.borrow_mut();
        if let Some(order_state) = orders.get(&order_id.to_string()) {
            match order_state {
                OrderState::Locked(order) => {
                    let score =
                        user_management::decrease_user_score(&order.clone().onramper_address)?;
                    ic_cdk::println!("[mark_order_as_paid] user score decreased = {:?}", score);

                    orders.remove(&order_id.to_string()).unwrap();
                    orders.insert(order_id.to_string(), OrderState::Created(order.base));
                    Ok(())
                }
                _ => Err(RampError::InvalidOrderState(order_state.to_string())),
            }
        } else {
            Err(RampError::OrderNotFound)
        }
    })
}

pub fn mark_order_as_paid(order_id: &str) -> Result<()> {
    storage::ORDERS.with(|orders| {
        let mut orders = orders.borrow_mut();
        if let Some(order_state) = orders.remove(&order_id.to_string()) {
            match order_state {
                OrderState::Locked(mut locked_order) => {
                    let score = user_management::increase_user_score(
                        &locked_order.clone().onramper_address,
                        locked_order.base.fiat_amount,
                    )?;
                    ic_cdk::println!("[mark_order_as_paid] user score increased = {:?}", score);

                    locked_order.payment_done = true;
                    orders.insert(order_id.to_string(), OrderState::Locked(locked_order));
                    Ok(())
                }
                _ => Err(RampError::InvalidOrderState(order_state.to_string())),
            }
        } else {
            Err(RampError::OrderNotFound)
        }
    })
}

pub fn cancel_order(order_id: &str) -> Result<String> {
    storage::ORDERS.with(|orders| {
        let mut orders = orders.borrow_mut();
        if let Some(order_state) = orders.remove(&order_id.to_string()) {
            match order_state {
                OrderState::Created(_) => {
                    orders.insert(
                        order_id.to_string(),
                        OrderState::Cancelled(order_id.to_string()),
                    );
                    Ok(order_id.to_string())
                }
                _ => Err(RampError::InvalidOrderState(order_state.to_string())),
            }
        } else {
            Err(RampError::OrderNotFound)
        }
    })
}
