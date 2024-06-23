use crate::state::storage::{self, Order, OrderState, PaymentProvider};

pub fn create_order(
    fiat_amount: u64,
    currency_symbol: String,
    crypto_amount: u64,
    offramper_providers: Vec<PaymentProvider>,
    offramper_address: String,
    chain_id: u64,
    token_address: Option<String>,
) -> Result<String, String> {
    let order_id = storage::generate_order_id();
    let order = Order {
        id: order_id.clone(),
        originator: ic_cdk::caller(),
        fiat_amount,
        currency_symbol,
        crypto_amount,
        offramper_providers,
        onramper_provider: None,
        offramper_address,
        onramper_address: None,
        chain_id,
        token_address,
    };

    storage::ORDERS.with(|p| {
        p.borrow_mut()
            .insert(order_id.clone(), OrderState::Created(order))
    });
    Ok(order_id)
}

pub fn get_orders() -> Vec<OrderState> {
    storage::ORDERS.with(|p| p.borrow().iter().map(|(_, v)| v.clone()).collect())
}

pub fn get_order_state_by_id(order_id: &str) -> Result<OrderState, String> {
    storage::ORDERS.with(|orders| {
        let orders = orders.borrow();
        if let Some(order_state) = orders.get(&order_id.to_string()) {
            Ok(order_state.clone())
        } else {
            Err("Order not found".to_string())
        }
    })
}

pub fn update_order_state(order_id: &str, new_state: OrderState) -> Result<(), String> {
    storage::ORDERS.with(|orders| {
        let mut orders = orders.borrow_mut();
        if orders.contains_key(&order_id.to_string()) {
            orders.insert(order_id.to_string(), new_state);
            Ok(())
        } else {
            Err("Order not found".to_string())
        }
    })
}

pub fn lock_order(
    order_id: &str,
    onramper_provider: PaymentProvider,
    onramper_address: String,
) -> Result<String, String> {
    storage::ORDERS.with(|orders| {
        let mut orders = orders.borrow_mut();
        if let Some(order_state) = orders.remove(&order_id.to_string()) {
            match order_state {
                OrderState::Created(order) => {
                    orders.insert(
                        order_id.to_string(),
                        OrderState::Locked(order.lock(onramper_provider, onramper_address)),
                    );
                    Ok("Order locked".to_string())
                }
                _ => Err("Order is not in a creatable state".to_string()),
            }
        } else {
            Err("order not found".to_string())
        }
    })
}

pub fn mark_order_as_paid(order_id: &str) -> Result<(), String> {
    ic_cdk::println!("[mark_order_as_paid");
    storage::ORDERS.with(|orders| {
        let mut orders = orders.borrow_mut();
        if let Some(order_state) = orders.remove(&order_id.to_string()) {
            match order_state {
                OrderState::Locked(mut locked_order) => {
                    locked_order.payment_done = true;
                    orders.insert(order_id.to_string(), OrderState::Locked(locked_order));
                    Ok(())
                }
                _ => Err("Order is not in a lockable state".to_string()),
            }
        } else {
            Err("Order not found".to_string())
        }
    })
}

pub fn remove_order(order_id: &str) -> Result<String, String> {
    storage::ORDERS.with(|orders| {
        let mut orders = orders.borrow_mut();
        if let Some(order_state) = orders.remove(&order_id.to_string()) {
            match order_state {
                OrderState::Locked(mut locked_order) => {
                    locked_order.removed = true;
                    orders.insert(order_id.to_string(), OrderState::Locked(locked_order));
                    Ok("Order removed successfully".to_string())
                }
                _ => Err("Order is not in a removable state".to_string()),
            }
        } else {
            Err("Order not found".to_string())
        }
    })
}
