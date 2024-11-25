//! Queries happen as a result of user interaction with inline keyboards, usually a call-to-action
//! or item selection (validator, referendum, NFT, etc.).
use serde::{Deserialize, Serialize};

pub mod process;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Query {
    #[serde(rename = "q")]
    pub query_type: QueryType,
    #[serde(rename = "p")]
    pub parameter: Option<String>,
}

/// Types of all queries, i.e. keyboard actions.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum QueryType {
    #[serde(rename = "CA")]
    Cancel,
    #[serde(rename = "CL")]
    Close,
    #[serde(rename = "CB")]
    ConfirmBroadcast,
    // (page index, whether it's the first load)
    #[serde(rename = "NF")]
    NFTs(usize, bool),
    #[serde(rename = "ND")]
    NominationDetails,
    #[serde(rename = "NDF")]
    NominationDetailsFull,
    #[serde(rename = "NS")]
    NominationSummary,
    #[serde(rename = "NOP")]
    NoOp,
    #[serde(rename = "PA")]
    Payouts,
    #[serde(rename = "RD")]
    ReferendumDetails,
    #[serde(rename = "RT")]
    ReferendumTracks,
    #[serde(rename = "RAV")]
    RemoveAllValidators,
    #[serde(rename = "RV")]
    RemoveValidator,
    #[serde(rename = "RW")]
    Rewards,
    #[serde(rename = "RB")]
    ReportBug,
    #[serde(rename = "RFR")]
    ReportFeatureRequest,
    #[serde(rename = "SE")]
    SettingsEdit(SettingsEditQueryType),
    #[serde(rename = "SN")]
    SettingsNavigate(SettingsSubSection),
    #[serde(rename = "VI")]
    ValidatorInfo,
}

/// `/settings` command keyboard subsections.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum SettingsSubSection {
    #[serde(rename = "R")]
    Root,
    #[serde(rename = "VA")]
    ValidatorActivity,
    #[serde(rename = "N")]
    Nominations,
    #[serde(rename = "NN")]
    NewNomination,
    #[serde(rename = "LN")]
    LostNomination,
    #[serde(rename = "AI")]
    ActiveInactive,
    #[serde(rename = "BA")]
    BlockAuthorship,
    #[serde(rename = "PV")]
    ParaValidation,
    #[serde(rename = "OKV")]
    OneKV,
    #[serde(rename = "OG")]
    Referenda,
}

/// Query to edit a settings sub-item, i.e. notification type.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum SettingsEditQueryType {
    #[serde(rename = "BA")]
    BlockAuthorship,
    #[serde(rename = "A")]
    Active,
    #[serde(rename = "ANS")]
    ActiveNextSession,
    #[serde(rename = "IA")]
    Inactive,
    #[serde(rename = "IANS")]
    InactiveNextSession,
    #[serde(rename = "CHL")]
    Chilled,
    #[serde(rename = "IC")]
    IdentityChanged,
    #[serde(rename = "OO")]
    OfflineOffence,
    #[serde(rename = "PS")]
    PayoutStakers,
    #[serde(rename = "SKC")]
    SessionKeysChanged,
    #[serde(rename = "SC")]
    SetController,
    #[serde(rename = "UP")]
    UnclaimedPayout,
    #[serde(rename = "NN")]
    NewNomination,
    #[serde(rename = "LN")]
    LostNomination,
    #[serde(rename = "STAPV")]
    StartedParaValidating,
    #[serde(rename = "STOPV")]
    StoppedParaValidating,
    #[serde(rename = "OKVR")]
    OneKVRankChange,
    #[serde(rename = "OKVV")]
    OneKVValidityChange,
    #[serde(rename = "OKVL")]
    OneKVLocationChange,
    #[serde(rename = "OKVO")]
    OneKVOnlineStatusChange,
    #[serde(rename = "RA")]
    ReferendumApproved,
    #[serde(rename = "RCA")]
    ReferendumCancelled,
    #[serde(rename = "RCO")]
    ReferendumConfirmed,
    #[serde(rename = "RDS")]
    ReferendumDecisionStarted,
    #[serde(rename = "RK")]
    ReferendumKilled,
    #[serde(rename = "RR")]
    ReferendumRejected,
    #[serde(rename = "RS")]
    ReferendumSubmitted,
    #[serde(rename = "RTO")]
    ReferendumTimedOut,
}
