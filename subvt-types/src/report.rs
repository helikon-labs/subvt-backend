//! Report presentation types. Utilized by the `subvt-report-service` crate to server era and
//! validator reports.
use crate::crypto::AccountId;
use crate::substrate::{Epoch, Era};
use crate::subvt::{ValidatorDetails, ValidatorSummary};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct EraValidatorReport {
    pub era: Era,
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

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct EraReport {
    pub era: Era,
    pub minimum_stake: Option<u128>,
    pub maximum_stake: Option<u128>,
    pub average_stake: Option<u128>,
    pub median_stake: Option<u128>,
    pub total_validator_reward: Option<u128>,
    pub total_reward_points: Option<u128>,
    pub total_reward: u128,
    pub total_stake: Option<u128>,
    pub active_validator_count: u32,
    pub inactive_validator_count: u32,
    pub active_nominator_count: Option<u64>,
    pub offline_offence_count: u64,
    pub slashed_amount: u128,
    pub chilling_count: u64,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct EraValidator {
    pub era_index: u64,
    pub validator_account_id: AccountId,
    pub is_active: bool,
    pub active_validator_index: Option<u64>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SessionParaValidator {
    pub session_index: u64,
    pub validator_account_id: AccountId,
    pub para_validator_group_index: u64,
    pub para_validator_index: u64,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct BlockSummary {
    pub number: u64,
    pub hash: String,
    pub timestamp: u64,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ParaVotesSummary {
    pub explicit: u32,
    pub implicit: u32,
    pub missed: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ParaVoteType {
    #[serde(rename = "explicit")]
    EXPLICIT,
    #[serde(rename = "implicit")]
    IMPLICIT,
    #[serde(rename = "missed")]
    MISSED,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ParaVote {
    pub block_number: u64,
    pub block_hash: String,
    #[serde(skip_serializing)]
    pub session_index: u64,
    pub para_id: u64,
    #[serde(skip_serializing)]
    pub para_validator_index: u64,
    pub vote: ParaVoteType,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HeartbeatEvent {
    pub block: BlockSummary,
    pub event_index: u32,
    pub im_online_key: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SessionValidatorReport {
    pub session: Epoch,
    pub is_active: bool,
    pub validator_index: Option<u64>,
    pub heartbeat_event: Option<HeartbeatEvent>,
    pub blocks_authored: Option<Vec<BlockSummary>>,
    pub para_validator_group_index: Option<u64>,
    pub para_validator_index: Option<u64>,
    pub para_votes_summary: Option<ParaVotesSummary>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SessionValidatorParaVoteReport {
    pub session: Epoch,
    pub para_validator_group_index: Option<u64>,
    pub para_validator_index: Option<u64>,
    pub para_votes_summary: Option<ParaVotesSummary>,
    pub para_votes: Option<Vec<ParaVote>>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SessionParaVoteReport {
    pub para_id: u64,
    pub para_votes_summary: ParaVotesSummary,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SessionParasVoteReport {
    pub session: Epoch,
    pub paras: Vec<SessionParaVoteReport>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ValidatorSummaryReport {
    pub finalized_block: BlockSummary,
    pub validator_summary: ValidatorSummary,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ValidatorDetailsReport {
    pub finalized_block: BlockSummary,
    pub validator_details: ValidatorDetails,
}
