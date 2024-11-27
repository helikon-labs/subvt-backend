use crate::crypto::AccountId;

#[derive(Default)]
pub struct SessionValidatorPerformance {
    pub id: u64,
    pub validator_account_id: AccountId,
    pub era_index: u32,
    pub session_index: u64,
    pub active_validator_index: u64,
    pub authored_block_count: u32,
    pub para_validator_group_index: Option<u64>,
    pub para_validator_index: Option<u64>,
    pub implicit_attestation_count: Option<u32>,
    pub explicit_attestation_count: Option<u32>,
    pub missed_attestation_count: Option<u32>,
    pub attestations_per_billion: Option<u32>,
}
