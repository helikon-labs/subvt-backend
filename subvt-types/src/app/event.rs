use crate::crypto::AccountId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ValidatorOfflineEvent {
    pub id: u32,
    pub block_hash: String,
    pub event_index: Option<u32>,
    pub validator_account_id: AccountId,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChilledEvent {
    pub id: u32,
    pub block_hash: String,
    pub extrinsic_index: Option<u32>,
    pub event_index: u32,
    pub stash_account_id: AccountId,
}
