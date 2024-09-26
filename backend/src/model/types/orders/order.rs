use std::collections::HashMap;

use candid::{CandidType, Deserialize};

use super::locked_order::{LockedOrder, Onramper, RevolutConsent};
use crate::{
    errors::{OrderError, Result, SystemError},
    model::{
        memory::heap,
        types::{common::AddressType, Crypto, PaymentProviderType},
    },
    types::{Blockchain, PaymentProvider, TransactionAddress},
};

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Order {
    pub id: u64,
    pub created_at: u64,
    pub currency: String,
    pub offramper_user_id: u64,
    pub offramper_address: TransactionAddress,
    pub offramper_providers: HashMap<PaymentProviderType, PaymentProvider>,
    pub crypto: Crypto,
    pub processing: bool,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct EvmOrderInput {
    pub tx_hash: String,
    pub estimated_gas_lock: u64,
    pub estimated_gas_withdraw: u64,
}

impl Order {
    pub fn new(
        currency: String,
        offramper_user_id: u64,
        offramper_address: TransactionAddress,
        offramper_providers: HashMap<PaymentProviderType, PaymentProvider>,
        blockchain: Blockchain,
        token: Option<String>,
        crypto_amount: u128,
        crypto_fee: u128,
    ) -> Result<Self> {
        offramper_address.validate()?;

        match (blockchain.clone(), &offramper_address.address_type) {
            (Blockchain::EVM { .. }, AddressType::EVM)
            | (Blockchain::ICP { .. }, AddressType::ICP)
            | (Blockchain::Solana, AddressType::Solana) => (),
            _ => {
                return Err(SystemError::InvalidInput(
                    "Address type does not match blockchain type".to_string(),
                )
                .into());
            }
        }

        let order_id = heap::generate_order_id();
        let order = Order {
            id: order_id.clone(),
            currency,
            created_at: ic_cdk::api::time(),
            offramper_user_id,
            offramper_address,
            offramper_providers,
            crypto: Crypto::new(blockchain, token, crypto_amount, crypto_fee),
            processing: false,
        };
        ic_cdk::println!("[new order] order = {:?}", order);

        Ok(order)
    }

    pub fn is_processing(&self) -> Result<()> {
        if !self.processing {
            return Err(OrderError::OrderNotProcessing.into());
        }
        Ok(())
    }

    fn processable(&self) -> Result<()> {
        if self.processing {
            return Err(OrderError::OrderProcessing.into());
        }
        Ok(())
    }

    pub fn set_processing(&mut self) -> Result<()> {
        self.processable()?;
        self.processing = true;
        Ok(())
    }

    pub fn unset_processing(&mut self) {
        self.processing = false;
    }

    pub fn lock(
        self,
        price: u64,
        offramper_fee: u64,
        onramper_user_id: u64,
        onramper_provider: PaymentProvider,
        onramper_address: TransactionAddress,
        revolut_consent: Option<RevolutConsent>,
    ) -> Result<LockedOrder> {
        // Check if the address type matches the blockchain type
        match (
            self.crypto.blockchain.clone(),
            &onramper_address.address_type,
        ) {
            (Blockchain::EVM { .. }, AddressType::EVM)
            | (Blockchain::ICP { .. }, AddressType::ICP)
            | (Blockchain::Solana, AddressType::Solana) => (),
            _ => {
                return Err(SystemError::InvalidInput(
                    "Address type does not match blockchain type".to_string(),
                )
                .into());
            }
        }

        let mut base_order = self.clone();
        base_order.unset_processing();

        Ok(LockedOrder {
            base: base_order,
            locked_at: ic_cdk::api::time(),
            price,
            offramper_fee,
            onramper: Onramper::new(onramper_user_id, onramper_provider, onramper_address),
            revolut_consent,
            payment_done: false,
            payment_id: None,
            uncommited: false,
        })
    }
}
