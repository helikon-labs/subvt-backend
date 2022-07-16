//! SubVT application events, on top of the Substrate events.
use crate::crypto::AccountId;
use crate::onekv::OneKVValidity;
use crate::substrate::Balance;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NewNomination {
    pub validator_account_id: AccountId,
    pub discovered_block_number: u64,
    pub nominator_stash_account_id: AccountId,
    pub active_amount: Balance,
    pub total_amount: Balance,
    pub nominee_count: u64,
    pub is_onekv: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LostNomination {
    pub validator_account_id: AccountId,
    pub discovered_block_number: u64,
    pub nominator_stash_account_id: AccountId,
    pub active_amount: Balance,
    pub total_amount: Balance,
    pub nominee_count: u64,
    pub is_onekv: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NominationAmountChange {
    pub validator_account_id: AccountId,
    pub discovered_block_number: u64,
    pub nominator_stash_account_id: AccountId,
    pub prev_active_amount: Balance,
    pub prev_total_amount: Balance,
    pub prev_nominee_count: u64,
    pub active_amount: Balance,
    pub total_amount: Balance,
    pub nominee_count: u64,
    pub is_onekv: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OneKVBinaryVersionChange {
    pub validator_account_id: AccountId,
    pub prev_version: Option<String>,
    pub current_version: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OneKVRankChange {
    pub validator_account_id: AccountId,
    pub prev_rank: Option<u64>,
    pub current_rank: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OneKVLocationChange {
    pub validator_account_id: AccountId,
    pub prev_location: Option<String>,
    pub current_location: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OneKVValidityChange {
    pub validator_account_id: AccountId,
    pub is_valid: bool,
    pub validity_items: Vec<OneKVValidity>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OneKVOnlineStatusChange {
    pub validator_account_id: AccountId,
    pub online_since: u64,
    pub offline_since: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CommissionChange {
    pub validator_account_id: AccountId,
    pub previous_commission_per_billion: u32,
    pub current_commission_per_billion: u32,
}
