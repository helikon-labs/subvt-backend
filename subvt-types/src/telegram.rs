use crate::crypto::AccountId;
use crate::subvt::{ValidatorDetails, ValidatorNominationSummary};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use subvt_utility::numeric::format_decimal;

#[derive(Debug, Eq, PartialEq)]
pub enum TelegramChatState {
    Default,
    AddValidator,
    EnterBugReport,
    EnterFeatureRequest,
}

impl FromStr for TelegramChatState {
    type Err = std::string::ParseError;

    fn from_str(state: &str) -> Result<Self, Self::Err> {
        match state {
            "Default" => Ok(Self::Default),
            "AddValidator" => Ok(Self::AddValidator),
            "EnterBugReport" => Ok(Self::EnterBugReport),
            "EnterFeatureRequest" => Ok(Self::EnterFeatureRequest),
            _ => panic!("Unknown state: {state}"),
        }
    }
}

impl Display for TelegramChatState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let display = match self {
            TelegramChatState::Default => "Default",
            TelegramChatState::AddValidator => "AddValidator",
            TelegramChatState::EnterBugReport => "EnterBugReport",
            TelegramChatState::EnterFeatureRequest => "EnterFeatureRequest",
        };
        write!(f, "{display}")
    }
}

#[derive(Clone, Debug)]
pub struct TelegramChatValidator {
    pub id: u64,
    pub account_id: AccountId,
    pub address: String,
    pub display: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TelegramChatValidatorSummary {
    pub display: String,
    pub address: String,
    pub is_active: bool,
    pub is_active_next_session: bool,
    pub is_para_validator: bool,
    pub is_valid_onekv: Option<bool>,
    pub self_stake_formatted: String,
    pub inactive_nomination_formatted: String,
    pub inactive_nominator_count: usize,
    pub active_nomination_formatted: String,
    pub active_nominator_count: usize,
    pub heartbeat_received: bool,
    pub missing_referendum_votes: Vec<u32>,
}

impl TelegramChatValidatorSummary {
    pub fn from(
        validator_details: &ValidatorDetails,
        token_decimals: usize,
        token_format_decimal_points: usize,
    ) -> TelegramChatValidatorSummary {
        let self_stake_formatted = format_decimal(
            validator_details.self_stake.active_amount,
            token_decimals,
            token_format_decimal_points,
        );
        let nomination_summary: ValidatorNominationSummary = validator_details.into();
        let active_nomination_formatted = format_decimal(
            nomination_summary.active_nomination_total,
            token_decimals,
            token_format_decimal_points,
        );
        let inactive_nomination_formatted = format_decimal(
            nomination_summary.inactive_nomination_total,
            token_decimals,
            token_format_decimal_points,
        );
        let display = validator_details
            .account
            .get_display_or_condensed_address(None);
        TelegramChatValidatorSummary {
            display,
            address: validator_details.account.address.clone(),
            is_active: validator_details.is_active,
            is_active_next_session: validator_details.is_active_next_session,
            is_para_validator: validator_details.is_para_validator,
            is_valid_onekv: validator_details.onekv_is_valid,
            self_stake_formatted,
            inactive_nomination_formatted,
            inactive_nominator_count: nomination_summary.inactive_nominator_count,
            active_nomination_formatted,
            active_nominator_count: nomination_summary.active_nominator_count,
            heartbeat_received: validator_details.heartbeat_received.unwrap_or(false),
            missing_referendum_votes: vec![],
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OneKVBotValidator {
    pub stash_address: String,
    pub name: String,
    pub chat_ids: Vec<i64>,
}
