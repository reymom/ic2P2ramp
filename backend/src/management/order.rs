use crate::state::storage::{self, PaymentProvider};

pub fn create_order(
    fiat_amount: u64,
    currency_symbol: String,
    crypto_amount: u64,
    offramper_providers: Vec<PaymentProvider>,
    offramper_address: String,
    chain_id: u64,
    token_type: String,
) -> Result<String, String> {
    let order_id = storage::generate_order_id();
    let order = storage::Order {
        id: order_id.clone(),
        originator: ic_cdk::caller(),
        fiat_amount,
        currency_symbol,
        crypto_amount,
        offramper_providers,
        onramper_provider: None,
        offramper_address,
        onramper_address: None,
        locked: false,
        proof_submitted: false,
        chain_id,
        token_type,
        payment_done: false,
        removed: false,
    };

    storage::ORDERS.with(|p| p.borrow_mut().insert(order_id.clone(), order));
    Ok(order_id)
}

pub fn get_orders() -> Vec<storage::Order> {
    storage::ORDERS.with(|p| p.borrow().iter().map(|(_, v)| v.clone()).collect())
}

pub fn get_order_by_id(order_id: String) -> Result<storage::Order, String> {
    storage::ORDERS.with(|orders| {
        let orders = orders.borrow();
        if let Some(order) = orders.get(&order_id) {
            Ok(order.clone())
        } else {
            Err("Order not found".to_string())
        }
    })
}

pub fn lock_order(
    order_id: String,
    onramper_provider: PaymentProvider,
    onramper_address: String,
) -> Result<String, String> {
    storage::ORDERS.with(|orders| {
        let mut orders = orders.borrow_mut();
        if let Some(mut order) = orders.remove(&order_id) {
            if order.locked {
                return Err("order is already locked".to_string());
            }
            order.locked = true;
            order.onramper_provider = Some(onramper_provider);
            order.onramper_address = Some(onramper_address);
            orders.insert(order_id.clone(), order);
            Ok("order locked".to_string())
        } else {
            Err("order not found".to_string())
        }
    })
}

pub fn mark_order_as_paid(order_id: String) -> Result<(), String> {
    ic_cdk::println!("[mark_order_as_paid");
    storage::ORDERS.with(|orders| {
        let mut orders = orders.borrow_mut();
        if let Some(mut order) = orders.remove(&order_id) {
            order.payment_done = true;
            orders.insert(order_id.clone(), order);
            Ok(())
        } else {
            Err("Order not found".to_string())
        }
    })
}

pub fn remove_order(order_id: String) -> Result<String, String> {
    storage::ORDERS.with(|orders| {
        let mut orders = orders.borrow_mut();
        if let Some(mut order) = orders.remove(&order_id) {
            order.removed = true;
            orders.insert(order_id.clone(), order);
            Ok("Order removed successfully".to_string())
        } else {
            Err("Order not found".to_string())
        }
    })
}
