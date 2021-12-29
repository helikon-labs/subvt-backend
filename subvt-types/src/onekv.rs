//! Types for the Thousand Validators Programme on Kusama and Polkadot.

use serde::{Deserialize, Serialize};
use subvt_proc_macro::Diff;

#[derive(Clone, Debug, Deserialize, Diff, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OneKVCandidate {
    #[serde(rename(deserialize = "kusamaStash"))]
    pub kusama_stash_address: String,
    #[serde(rename(deserialize = "stash"))]
    pub stash_address: String,
    pub score: Option<OneKVScore>,
}

#[derive(Clone, Debug, Deserialize, Diff, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OneKVCandidateDetails {
    #[serde(rename(deserialize = "_id"))]
    pub id: String,
    pub identity: OneKVIdentity,
    pub bonded: Option<u128>,
    pub commission: f32,
    pub controller: String,
    pub discovered_at: u64,
    pub fault_events: Vec<OneKVFaultEvent>,
    pub inclusion: f32,
    #[serde(rename(deserialize = "active"))]
    pub is_active: bool,
    #[serde(rename(deserialize = "kusamaStash"))]
    pub kusama_stash_address: String,
    pub last_valid: Option<u64>,
    pub name: String,
    pub nominated_at: Option<u64>,
    pub offline_accumulated: u64,
    pub offline_since: u64,
    pub online_since: u64,
    pub node_refs: u32,
    pub rank: i64,
    pub rank_events: Vec<OneKVRankEvent>,
    pub reward_destination: String,
    pub score: Option<OneKVScore>,
    pub span_inclusion: f32,
    #[serde(rename(deserialize = "stash"))]
    pub stash_address: String,
    pub telemetry_id: Option<u32>,
    pub unclaimed_eras: Option<Vec<u32>>,
    #[serde(rename(deserialize = "invalidity"))]
    pub validity: Vec<OneKVValidity>,
    pub version: Option<String>,
    pub location: Option<String>,
}

impl OneKVCandidateDetails {
    pub fn is_valid(&self) -> bool {
        self.validity.iter().all(|validity| validity.is_valid)
    }
}

#[derive(Clone, Debug, Deserialize, Diff, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OneKVScore {
    #[serde(rename(deserialize = "updated"))]
    pub updated_at: u64,
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
    pub location: Option<f64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OneKVFaultEvent {
    #[serde(rename(deserialize = "_id"))]
    pub id: String,
    #[serde(rename(deserialize = "prevRank"))]
    pub previous_rank: Option<i32>,
    pub reason: String,
    pub when: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OneKVRankEvent {
    #[serde(rename(deserialize = "_id"))]
    pub id: String,
    pub active_era: u32,
    pub start_era: u32,
    pub when: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OneKVIdentity {
    #[serde(rename(deserialize = "_id"))]
    pub id: String,
    pub name: String,
    pub sub: Option<String>,
    pub verified: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OneKVValidity {
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
