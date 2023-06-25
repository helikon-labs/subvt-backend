use crate::substrate::Balance;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ReferendumApprovedEvent {
    pub id: u32,
    pub block_hash: String,
    pub extrinsic_index: Option<u32>,
    pub event_index: u32,
    pub referendum_index: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ReferendumCancelledEvent {
    pub id: u32,
    pub block_hash: String,
    pub extrinsic_index: Option<u32>,
    pub event_index: u32,
    pub referendum_index: u32,
    pub ayes: Balance,
    pub nays: Balance,
    pub support: Balance,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ReferendumConfirmedEvent {
    pub id: u32,
    pub block_hash: String,
    pub extrinsic_index: Option<u32>,
    pub event_index: u32,
    pub referendum_index: u32,
    pub ayes: Balance,
    pub nays: Balance,
    pub support: Balance,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ReferendumDecisionStartedEvent {
    pub id: u32,
    pub block_hash: String,
    pub extrinsic_index: Option<u32>,
    pub event_index: u32,
    pub referendum_index: u32,
    pub track_id: u16,
    pub ayes: Balance,
    pub nays: Balance,
    pub support: Balance,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ReferendumKilledEvent {
    pub id: u32,
    pub block_hash: String,
    pub extrinsic_index: Option<u32>,
    pub event_index: u32,
    pub referendum_index: u32,
    pub ayes: Balance,
    pub nays: Balance,
    pub support: Balance,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ReferendumRejectedEvent {
    pub id: u32,
    pub block_hash: String,
    pub extrinsic_index: Option<u32>,
    pub event_index: u32,
    pub referendum_index: u32,
    pub ayes: Balance,
    pub nays: Balance,
    pub support: Balance,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ReferendumSubmittedEvent {
    pub id: u32,
    pub block_hash: String,
    pub extrinsic_index: Option<u32>,
    pub event_index: u32,
    pub referendum_index: u32,
    pub track_id: u16,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ReferendumTimedOutEvent {
    pub id: u32,
    pub block_hash: String,
    pub extrinsic_index: Option<u32>,
    pub event_index: u32,
    pub referendum_index: u32,
    pub ayes: Balance,
    pub nays: Balance,
    pub support: Balance,
}
