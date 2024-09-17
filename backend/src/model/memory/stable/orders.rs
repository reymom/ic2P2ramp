use crate::errors::{RampError, Result};
use crate::model::memory::heap::clear_order_timer;
use crate::types::order::{Order, OrderState};

use super::storage::ORDERS;

pub fn insert_order(order: &Order) -> Option<OrderState> {
    ORDERS.with_borrow_mut(|p| p.insert(order.id.clone(), OrderState::Created(order.clone())))
}

pub fn get_order(order_id: &u64) -> Result<OrderState> {
    ORDERS
        .with_borrow(|orders| orders.get(order_id))
        .ok_or_else(|| RampError::OrderNotFound)
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
        if let Some(mut order_state) = orders.get(&order_id) {
            let result = f(&mut order_state);
            orders.insert(*order_id, order_state);
            Ok(result)
        } else {
            Err(RampError::OrderNotFound)
        }
    })
}

pub fn unlock_order(order_id: u64) -> Result<()> {
    mutate_order(&order_id, |order_state| match order_state {
        OrderState::Locked(order) => {
            super::users::mutate_user(order.onramper_user_id, |user| {
                user.decrease_score();
            })?;
            ic_cdk::println!(
                "[unlock_order] score decreased for user #{:?}",
                order.onramper_user_id
            );
            *order_state = OrderState::Created(order.base.clone());
            Ok(())
        }
        _ => Err(RampError::InvalidOrderState(order_state.to_string())),
    })??;

    clear_order_timer(order_id)
}
