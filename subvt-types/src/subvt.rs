//! SubVT types. These types are used to send the network status and
//! validator details/summary to Redis in-memory database. This data then gets consumed
//! by other services that require it.

use crate::crypto::AccountId;
use crate::substrate::para::ParaCoreAssignment;
use crate::substrate::{
    Account, Balance, Epoch, Era, InactiveNominationsSummary, NominationSummary, RewardDestination,
    Stake, StakeSummary, ValidatorPreferences, ValidatorStake,
};
use serde::{Deserialize, Serialize};
use std::convert::From;
use subvt_proc_macro::Diff;

/// Represents the network's status that changes with every block.
#[derive(Clone, Debug, Diff, Default, Deserialize, Serialize)]
pub struct NetworkStatus {
    pub finalized_block_number: u64,
    pub finalized_block_hash: String,
    pub best_block_number: u64,
    pub best_block_hash: String,
    pub active_era: Era,
    pub current_epoch: Epoch,
    pub active_validator_count: u32,
    pub inactive_validator_count: u32,
    pub last_era_total_reward: Balance,
    pub total_stake: Balance,
    pub return_rate_per_million: u32,
    pub min_stake: Balance,
    pub max_stake: Balance,
    pub average_stake: Balance,
    pub median_stake: Balance,
    pub era_reward_points: u32,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct NetworkStatusUpdate {
    pub network: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<NetworkStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff_base_block_number: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff: Option<NetworkStatusDiff>,
}

/// Represents an inactive validator, waiting to be in the active set.
#[derive(Clone, Debug, Default, Deserialize, Diff, Eq, Hash, PartialEq, Serialize)]
pub struct ValidatorDetails {
    #[diff_key]
    pub account: Account,
    pub network_id: u32,
    pub controller_account_id: AccountId,
    pub preferences: ValidatorPreferences,
    pub self_stake: Stake,
    pub reward_destination: RewardDestination,
    pub next_session_keys: String,
    pub queued_session_keys: Option<String>,
    pub is_active: bool,
    pub is_active_next_session: bool,
    pub nominations: Vec<NominationSummary>,
    pub oversubscribed: bool,
    pub active_era_count: u64,
    pub inactive_era_count: u64,
    pub slash_count: u64,
    pub offline_offence_count: u64,
    pub unclaimed_era_indices: Vec<u32>,
    pub is_para_validator: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub para_core_assignment: Option<ParaCoreAssignment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_rate_per_billion: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocks_authored: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reward_points: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heartbeat_received: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validator_stake: Option<ValidatorStake>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub onekv_candidate_record_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub onekv_rank: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub onekv_location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub onekv_is_valid: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub onekv_offline_since: Option<u64>,
}

#[derive(Clone, Debug, Default, Deserialize, Diff, Eq, Hash, PartialEq, Serialize)]
pub struct ValidatorSummary {
    #[diff_key]
    pub account_id: AccountId,
    pub address: String,
    pub network_id: u32,
    pub controller_account_id: AccountId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_display: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub child_display: Option<String>,
    pub confirmed: bool,
    pub preferences: ValidatorPreferences,
    pub self_stake: StakeSummary,
    pub is_active: bool,
    pub is_active_next_session: bool,
    pub inactive_nominations: InactiveNominationsSummary,
    pub oversubscribed: bool,
    pub slash_count: u64,
    pub is_enrolled_in_1kv: bool,
    pub is_para_validator: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub para_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_rate_per_billion: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocks_authored: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reward_points: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heartbeat_received: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validator_stake: Option<ValidatorStakeSummary>,
}

impl From<&ValidatorDetails> for ValidatorSummary {
    fn from(validator: &ValidatorDetails) -> ValidatorSummary {
        let active_staker_account_ids: Vec<AccountId> =
            if let Some(validator_stake) = &validator.validator_stake {
                validator_stake
                    .nominators
                    .iter()
                    .map(|nominator_stake| nominator_stake.account.id)
                    .collect()
            } else {
                Vec::new()
            };
        let inactive_nominations: Vec<NominationSummary> = validator
            .nominations
            .iter()
            .cloned()
            .filter(|nomination| !active_staker_account_ids.contains(&nomination.stash_account.id))
            .collect();

        ValidatorSummary {
            account_id: validator.account.id,
            address: validator.account.address.clone(),
            network_id: validator.network_id,
            controller_account_id: validator.controller_account_id,
            display: validator.account.get_display(),
            parent_display: validator.account.get_parent_display(),
            child_display: validator.account.child_display.clone(),
            confirmed: validator.account.get_confirmed(),
            preferences: validator.preferences.clone(),
            self_stake: StakeSummary::from(&validator.self_stake),
            is_active: validator.is_active,
            is_para_validator: validator.is_para_validator,
            para_id: validator
                .para_core_assignment
                .as_ref()
                .map(|core_assignment| core_assignment.para_id),
            is_active_next_session: validator.is_active_next_session,
            inactive_nominations: InactiveNominationsSummary::from(&inactive_nominations),
            oversubscribed: validator.oversubscribed,
            slash_count: validator.slash_count,
            is_enrolled_in_1kv: validator.onekv_candidate_record_id.is_some(),
            blocks_authored: validator.blocks_authored,
            reward_points: validator.reward_points,
            heartbeat_received: validator.heartbeat_received,
            return_rate_per_billion: validator.return_rate_per_billion,
            validator_stake: validator
                .validator_stake
                .as_ref()
                .map(ValidatorStakeSummary::from),
        }
    }
}

impl ValidatorSummary {
    pub fn filter(&self, query: &str) -> bool {
        let query = query.to_lowercase();
        self.address.to_lowercase().contains(&query)
            || self
                .display
                .as_ref()
                .map(|display| display.to_lowercase().contains(&query))
                .unwrap_or(false)
            || self
                .parent_display
                .as_ref()
                .map(|display| display.to_lowercase().contains(&query))
                .unwrap_or(false)
            || self
                .child_display
                .as_ref()
                .map(|display| display.to_lowercase().contains(&query))
                .unwrap_or(false)
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct ValidatorListUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finalized_block_number: Option<u64>,
    pub insert: Vec<ValidatorSummary>,
    pub update: Vec<ValidatorSummaryDiff>,
    pub remove_ids: Vec<AccountId>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ValidatorStakeSummary {
    pub self_stake: Balance,
    pub total_stake: Balance,
    pub nominator_count: u64,
}

impl From<&ValidatorStake> for ValidatorStakeSummary {
    fn from(validator_stake: &ValidatorStake) -> Self {
        ValidatorStakeSummary {
            self_stake: validator_stake.self_stake,
            total_stake: validator_stake.total_stake,
            nominator_count: validator_stake.nominators.len() as u64,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ValidatorNominationSummary {
    pub active_nominator_count: usize,
    pub active_nomination_total: Balance,
    pub inactive_nominator_count: usize,
    pub inactive_nomination_total: Balance,
}

impl From<&ValidatorDetails> for ValidatorNominationSummary {
    fn from(validator_details: &ValidatorDetails) -> Self {
        let (
            active_nominator_count,
            active_nomination_total,
            inactive_nominator_count,
            inactive_nomination_total,
        ) = if let Some(validator_stake) = &validator_details.validator_stake {
            let active_nominator_account_ids: Vec<AccountId> = validator_stake
                .nominators
                .iter()
                .map(|n| n.account.id)
                .collect();
            let mut inactive_nominator_count: usize = 0;
            let mut inactive_nomination_total: Balance = 0;
            for nomination in &validator_details.nominations {
                if !active_nominator_account_ids.contains(&nomination.stash_account.id) {
                    inactive_nominator_count += 1;
                    inactive_nomination_total += nomination.stake.active_amount;
                }
            }
            (
                active_nominator_account_ids.len(),
                validator_stake.total_stake,
                inactive_nominator_count,
                inactive_nomination_total,
            )
        } else {
            let inactive_nomination_total: Balance = validator_details
                .nominations
                .iter()
                .map(|n| n.stake.total_amount)
                .sum();
            (
                0,
                0,
                validator_details.nominations.len(),
                inactive_nomination_total,
            )
        };
        ValidatorNominationSummary {
            active_nominator_count,
            active_nomination_total,
            inactive_nominator_count,
            inactive_nomination_total,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct ValidatorSearchSummary {
    pub address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_display: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub child_display: Option<String>,
    pub confirmed: bool,
}

impl From<&ValidatorSummary> for ValidatorSearchSummary {
    fn from(validator_summary: &ValidatorSummary) -> ValidatorSearchSummary {
        ValidatorSearchSummary {
            address: validator_summary.address.clone(),
            display: validator_summary.display.clone(),
            parent_display: validator_summary.parent_display.clone(),
            child_display: validator_summary.child_display.clone(),
            confirmed: validator_summary.confirmed,
        }
    }
}
