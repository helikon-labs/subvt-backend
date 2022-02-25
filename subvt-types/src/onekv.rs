//! Types for the Thousand Validators Programme on Kusama and Polkadot.

use crate::substrate::Balance;
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

#[derive(Clone, Debug)]
pub struct OneKVCandidateSummary {
    pub record_id: u32,
    pub onekv_id: String,
    pub discovered_at: u64,
    pub name: String,
    pub nominated_at: Option<u64>,
    pub offline_since: u64,
    pub online_since: u64,
    pub rank: Option<u64>,
    pub total_score: Option<f64>,
    pub aggregate_score: Option<f64>,
    pub telemetry_id: Option<u32>,
    pub validity: Vec<OneKVValidity>,
    pub version: Option<String>,
    pub location: Option<String>,
    pub democracy_vote_count: u32,
    pub council_votes: Vec<String>,
    pub record_created_at: u64,
}

impl OneKVCandidateSummary {
    pub fn is_valid(&self) -> bool {
        self.validity.iter().all(|validity| validity.is_valid)
    }
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
    pub democracy_vote_count: u32,
    pub democracy_votes: Vec<u32>,
    pub council_stake: String,
    pub council_votes: Vec<String>,
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
    #[serde(rename(deserialize = "councilStake"))]
    pub council_stake: Option<f64>,
    pub democracy: Option<f64>,
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

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OneKVNominator {
    #[serde(rename(deserialize = "_id"))]
    pub id: String,
    pub address: String,
    #[serde(rename(deserialize = "stash"))]
    pub stash_address: String,
    #[serde(rename(deserialize = "proxy"))]
    pub proxy_address: String,
    #[serde(rename(deserialize = "bonded"))]
    pub bonded_amount: Balance,
    pub proxy_delay: u32,
    #[serde(rename(deserialize = "current"))]
    pub nominees: Vec<OneKVNominee>,
    #[serde(rename(deserialize = "lastNomination"))]
    pub last_nomination_at: u64,
    pub created_at: u64,
    #[serde(rename(deserialize = "avgStake"))]
    pub average_stake: f64,
    pub new_bonded_amount: f64,
    pub reward_destination: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OneKVNominee {
    pub name: String,
    #[serde(rename(deserialize = "stash"))]
    pub stash_address: String,
    pub identity: OneKVIdentity,
}
