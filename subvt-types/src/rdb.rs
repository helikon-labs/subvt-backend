//! Types used in relational database storage.
use serde::{Deserialize, Serialize};

pub struct ValidatorInfo {
    pub discovered_at: Option<u64>,
    pub slash_count: u64,
    pub offline_offence_count: u64,
    pub active_era_count: u64,
    pub inactive_era_count: u64,
    pub unclaimed_era_indices: Vec<u32>,
    pub blocks_authored: Option<u64>,
    pub reward_points: Option<u64>,
    pub heartbeat_received: Option<bool>,
    pub dn_record_id: Option<u32>,
    pub dn_status: Option<String>,
    pub performance: Vec<Vec<u32>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BlockProcessedNotification {
    pub block_number: u64,
    pub block_hash: String,
}
