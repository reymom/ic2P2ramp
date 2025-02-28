use candid::{CandidType, Deserialize};
use serde::Serialize;

#[derive(Serialize, CandidType, Deserialize, Clone)]
pub struct Inscription {
    pub content: String,
    pub content_type: String,
    pub metadata: Option<String>,
}
