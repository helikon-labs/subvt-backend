pub struct ValidatorInfo {
    pub discovered_at: Option<u64>,
    pub killed_at: Option<u64>,
    pub slash_count: u64,
    pub offline_offence_count: u64,
    pub active_era_count: u64,
    pub inactive_era_count: u64,
    pub total_reward_points: u64,
    pub unclaimed_era_indices: Vec<u32>,
}
