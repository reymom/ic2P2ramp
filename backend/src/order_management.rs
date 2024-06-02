use crate::storage;

pub async fn create_order(
    fiat_amount: u64,
    crypto_amount: u64,
    offramper_paypal_id: String,
    offramper_address: String,
    chain_id: u64,
    token_type: String,
) -> Result<String, String> {
    let order_id = storage::generate_order_id();
    let order = storage::Order {
        id: order_id.clone(),
        originator: ic_cdk::caller(),
        fiat_amount,
        crypto_amount,
        offramper_paypal_id,
        onramper_paypal_id: None,
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

pub async fn get_order_by_id(order_id: String) -> Result<storage::Order, String> {
    storage::ORDERS.with(|orders| {
        let orders = orders.borrow();
        if let Some(order) = orders.get(&order_id) {
            Ok(order.clone())
        } else {
            Err("Order not found".to_string())
        }
    })
}

pub async fn lock_order(
    order_id: String,
    onramper_paypal_id: String,
    onramper_address: String,
) -> Result<String, String> {
    storage::ORDERS.with(|orders| {
        let mut orders = orders.borrow_mut();
        if let Some(mut order) = orders.remove(&order_id) {
            if order.locked {
                return Err("Order is already locked".to_string());
            }
            order.locked = true;
            order.onramper_paypal_id = Some(onramper_paypal_id);
            order.onramper_address = Some(onramper_address);
            orders.insert(order_id.clone(), order);
            Ok("Order locked".to_string())
        } else {
            Err("Order not found".to_string())
        }
    })
}

pub async fn mark_order_as_paid(order_id: String) -> Result<(), String> {
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

pub async fn remove_order(order_id: String) -> Result<String, String> {
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
