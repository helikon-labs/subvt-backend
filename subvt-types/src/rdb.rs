//! Types used in relational database storage.
use serde::{Deserialize, Serialize};

pub struct ValidatorInfo {
    pub discovered_at: Option<u64>,
    pub killed_at: Option<u64>,
    pub slash_count: u64,
    pub offline_offence_count: u64,
    pub active_era_count: u64,
    pub inactive_era_count: u64,
    pub total_reward_points: u64,
    pub unclaimed_era_indices: Vec<u32>,
    pub blocks_authored: Option<u64>,
    pub reward_points: Option<u64>,
    pub heartbeat_received: Option<bool>,
    pub onekv_candidate_record_id: Option<u32>,
    pub onekv_rank: Option<u64>,
    pub onekv_location: Option<String>,
    pub onekv_is_valid: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BlockProcessedNotification {
    pub block_number: u64,
    pub block_hash: String,
}
