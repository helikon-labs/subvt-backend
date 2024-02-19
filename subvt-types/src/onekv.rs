//! Types for the Thousand Validators Programme on Kusama and Polkadot.

use crate::crypto::AccountId;
use crate::substrate::Balance;
use serde::{Deserialize, Serialize};
use subvt_proc_macro::Diff;

#[derive(Clone, Debug, Deserialize, Diff, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OneKVCandidate {
    pub identity: Option<OneKVIdentity>,
    pub commission: Option<f32>,
    pub discovered_at: u64,
    pub inclusion: f32,
    #[serde(rename(deserialize = "active"))]
    pub is_active: Option<bool>,
    #[serde(rename(deserialize = "kusamaStash"))]
    pub kusama_stash_address: String,
    pub name: String,
    pub nominated_at: Option<u64>,
    pub offline_accumulated: i64,
    pub offline_since: u64,
    pub rank: i64,
    #[serde(rename(deserialize = "faults"))]
    pub fault_count: i64,
    pub score: Option<OneKVScore>,
    #[serde(rename(deserialize = "stash"))]
    pub stash_address: String,
    pub unclaimed_eras: Option<Vec<u32>>,
    pub validity: Option<Vec<OneKVValidity>>,
    pub location: Option<String>,
    pub provider: Option<String>,
    pub conviction_vote_count: u32,
    pub conviction_votes: Vec<u32>,
}

impl OneKVCandidate {
    pub fn is_valid(&self) -> bool {
        if let Some(validity) = &self.validity {
            validity.iter().all(|validity| validity.is_valid)
        } else {
            false
        }
    }
}

#[derive(Clone, Debug)]
pub struct OneKVCandidateSummary {
    pub record_id: u32,
    pub discovered_at: u64,
    pub name: String,
    pub nominated_at: Option<u64>,
    pub offline_since: u64,
    pub rank: Option<u64>,
    pub fault_count: u64,
    pub total_score: Option<f64>,
    pub aggregate_score: Option<f64>,
    pub validity: Vec<OneKVValidity>,
    pub location: Option<String>,
    pub conviction_vote_count: u32,
    pub conviction_votes: Vec<u32>,
    pub record_created_at: u64,
}

impl OneKVCandidateSummary {
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
    #[serde(rename(deserialize = "openGov"))]
    pub opengov: f64,
    #[serde(rename(deserialize = "openGovDelegations"))]
    pub opengov_delegations: f64,
    pub location: Option<f64>,
    pub country: Option<f64>,
    pub provider: Option<f64>,
    #[serde(rename(deserialize = "nominatorStake"))]
    pub nominator_stake: Option<f64>,
    pub region: Option<f64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OneKVIdentity {
    #[serde(rename(deserialize = "_id"))]
    pub id: String,
    pub name: String,
    pub sub_identities: Option<Vec<OneKVSubIdentity>>,
    pub verified: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OneKVSubIdentity {
    #[serde(rename(deserialize = "_id"))]
    pub id: String,
    pub name: String,
    pub address: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OneKVValidity {
    #[serde(rename = "_id")]
    pub id: String,
    pub details: String,
    #[serde(rename = "valid")]
    pub is_valid: bool,
    #[serde(rename = "type")]
    pub ty: String,
    #[serde(rename = "updated")]
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
pub struct OneKVNominatorSummary {
    pub id: u64,
    pub onekv_id: String,
    pub stash_account_id: AccountId,
    pub stash_address: String,
    pub bonded_amount: Balance,
    pub last_nomination_at: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OneKVNominee {
    pub name: String,
    #[serde(rename(deserialize = "stash"))]
    pub stash_address: String,
    pub identity: Option<OneKVIdentity>,
}
