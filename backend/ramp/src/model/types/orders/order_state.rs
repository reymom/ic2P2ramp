use std::{borrow::Cow, fmt};

use candid::{CandidType, Decode, Deserialize, Encode};
use ic_stable_structures::{storable::Bound, Storable};

use crate::errors::{OrderError, Result};

use super::{CompletedOrder, LockedOrder, Order};

const MAX_ORDER_SIZE: u32 = 8000;

pub type OrderId = u64;

#[derive(CandidType, Deserialize, Clone)]
pub enum OrderState {
    Created(Order),
    Locked(LockedOrder),
    Completed(CompletedOrder),
    Cancelled(u64),
}

impl OrderState {
    pub fn created(&self) -> Result<Order> {
        match self {
            OrderState::Created(order) => Ok(order.clone()),
            _ => Err(OrderError::InvalidOrderState(self.to_string()).into()),
        }
    }

    pub fn created_mut(&mut self) -> Result<&mut Order> {
        match self {
            OrderState::Created(order) => Ok(order),
            _ => Err(OrderError::InvalidOrderState(self.to_string()).into()),
        }
    }

    pub fn locked(&self) -> Result<LockedOrder> {
        match self {
            OrderState::Locked(order) => Ok(order.clone()),
            _ => Err(OrderError::InvalidOrderState(self.to_string()).into()),
        }
    }
}

impl fmt::Display for OrderState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrderState::Created(_) => write!(f, "Created"),
            OrderState::Locked(_) => write!(f, "Locked"),
            OrderState::Completed(_) => write!(f, "Completed"),
            OrderState::Cancelled(_) => write!(f, "Cancelled"),
        }
    }
}

impl Storable for OrderState {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: MAX_ORDER_SIZE,
        is_fixed_size: false,
    };
}
