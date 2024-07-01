use std::collections::HashSet;

use crate::{
    errors::{RampError, Result},
    management::user as user_management,
    state::storage::{self, Order, OrderFilter, OrderState, OrderStateFilter, PaymentProvider},
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

    storage::insert_order(&order);
    Ok(order.id)
}

pub fn get_orders(filter: Option<OrderFilter>) -> Vec<OrderState> {
    match filter {
        None => storage::ORDERS.with(|p| p.borrow().iter().map(|(_, v)| v.clone()).collect()),
        Some(OrderFilter::ByOfframperAddress(address)) => {
            storage::filter_orders(|order_state| match order_state {
                OrderState::Created(order) => order.offramper_address == address,
                OrderState::Locked(order) => order.base.offramper_address == address,
                _ => false,
            })
        }
        Some(OrderFilter::LockedByOnramper(address)) => {
            storage::filter_orders(|order_state| match order_state {
                OrderState::Locked(order) => order.onramper_address == address,
                _ => false,
            })
        }
        Some(OrderFilter::ByState(state)) => {
            storage::filter_orders(|order_state| match (state.clone(), order_state) {
                (OrderStateFilter::Created, OrderState::Created(_))
                | (OrderStateFilter::Locked, OrderState::Locked(_))
                | (OrderStateFilter::Completed, OrderState::Completed(_))
                | (OrderStateFilter::Cancelled, OrderState::Cancelled(_)) => true,
                _ => false,
            })
        }
        Some(OrderFilter::ByChainId(chain_id)) => {
            storage::filter_orders(|order_state| match order_state {
                OrderState::Created(order) => order.chain_id == chain_id,
                OrderState::Locked(order) => order.base.chain_id == chain_id,
                _ => false,
            })
        }
    }
}

pub fn update_order_state(order_id: u64, new_state: OrderState) -> Result<()> {
    storage::mutate_order(&order_id, |order_state| *order_state = new_state)
}

pub fn lock_order(
    order_id: u64,
    onramper_provider: PaymentProvider,
    onramper_address: String,
) -> Result<()> {
    storage::mutate_order(&order_id, |order_state| match order_state {
        OrderState::Created(order) => {
            *order_state =
                OrderState::Locked(order.clone().lock(onramper_provider, onramper_address));
            Ok(())
        }
        _ => return Err(RampError::InvalidOrderState(order_state.to_string())),
    })?
}

pub fn unlock_order(order_id: u64) -> Result<()> {
    storage::mutate_order(&order_id, |order_state| match order_state {
        OrderState::Locked(order) => {
            let score = user_management::decrease_user_score(&order.onramper_address)?;
            ic_cdk::println!("[unlock_order] user score decreased = {:?}", score);
            *order_state = OrderState::Created(order.base.clone());
            Ok(())
        }
        _ => Err(RampError::InvalidOrderState(order_state.to_string())),
    })?
}

pub fn mark_order_as_paid(order_id: u64) -> Result<()> {
    storage::mutate_order(&order_id, |order_state| match order_state {
        OrderState::Locked(order) => {
            let score = user_management::update_onramper_payment(
                &order.onramper_address,
                order.base.fiat_amount,
            )?;
            ic_cdk::println!("[mark_order_as_paid] user score increased = {:?}", score);
            user_management::update_offramper_payment(
                &order.base.offramper_address,
                order.base.fiat_amount,
            )?;
            order.payment_done = true;
            Ok(())
        }
        _ => Err(RampError::InvalidOrderState(order_state.to_string())),
    })?
}

pub fn cancel_order(order_id: u64) -> Result<()> {
    storage::mutate_order(&order_id, |order_state| match order_state {
        OrderState::Created(_) => {
            *order_state = OrderState::Cancelled(order_id);
            Ok(())
        }
        _ => Err(RampError::InvalidOrderState(order_state.to_string())),
    })?
}
