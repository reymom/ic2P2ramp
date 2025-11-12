use candid::{CandidType, Deserialize};
use icramp_types::bitcoin::runes::RuneUTXOEntry;

use super::locked_order::{LockedOrder, Onramper, RevolutConsent};
use crate::{
    errors::{OrderError, Result, SystemError},
    model::memory::heap,
    types::{
        BlockchainAsset, Crypto, PaymentProvider, TransactionAddress, common::AddressType,
        orders::validators::validate_offramper_providers_for_order,
    },
};

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Order {
    pub id: u64,
    pub created_at: u64,
    pub currency: String,
    pub offramper_user_id: u64,
    pub offramper_address: TransactionAddress,
    pub offramper_providers: Vec<PaymentProvider>,
    pub crypto: Crypto,
    pub fills: Vec<FillRecord>,
    pub processing: bool,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct FillRecord {
    pub payer_user_id: u64,
    pub payer: TransactionAddress,
    pub provider: PaymentProvider,
    pub fiat: u64, // price
    pub offramper_fee: u64,
    pub crypto_amount: u128,
    pub crypto_fee: u128,
    pub payment_id: String, // PayPal capture id / Stripe session id
    pub tx_id: Option<String>,
    pub created_at: u64,
}

#[derive(CandidType, Deserialize, Clone)]
pub enum DepositInput {
    Evm(EvmOrderInput),
    Bitcoin(BitcoinOrderInput),
    Solana(SolanaOrderInput),
}

#[derive(CandidType, Deserialize, Clone)]
pub struct EvmOrderInput {
    pub tx_hash: String,
    pub estimated_gas_lock: u64,
    pub estimated_gas_withdraw: u64,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct BitcoinOrderInput {
    pub tx_id: String,
    pub canister_address: String,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct SolanaOrderInput {
    pub signature: String,
    pub mint: Option<String>,
}

impl Order {
    pub fn new(
        currency: String,
        offramper_user_id: u64,
        offramper_address: TransactionAddress,
        offramper_providers: Vec<PaymentProvider>,
        asset: BlockchainAsset,
        crypto_amount: u128,
        crypto_fee: u128,
        rune_utxos: Option<Vec<RuneUTXOEntry>>,
    ) -> Result<Self> {
        offramper_address.validate()?;
        validate_offramper_providers_for_order(&offramper_providers)?;

        match (asset.clone(), &offramper_address.address_type) {
            (BlockchainAsset::EVM { .. }, AddressType::EVM)
            | (BlockchainAsset::ICP { .. }, AddressType::ICP)
            | (BlockchainAsset::Bitcoin { .. }, AddressType::Bitcoin)
            | (BlockchainAsset::Solana { .. }, AddressType::Solana) => (),
            _ => {
                return Err(SystemError::InvalidInput(
                    "Address type does not match blockchain type".to_string(),
                )
                .into());
            }
        }

        let order_id = heap::generate_order_id();
        let order = Order {
            id: order_id,
            currency,
            created_at: ic_cdk::api::time(),
            offramper_user_id,
            offramper_address,
            offramper_providers,
            crypto: Crypto::new(asset, crypto_amount, crypto_fee, rune_utxos)?,
            fills: vec![],
            processing: false,
        };
        ic_cdk::println!("[new order] order = {:?}", order);

        Ok(order)
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
        lock_amount: u128,
        price: u64,
        offramper_fee: u64,
        onramper_user_id: u64,
        onramper_provider: PaymentProvider,
        onramper_address: TransactionAddress,
        revolut_consent: Option<RevolutConsent>,
        stripe_session: Option<(String, String)>,
    ) -> Result<LockedOrder> {
        // Check if the address type matches the blockchain type
        match (self.crypto.asset.clone(), &onramper_address.address_type) {
            (BlockchainAsset::EVM { .. }, AddressType::EVM)
            | (BlockchainAsset::ICP { .. }, AddressType::ICP)
            | (BlockchainAsset::Bitcoin { .. }, AddressType::Bitcoin)
            | (BlockchainAsset::Solana { .. }, AddressType::Solana) => (),
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
            lock_amount,
            locked_at: ic_cdk::api::time(),
            price,
            offramper_fee,
            onramper: Onramper::new(onramper_user_id, onramper_provider, onramper_address),
            revolut_consent,
            payment_done: false,
            payment_id: stripe_session.as_ref().map(|(id, _)| id.clone()),
            payment_url: stripe_session.as_ref().map(|(_, url)| url.clone()),
            uncommited: false,
            pending_fill: None,
        })
    }
}
