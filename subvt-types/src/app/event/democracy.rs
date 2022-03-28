use crate::crypto::AccountId;
use crate::substrate::Balance;
pub use pallet_democracy::{AccountVote, Conviction, VoteThreshold};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DemocracyCancelledEvent {
    pub id: u32,
    pub block_hash: String,
    pub extrinsic_index: Option<u32>,
    pub event_index: u32,
    pub referendum_index: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DemocracyDelegatedEvent {
    pub id: u32,
    pub block_hash: String,
    pub extrinsic_index: Option<u32>,
    pub event_index: u32,
    pub original_account_id: AccountId,
    pub delegate_account_id: AccountId,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DemocracyNotPassedEvent {
    pub id: u32,
    pub block_hash: String,
    pub extrinsic_index: Option<u32>,
    pub event_index: u32,
    pub referendum_index: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DemocracyPassedEvent {
    pub id: u32,
    pub block_hash: String,
    pub extrinsic_index: Option<u32>,
    pub event_index: u32,
    pub referendum_index: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DemocracyProposedEvent {
    pub id: u32,
    pub block_hash: String,
    pub extrinsic_index: Option<u32>,
    pub event_index: u32,
    pub proposal_index: u64,
    pub deposit: Balance,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DemocracySecondedEvent {
    pub id: u32,
    pub block_hash: String,
    pub extrinsic_index: Option<u32>,
    pub event_index: u32,
    pub account_id: AccountId,
    pub proposal_index: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DemocracyStartedEvent {
    pub id: u32,
    pub block_hash: String,
    pub extrinsic_index: Option<u32>,
    pub event_index: u32,
    pub referendum_index: u64,
    pub vote_threshold: VoteThreshold,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DemocracyUndelegatedEvent {
    pub id: u32,
    pub block_hash: String,
    pub extrinsic_index: Option<u32>,
    pub event_index: u32,
    pub account_id: AccountId,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DemocracyVotedEvent {
    pub id: u32,
    pub block_hash: String,
    pub extrinsic_index: Option<u32>,
    pub event_index: u32,
    pub account_id: AccountId,
    pub referendum_index: u64,
    pub aye_balance: Option<Balance>,
    pub nay_balance: Option<Balance>,
    pub conviction: Option<u8>,
}
