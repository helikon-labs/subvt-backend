//! Report presentation types. Utilized by the `subvt-report-service` crate to server era and
//! validator reports.
use crate::crypto::AccountId;
use crate::substrate::{Account, Balance, Epoch, Era, Stake};
use crate::subvt::{ValidatorDetails, ValidatorSummary};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct EraValidatorListReport {
    pub era: Era,
    pub validators: Vec<EraValidatorReport>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct EraValidatorReport {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<AccountId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub era: Option<Era>,
    pub is_active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commission_per_billion: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub self_stake: Option<u128>,
    #[serde(skip_serializing_if = "Option::is_none")]
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
    pub total_reward: Option<u128>,
    pub total_reward_points: Option<u128>,
    pub total_paid_out: u128,
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

#[derive(Eq, PartialEq, Hash, Clone, Debug, Default, Deserialize, Serialize)]
pub struct ParaVotesSummary {
    pub explicit: u32,
    pub implicit: u32,
    pub missed: u32,
}

impl ParaVotesSummary {
    pub fn from_para_votes(para_votes: &[ParaVote]) -> ParaVotesSummary {
        let mut para_votes_summary = ParaVotesSummary::default();
        for vote in para_votes {
            match vote.vote {
                ParaVoteType::EXPLICIT => para_votes_summary.explicit += 1,
                ParaVoteType::IMPLICIT => para_votes_summary.implicit += 1,
                ParaVoteType::MISSED => para_votes_summary.missed += 1,
            }
        }
        para_votes_summary
    }
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

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ValidatorListReport {
    pub finalized_block: BlockSummary,
    pub validators: Vec<ValidatorSummary>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct EraValidatorRewardReport {
    pub era: Era,
    pub reward: Balance,
}

impl From<&(Era, Balance)> for EraValidatorRewardReport {
    fn from(era_reward: &(Era, Balance)) -> Self {
        Self {
            era: era_reward.0.clone(),
            reward: era_reward.1,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct EraValidatorPayoutReport {
    pub era: Era,
    pub payout: Balance,
}

impl From<&(Era, Balance)> for EraValidatorPayoutReport {
    fn from(era_reward: &(Era, Balance)) -> Self {
        Self {
            era: era_reward.0.clone(),
            payout: era_reward.1,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ValidatorTotalReward {
    pub validator_account_id: AccountId,
    pub total_reward: Balance,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ValidatorTotalRewardChartData {
    pub accounts: Vec<Account>,
    pub rewards: Vec<ValidatorTotalReward>,
    pub start_timestamp: u64,
    pub end_timestamp: u64,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Controller {
    pub controller_account_id: AccountId,
    pub controller_address: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Bond {
    pub controller_account_id: AccountId,
    pub controller_address: String,
    pub bond: Stake,
}

#[derive(Clone, Debug)]
pub struct Reward {
    pub id: u32,
    pub block_hash: String,
    pub block_number: u64,
    pub block_timestamp: u64,
    pub extrinsic_index: u32,
    pub event_index: u32,
    pub rewardee_account_id: AccountId,
    pub amount: Balance,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct MonthlyIncome {
    pub year: u32,
    pub month: u32,
    pub income: f64,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct MonthlyIncomeReport {
    pub rewardee: AccountId,
    pub token_symbol: String,
    pub monthly_income: Vec<MonthlyIncome>,
}
