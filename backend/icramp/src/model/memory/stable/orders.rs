use crate::errors::{OrderError, Result};
use crate::model::memory::heap::{clear_order_timer, set_order_timer};
use crate::types::{
    PaymentProvider, TransactionAddress,
    orders::{FillRecord, Order, OrderState, RevolutConsent},
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
            .filter_map(|e| {
                let order_state = e.value();
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
    lock_amount: u128,
    price: u64,
    offramper_fee: u64,
    onramper_user_id: u64,
    onramper_provider: PaymentProvider,
    onramper_address: TransactionAddress,
    revolut_consent: Option<RevolutConsent>,
    stripe_session: Option<(String, String)>,
) -> Result<()> {
    mutate_order(&order_id, |order_state| -> Result<()> {
        match order_state {
            OrderState::Created(order) => {
                *order_state = OrderState::Locked(order.clone().lock(
                    lock_amount,
                    price,
                    offramper_fee,
                    onramper_user_id,
                    onramper_provider,
                    onramper_address,
                    revolut_consent,
                    stripe_session,
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

pub fn set_payment_id(order_id: u64, payment_id: String) -> Result<()> {
    mutate_order(&order_id, |order_state| match order_state {
        OrderState::Locked(order) => {
            order.payment_id = Some(payment_id);
            Ok(())
        }
        _ => Err(OrderError::InvalidOrderState(order_state.to_string()))?,
    })?
}

pub fn append_fill_if_new(order_id: u64, fill: FillRecord) -> Result<()> {
    mutate_order(&order_id, |state| -> Result<()> {
        let OrderState::Locked(lo) = state else {
            return Err(OrderError::InvalidOrderState(state.to_string()).into());
        };
        // idempotent: avoid double append
        if !lo
            .base
            .fills
            .iter()
            .any(|f| f.payment_id == fill.payment_id)
        {
            let mut base = lo.base.clone();
            base.fills.push(fill);
            // write back into locked
            let mut locked = lo.clone();
            locked.base = base;
            *state = OrderState::Locked(locked);
        }
        Ok(())
    })?
}

pub fn set_order_completed(order_id: u64) -> Result<()> {
    mutate_order(&order_id, |order_state| match order_state {
        OrderState::Locked(order) => {
            let total = order.base.crypto.amount;
            let filled = order.lock_amount;

            if filled > total {
                return Err(OrderError::InvalidInput("locked > available".into()));
            }
            order.base.unset_processing();
            if filled == total {
                *order_state = OrderState::Completed(order.clone().complete());
            } else {
                let remaining = total - filled;
                let mut base = order.base.clone();
                base.crypto.amount = remaining;
                *order_state = OrderState::Created(base);
            }

            Ok(())
        }
        _ => Err(OrderError::InvalidOrderState(order_state.to_string()))?,
    })??;

    clear_order_timer(order_id)
}

pub fn set_pending_fill(order_id: u64, fill: FillRecord) -> Result<()> {
    mutate_order(&order_id, |s| -> Result<()> {
        if let OrderState::Locked(mut lo) = s.clone() {
            lo.pending_fill = Some(fill);
            *s = OrderState::Locked(lo);
            Ok(())
        } else {
            Err(OrderError::InvalidOrderState(s.to_string()).into())
        }
    })?
}

pub fn finalize_pending_fill(order_id: u64, tx_id: Option<String>) -> Result<()> {
    mutate_order(&order_id, |s| -> Result<()> {
        let OrderState::Locked(mut lo) = s.clone() else {
            return Err(OrderError::InvalidOrderState(s.to_string()).into());
        };

        // Helper to try updating an in-place fill already in base.fills (tx_id == None)
        let mut updated_existing = false;
        if let Some(ref tid) = tx_id {
            if let Some(idx) = lo
                .base
                .fills
                .iter()
                .rposition(|f| f.tx_id.is_none() &&
                               // strongest keys first; fall back to amount-only if needed
                               ( !f.payment_id.is_empty() && f.payment_id == lo.pending_fill.as_ref().map(|pf| pf.payment_id.clone()).unwrap_or_default()
                                 || f.crypto_amount == lo.pending_fill.as_ref().map(|pf| pf.crypto_amount).unwrap_or(0)
                               )
                )
            {
                let f = &mut lo.base.fills[idx];
                // If already finalized by another path, don't overwrite
                if f.tx_id.is_none() {
                    f.tx_id = Some(tid.clone());
                    updated_existing = true;
                }
            }
        }

        if updated_existing {
            *s = OrderState::Locked(lo);
            return Ok(());
        }

        // Otherwise, consume pending_fill if any, attach tx_id, and push (without duplication)
        if let Some(mut fill) = lo.pending_fill.take() {
            if let Some(tid) = tx_id {
                fill.tx_id = Some(tid);
            }

            // Avoid duplicates: if an equivalent fill (same payer + amount + payment_id) exists, just set tx_id there
            if let Some(idx) = lo.base.fills.iter().rposition(|f| {
                f.payer_user_id == fill.payer_user_id
                    && f.crypto_amount == fill.crypto_amount
                    && (f.payment_id == fill.payment_id
                        || f.payment_id.is_empty()
                        || fill.payment_id.is_empty())
            }) {
                if lo.base.fills[idx].tx_id.is_none() {
                    lo.base.fills[idx].tx_id = fill.tx_id.clone();
                }
            } else {
                lo.base.fills.push(fill);
            }

            *s = OrderState::Locked(lo);
            return Ok(());
        }

        // Nothing to finalize: safe no-op (idempotent)
        *s = OrderState::Locked(lo);
        Ok(())
    })?
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
