use crate::crypto::AccountId;
use crate::substrate::Balance;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NewNomination {
    pub id: u32,
    pub validator_account_id: AccountId,
    pub discovered_block_number: u64,
    pub nominator_stash_account_id: AccountId,
    pub active_amount: Balance,
    pub total_amount: Balance,
    pub nominee_count: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LostNomination {
    pub id: u32,
    pub validator_account_id: AccountId,
    pub discovered_block_number: u64,
    pub nominator_stash_account_id: AccountId,
    pub active_amount: Balance,
    pub total_amount: Balance,
    pub nominee_count: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NominationAmountChange {
    pub id: u32,
    pub validator_account_id: AccountId,
    pub discovered_block_number: u64,
    pub nominator_stash_account_id: AccountId,
    pub prev_active_amount: Balance,
    pub prev_total_amount: Balance,
    pub prev_nominee_count: u64,
    pub active_amount: Balance,
    pub total_amount: Balance,
    pub nominee_count: u64,
}
