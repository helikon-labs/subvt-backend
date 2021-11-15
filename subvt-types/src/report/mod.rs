use crate::substrate::Era;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ReportError {
    pub description: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct EraValidatorReport {
    pub era: Option<Era>,
    pub is_active: Option<bool>,
    pub commission_per_billion: Option<u32>,
    pub self_stake: Option<u128>,
    pub total_stake: Option<u128>,
    pub block_count: u32,
    pub reward_points: Option<u128>,
    pub self_reward: u128,
    pub staker_reward: u128,
    pub offline_offence_count: u16,
    pub slashed_amount: u128,
    pub chilling_count: u16,
}

pub struct EraReport {}
