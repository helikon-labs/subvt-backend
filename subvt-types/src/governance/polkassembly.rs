use crate::crypto::AccountId;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ReferendumStatus {
    Submitted,
    DecisionDepositPlaced,
    Deciding,
    Executed,
    ExecutionFailed,
    TimedOut,
    Cancelled,
    Rejected,
    ConfirmStarted,
    Confirmed,
    Killed,
}

impl Display for ReferendumStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let display = match self {
            ReferendumStatus::Submitted => "Submitted",
            ReferendumStatus::DecisionDepositPlaced => "DecisionDepositPlaced",
            ReferendumStatus::Deciding => "Deciding",
            ReferendumStatus::Executed => "Executed",
            ReferendumStatus::ExecutionFailed => "ExecutionFailed",
            ReferendumStatus::TimedOut => "TimedOut",
            ReferendumStatus::Cancelled => "Cancelled",
            ReferendumStatus::Rejected => "Rejected",
            ReferendumStatus::ConfirmStarted => "ConfirmStarted",
            ReferendumStatus::Confirmed => "Confirmed",
            ReferendumStatus::Killed => "Killed",
        };
        write!(f, "{display}")
    }
}

impl FromStr for ReferendumStatus {
    type Err = std::string::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Submitted" => Ok(Self::Submitted),
            "DecisionDepositPlaced" => Ok(Self::DecisionDepositPlaced),
            "Deciding" => Ok(Self::Deciding),
            "Executed" => Ok(Self::Executed),
            "ExecutionFailed" => Ok(Self::ExecutionFailed),
            "TimedOut" => Ok(Self::TimedOut),
            "Cancelled" => Ok(Self::Cancelled),
            "Rejected" => Ok(Self::Rejected),
            "Confirmed" => Ok(Self::Confirmed),
            _ => panic!("Unknown referendum status: {s}"),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ReferendaQueryResponse {
    pub count: u32,
    pub posts: Vec<ReferendumPost>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ReferendumPost {
    pub post_id: u32,
    pub track_no: u16,
    pub proposer: AccountId,
    #[serde(rename = "title")]
    pub maybe_title: Option<String>,
    #[serde(rename = "method")]
    pub maybe_method: Option<String>,
    pub status: ReferendumStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "type")]
    pub ty: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ReferendumPostDetails {
    pub post_id: u32,
    pub proposer: String,
    #[serde(rename = "title")]
    pub maybe_title: Option<String>,
    pub status: ReferendumStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "content")]
    pub maybe_content: Option<String>,
    #[serde(rename = "end")]
    pub end_block_number: Option<u32>,
    pub ended_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(rename = "track_number")]
    pub track_id: u16,
}
