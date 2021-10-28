//! Types for the Thousand Validators Programme on Kusama and Polkadot.

use serde::{Deserialize, Serialize};
use subvt_proc_macro::Diff;

#[derive(Clone, Debug, Deserialize, Diff, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    #[serde(rename(deserialize = "_id"))]
    id: String,
    bonded: u128,
    commission: f32,
    controller: String,
    discovered_at: u64,
    fault_events: Vec<FaultEvent>,
    inclusion: f32,
    #[serde(rename(deserialize = "active"))]
    is_active: bool,
    kusama_stash: String,
    last_valid: u64,
    name: String,
    nominated_at: u64,
    offline_accumulated: u64,
    offline_since: u64,
    online_since: u64,
    node_refs: u32,
    rank: u32,
    rank_events: Vec<RankEvent>,
    reward_destination: String,
    span_inclusion: f32,
    stash: String,
    telemetry_id: u32,
    unclaimed_eras: Vec<u32>,
    #[serde(rename(deserialize = "invalidity"))]
    validity: Vec<Validity>,
    version: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FaultEvent {
    #[serde(rename(deserialize = "_id"))]
    id: String,
    prev_rank: u32,
    reason: String,
    when: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RankEvent {
    #[serde(rename(deserialize = "_id"))]
    id: String,
    active_era: u32,
    start_era: u32,
    when: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Identity {
    #[serde(rename(deserialize = "_id"))]
    id: String,
    name: String,
    sub: String,
    verified: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Validity {
    #[serde(rename(deserialize = "_id"))]
    id: String,
    details: String,
    #[serde(rename(deserialize = "valid"))]
    is_valid: bool,
    #[serde(rename(deserialize = "type"))]
    ty: String,
    #[serde(rename(deserialize = "updated"))]
    updated_at: u64,
}
