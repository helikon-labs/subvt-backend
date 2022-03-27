use crate::crypto::AccountId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DemocracyCancelledEvent {
    pub id: u32,
    pub block_hash: String,
    pub extrinsic_index: Option<u32>,
    pub event_index: u32,
    pub referendum_index: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DemocracyDelegatedEvent {
    pub id: u32,
    pub block_hash: String,
    pub extrinsic_index: Option<u32>,
    pub event_index: u32,
    pub original_account_id: AccountId,
    pub delegate_account_id: AccountId,
}
