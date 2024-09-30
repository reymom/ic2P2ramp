use candid::{CandidType, Deserialize};

use crate::{
    model::memory::heap,
    types::{Blockchain, PaymentProvider, TransactionAddress},
};

use super::order::Order;

pub struct LockInput {
    pub price: u64,
    pub offramper_fee: u64,
    pub onramper_user_id: u64,
    pub onramper_provider: PaymentProvider,
    pub onramper_address: TransactionAddress,
    pub revolut_consent: Option<RevolutConsent>,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct Onramper {
    pub user_id: u64,
    pub provider: PaymentProvider,
    pub address: TransactionAddress,
}

impl Onramper {
    pub fn new(user_id: u64, provider: PaymentProvider, address: TransactionAddress) -> Self {
        Onramper {
            user_id,
            provider,
            address,
        }
    }
}

#[derive(CandidType, Deserialize, Clone)]
pub struct RevolutConsent {
    pub id: String,
    url: String,
}

impl RevolutConsent {
    pub fn new(id: String, url: String) -> Self {
        RevolutConsent { id, url }
    }
}

#[derive(CandidType, Deserialize, Clone)]
pub struct LockedOrder {
    pub base: Order,
    pub locked_at: u64,
    pub price: u64,
    pub offramper_fee: u64,
    pub onramper: Onramper,
    pub revolut_consent: Option<RevolutConsent>,
    pub payment_id: Option<String>,
    pub payment_done: bool,
    pub uncommited: bool,
}

impl LockedOrder {
    pub fn complete(self) -> CompletedOrder {
        self.into()
    }

    pub fn uncommit(&mut self) {
        self.uncommited = true;
    }

    pub fn payment_amount_matches(&self, received_amount: &str) -> bool {
        let total_expected_amount = (self.price + self.offramper_fee) as f64 / 100.0;
        let received_amount_as_float = received_amount.parse::<f64>().unwrap_or(0.0);

        (received_amount_as_float - total_expected_amount).abs() < f64::EPSILON
    }

    pub fn is_inside_lock_time(&self) -> bool {
        self.locked_at + heap::LOCK_DURATION_TIME_SECONDS * 1_000_000_000 > ic_cdk::api::time()
    }
}

#[derive(CandidType, Deserialize, Clone)]
pub struct CompletedOrder {
    pub onramper: TransactionAddress,
    pub offramper: TransactionAddress,
    pub price: u64,
    pub offramper_fee: u64,
    pub blockchain: Blockchain,
    pub completed_at: u64,
}

impl From<LockedOrder> for CompletedOrder {
    fn from(locked_order: LockedOrder) -> Self {
        let base = locked_order.base;
        CompletedOrder {
            onramper: locked_order.onramper.address,
            offramper: base.offramper_address,
            price: locked_order.price,
            offramper_fee: locked_order.offramper_fee,
            blockchain: base.crypto.blockchain,
            completed_at: ic_cdk::api::time(),
        }
    }
}
