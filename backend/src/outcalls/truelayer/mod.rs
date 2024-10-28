pub mod auth;
pub mod payments;
pub mod providers;

#[derive(serde::Serialize, serde::Deserialize, Debug, candid::CandidType)]
pub(super) struct ErrorResponse {
    pub error: String,
}
