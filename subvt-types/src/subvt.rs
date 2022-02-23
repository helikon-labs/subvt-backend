//! SubVT types. These types are used to send the network status and
//! validator details/summary to Redis in-memory database. This data then gets consumed
//! by other services that require it.

use crate::crypto::AccountId;
use crate::substrate::paras::ParaCoreAssignment;
use crate::substrate::{
    Account, Balance, Epoch, Era, InactiveNominationsSummary, Nomination, RewardDestination, Stake,
    StakeSummary, ValidatorPreferences, ValidatorStake,
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
    pub controller_account_id: AccountId,
    pub preferences: ValidatorPreferences,
    pub self_stake: Stake,
    pub reward_destination: RewardDestination,
    pub next_session_keys: String,
    pub is_active: bool,
    pub active_next_session: bool,
    pub nominations: Vec<Nomination>,
    pub oversubscribed: bool,
    pub active_era_count: u64,
    pub inactive_era_count: u64,
    pub slash_count: u64,
    pub offline_offence_count: u64,
    pub total_reward_points: u64,
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
    pub onekv_is_valid: Option<bool>,
}

#[derive(Clone, Debug, Default, Deserialize, Diff, Eq, Hash, PartialEq, Serialize)]
pub struct ValidatorSummary {
    #[diff_key]
    pub account_id: AccountId,
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
    pub active_next_session: bool,
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

impl ValidatorDetails {
    pub fn get_display(&self) -> Option<String> {
        if let Some(identity) = &self.account.identity {
            identity.display.clone()
        } else {
            None
        }
    }

    pub fn get_parent_display(&self) -> Option<String> {
        if let Some(parent) = &*self.account.parent {
            if let Some(identity) = &parent.identity {
                identity.display.clone()
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_full_display(&self) -> Option<String> {
        if let Some(identity) = &self.account.identity {
            return identity.display.clone();
        } else if let Some(parent) = &*self.account.parent {
            if let Some(identity) = &parent.identity {
                if let Some(parent_display) = &identity.display {
                    if let Some(child_display) = &self.account.child_display {
                        return Some(format!("{} / {}", parent_display, child_display,));
                    }
                }
            }
        }
        None
    }
}

impl From<&ValidatorDetails> for ValidatorSummary {
    fn from(validator: &ValidatorDetails) -> ValidatorSummary {
        let active_staker_account_ids: Vec<AccountId> =
            if let Some(validator_stake) = &validator.validator_stake {
                validator_stake
                    .nominators
                    .iter()
                    .map(|nominator_stake| nominator_stake.account.id.clone())
                    .collect()
            } else {
                Vec::new()
            };
        let inactive_nominations: Vec<Nomination> = validator
            .nominations
            .iter()
            .cloned()
            .filter(|nomination| !active_staker_account_ids.contains(&nomination.stash_account.id))
            .collect();

        ValidatorSummary {
            account_id: validator.account.id.clone(),
            controller_account_id: validator.controller_account_id.clone(),
            display: validator.get_display(),
            parent_display: validator.get_parent_display(),
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
            active_next_session: validator.active_next_session,
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
