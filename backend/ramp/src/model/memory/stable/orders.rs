use crate::errors::{OrderError, Result};
use crate::model::memory::heap::{clear_order_timer, set_order_timer};
use crate::types::{
    orders::{Order, OrderState, RevolutConsent},
    PaymentProvider, TransactionAddress,
};

use super::storage::ORDERS;

pub fn insert_order(order: &Order) -> Option<OrderState> {
    ORDERS.with_borrow_mut(|p| p.insert(order.id, OrderState::Created(order.clone())))
}

pub fn get_order(order_id: &u64) -> Result<OrderState> {
    ORDERS
        .with_borrow(|orders| orders.get(order_id))
        .ok_or_else(|| OrderError::OrderNotFound.into())
}

pub fn filter_orders<F>(filter: F, page: Option<u32>, page_size: Option<u32>) -> Vec<OrderState>
where
    F: Fn(&OrderState) -> bool,
{
    let start_index = page.unwrap_or(1).saturating_sub(1) * page_size.unwrap_or(10);
    let end_index = start_index + page_size.unwrap_or(10);

    ORDERS.with_borrow(|orders| {
        orders
            .iter()
            .filter_map(|(_, order_state)| {
                if filter(&order_state) {
                    Some(order_state.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .skip(start_index as usize)
            .take((end_index - start_index) as usize)
            .collect()
    })
}

pub fn mutate_order<F, R>(order_id: &u64, f: F) -> Result<R>
where
    F: FnOnce(&mut OrderState) -> R,
{
    ORDERS.with_borrow_mut(|orders| {
        if let Some(mut order_state) = orders.get(order_id) {
            let result = f(&mut order_state);
            orders.insert(*order_id, order_state);
            Ok(result)
        } else {
            Err(OrderError::OrderNotFound)?
        }
    })
}

pub fn lock_order(
    order_id: u64,
    price: u64,
    offramper_fee: u64,
    onramper_user_id: u64,
    onramper_provider: PaymentProvider,
    onramper_address: TransactionAddress,
    revolut_consent: Option<RevolutConsent>,
) -> Result<()> {
    mutate_order(&order_id, |order_state| -> Result<()> {
        match order_state {
            OrderState::Created(order) => {
                *order_state = OrderState::Locked(order.clone().lock(
                    price,
                    offramper_fee,
                    onramper_user_id,
                    onramper_provider,
                    onramper_address,
                    revolut_consent,
                )?);
                Ok(())
            }
            _ => Err(OrderError::InvalidOrderState(order_state.to_string()))?,
        }
    })??;

    set_order_timer(order_id);
    Ok(())
}

pub fn unlock_order(order_id: u64) -> Result<()> {
    mutate_order(&order_id, |order_state| match order_state {
        OrderState::Locked(order) => {
            order.uncommit();
            Ok(())
        }
        _ => Err(OrderError::InvalidOrderState(order_state.to_string())),
    })??;

    mutate_order(&order_id, |order_state| -> Result<()> {
        match order_state {
            OrderState::Locked(order) => {
                super::users::mutate_user(order.onramper.user_id, |user| {
                    user.decrease_score();
                })?;
                ic_cdk::println!(
                    "[unlock_order] score decreased for user #{:?}",
                    order.onramper.user_id
                );

                let mut base_order = order.base.clone();
                base_order.unset_processing();

                *order_state = OrderState::Created(base_order);
                Ok(())
            }
            _ => Err(OrderError::InvalidOrderState(order_state.to_string()))?,
        }
    })??;

    clear_order_timer(order_id)
}

pub fn cancel_order(order_id: u64) -> Result<()> {
    mutate_order(&order_id, |order_state| -> Result<()> {
        match order_state {
            OrderState::Created(_) => {
                *order_state = OrderState::Cancelled(order_id);
                Ok(())
            }
            _ => Err(OrderError::InvalidOrderState(order_state.to_string()))?,
        }
    })?
}

pub fn set_processing_order(order_id: &u64) -> Result<()> {
    mutate_order(order_id, |order_state| match order_state {
        OrderState::Created(order) => order.set_processing(),
        OrderState::Locked(order) => order.base.set_processing(),
        _ => Err(OrderError::InvalidOrderState(order_state.to_string()))?,
    })?
}

pub fn unset_processing_order(order_id: &u64) -> Result<()> {
    mutate_order(order_id, |order_state| match order_state {
        OrderState::Created(order) => {
            order.unset_processing();
            Ok(())
        }
        OrderState::Locked(order) => {
            order.base.unset_processing();
            Ok(())
        }
        _ => Err(OrderError::InvalidOrderState(order_state.to_string()))?,
    })?
}
