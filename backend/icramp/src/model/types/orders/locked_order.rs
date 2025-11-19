use candid::{CandidType, Deserialize};

use super::order::Order;
use crate::{
    model::{memory::heap, types::orders::FillRecord},
    types::{BlockchainAsset, PaymentProvider, TransactionAddress},
};

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
    pub lock_amount: u128,
    pub locked_at: u64,
    pub price: u64,
    pub offramper_fee: u64,
    pub onramper: Onramper,
    pub revolut_consent: Option<RevolutConsent>,
    pub payment_id: Option<String>,
    pub payment_url: Option<String>, // for Stripe
    pub payment_done: bool,
    pub uncommited: bool,
    pub pending_fill: Option<FillRecord>,
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
    pub offramper: TransactionAddress,
    pub price: u64,
    pub currency: String,
    pub asset: BlockchainAsset,
    pub fills: Vec<FillRecord>,
    pub total_fiat: u64,
    pub total_offramper_fee: u64,
    pub total_crypto: u128,
    pub total_crypto_fee: u128,
    pub completed_at: u64,
}

impl From<LockedOrder> for CompletedOrder {
    fn from(locked_order: LockedOrder) -> Self {
        let base = locked_order.base;
        let total_offramper_fee = base.fills.iter().map(|f| f.offramper_fee).sum();
        let total_fiat: u64 = base.fills.iter().map(|f| f.fiat).sum();
        let total_crypto: u128 = base.fills.iter().map(|f| f.crypto_amount).sum();
        let total_crypto_fee: u128 = base.fills.iter().map(|f| f.crypto_fee).sum();
        CompletedOrder {
            offramper: base.offramper_address,
            price: locked_order.price,
            currency: base.currency,
            asset: base.crypto.asset,
            fills: base.fills,
            total_fiat,
            total_offramper_fee,
            total_crypto,
            total_crypto_fee,
            completed_at: ic_cdk::api::time(),
        }
    }
}
