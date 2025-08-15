use super::storage::PROCESSED_TX_HASHES;

const TXS_THRESHOLD_DISCARD: u64 = 30 * 7 * 24 * 3600;

pub fn is_tx_hash_processed(tx_hash: &String) -> bool {
    PROCESSED_TX_HASHES.with_borrow(|hashes| hashes.contains_key(tx_hash))
}

pub fn mark_tx_hash_as_processed(tx_hash: String) {
    PROCESSED_TX_HASHES.with_borrow_mut(|hashes| {
        hashes.insert(tx_hash, ic_cdk::api::time() / 1_000_000_000);
    });
}

pub fn discard_old_transactions() {
    PROCESSED_TX_HASHES.with_borrow_mut(|hashes| {
        let keys_to_remove: Vec<String> = hashes
            .iter()
            .filter_map(|(tx_hash, timestamp)| {
                // Check if the transaction is too old
                if timestamp < ic_cdk::api::time() / 1_000_000_000 - TXS_THRESHOLD_DISCARD {
                    Some(tx_hash.clone()) // Collect the tx_hash for removal
                } else {
                    None
                }
            })
            .collect();

        for tx_hash in keys_to_remove {
            hashes.remove(&tx_hash);
        }
    });
}
