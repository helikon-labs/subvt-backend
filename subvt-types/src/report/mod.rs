use crate::substrate::Era;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ValidatorEraReport {
    pub era: Era,
    pub is_active: bool,
    pub commission_per_billion: u32,
    pub self_stake: u128,
    pub total_stake: u128,
    pub block_count: u32,
    pub reward_points: u128,
    pub self_reward: u128,
    pub staker_reward: u128,
    pub offline_offence_count: u16,
    pub slashed_amount: u128,
    pub chilling_count: u16,
}
