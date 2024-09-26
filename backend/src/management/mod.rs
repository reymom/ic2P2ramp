pub mod order;
pub mod payment;
pub mod random;
pub mod user;
pub mod vault;

pub fn on_fail_callback(order_id: u64) -> impl Fn() + 'static {
    move || match crate::memory::stable::orders::unset_processing_order(&order_id) {
        Ok(()) => ic_cdk::println!("Successfully unset order: {}", order_id),
        Err(e) => ic_cdk::println!("Error unsetting order: {}, error: {}", order_id, e),
    }
}
