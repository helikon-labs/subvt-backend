//! Types for the Thousand Validators Programme on Kusama and Polkadot.

use serde::{Deserialize, Serialize};
use subvt_proc_macro::Diff;

#[derive(Clone, Debug, Deserialize, Diff, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    #[serde(rename(deserialize = "kusamaStash"))]
    pub kusama_stash_address: String,
    #[serde(rename(deserialize = "stash"))]
    pub stash_address: String,
    pub score: Option<Score>,
}

#[derive(Clone, Debug, Deserialize, Diff, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CandidateDetails {
    #[serde(rename(deserialize = "_id"))]
    pub id: String,
    pub bonded: Option<u128>,
    pub commission: f32,
    pub controller: String,
    pub discovered_at: u64,
    pub fault_events: Vec<FaultEvent>,
    pub inclusion: f32,
    #[serde(rename(deserialize = "active"))]
    pub is_active: bool,
    #[serde(rename(deserialize = "kusamaStash"))]
    pub kusama_stash_address: String,
    pub last_valid: Option<u64>,
    pub name: String,
    pub nominated_at: u64,
    pub offline_accumulated: u64,
    pub offline_since: u64,
    pub online_since: u64,
    pub node_refs: u32,
    pub rank: i32,
    pub rank_events: Vec<RankEvent>,
    pub reward_destination: String,
    pub score: Option<Score>,
    pub span_inclusion: f32,
    #[serde(rename(deserialize = "stash"))]
    pub stash_address: String,
    pub telemetry_id: Option<u32>,
    pub unclaimed_eras: Option<Vec<u32>>,
    #[serde(rename(deserialize = "invalidity"))]
    pub validity: Vec<Validity>,
    pub version: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Diff, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Score {
    pub updated: u64,
    pub total: f64,
    pub aggregate: f64,
    pub inclusion: f64,
    pub discovered: f64,
    pub nominated: f64,
    pub rank: f64,
    pub unclaimed: f64,
    pub bonded: f64,
    pub faults: f64,
    pub offline: f64,
    pub randomness: f64,
    pub span_inclusion: f64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FaultEvent {
    #[serde(rename(deserialize = "_id"))]
    pub id: String,
    pub prev_rank: Option<i32>,
    pub reason: String,
    pub when: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RankEvent {
    #[serde(rename(deserialize = "_id"))]
    pub id: String,
    pub active_era: u32,
    pub start_era: u32,
    pub when: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Identity {
    #[serde(rename(deserialize = "_id"))]
    pub id: String,
    pub name: String,
    pub sub: String,
    pub verified: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Validity {
    #[serde(rename(deserialize = "_id"))]
    pub id: String,
    pub details: String,
    #[serde(rename(deserialize = "valid"))]
    pub is_valid: bool,
    #[serde(rename(deserialize = "type"))]
    pub ty: String,
    #[serde(rename(deserialize = "updated"))]
    pub updated_at: u64,
}
